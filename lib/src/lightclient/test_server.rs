use std::cmp;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use futures::{FutureExt, Stream};
use orchard::tree::MerkleHashOrchard;
use rand::rngs::OsRng;
use rand::Rng;
use tempdir::TempDir;
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use zcash_primitives::block::BlockHash;
use zcash_primitives::consensus::{self, BlockHeight, BranchId};
use zcash_primitives::merkle_tree::CommitmentTree;
use zcash_primitives::sapling::Node;
use zcash_primitives::transaction::{Transaction, TxId};

use super::config::{LightClientConfig, UnitTestNetwork};
use super::LightClient;
use crate::grpc::compact_tx_streamer_server::{CompactTxStreamer, CompactTxStreamerServer};
use crate::grpc::{
    Address, AddressList, Balance, BlockId, BlockRange, ChainSpec, CompactBlock, CompactTx, Duration, Empty, Exclude,
    GetAddressUtxosArg, GetAddressUtxosReply, GetAddressUtxosReplyList, LightdInfo, PingResponse, PriceRequest,
    PriceResponse, RawTransaction, SendResponse, TransparentAddressBlockFilter, TreeState, TxFilter,
};
use crate::lightclient::blaze::test_utils::{orchardtree_to_string, tree_to_string, FakeCompactBlockList};
use crate::lightwallet::data::wallettx::WalletTx;
use crate::lightwallet::utils::now;

pub async fn create_test_server<P: consensus::Parameters + Send + Sync + 'static>(
    params: P
) -> (
    Arc<RwLock<TestServerData<P>>>,
    LightClientConfig<P>,
    oneshot::Receiver<bool>,
    oneshot::Sender<bool>,
    JoinHandle<()>,
) {
    let (ready_tx, ready_rx) = oneshot::channel();
    let (stop_tx, stop_rx) = oneshot::channel();
    let (data_dir_tx, data_dir_rx) = oneshot::channel();

    let port = portpicker::pick_unused_port().expect("Failed to pick an unused port");
    let server_port = format!("127.0.0.1:{}", port);
    let uri = format!("http://{}", server_port);
    let addr = server_port
        .parse()
        .expect("Failed to parse server port");

    let mut config = LightClientConfig::new_unconnected(params, None);
    config.server = uri
        .parse()
        .expect("Failed to parse URI");

    let (service, data) = TestGRPCService::new(config.clone());

    let server_handle = tokio::spawn(async move {
        let temp_dir = TempDir::new(&format!("test{}", port)).expect("Failed to create a temporary directory");

        // Send the path name. Do into_path() to preserve the temp directory
        data_dir_tx
            .send(
                temp_dir
                    .into_path()
                    .canonicalize()
                    .expect("Failed to canonicalize path")
                    .to_str()
                    .expect("Failed to convert path to string")
                    .to_string(),
            )
            .expect("Failed to send data directory");

        ready_tx
            .send(true)
            .expect("Failed to send ready signal");

        // Create a new service
        let svc = CompactTxStreamerServer::new(service);

        // Start
        Server::builder()
            .add_service(svc)
            .serve_with_shutdown(addr, stop_rx.map(drop))
            .await
            .expect("Server failed to run");

        println!("Server stopped");
    });

    let data_dir = data_dir_rx
        .await
        .expect("Failed to receive data directory");
    println!("GRPC Server listening on: {}. With datadir {}", addr, data_dir);
    config.data_dir = Some(data_dir);

    (data, config, ready_rx, stop_tx, server_handle)
}

pub async fn mine_random_blocks<P: consensus::Parameters + Send + Sync + 'static>(
    fcbl: &mut FakeCompactBlockList,
    data: &Arc<RwLock<TestServerData<P>>>,
    lc: &LightClient<P>,
    num: u64,
) {
    let cbs = fcbl
        .add_blocks(num)
        .into_compact_blocks();

    data.write()
        .await
        .add_blocks(cbs.clone());
    lc.do_sync(true)
        .await
        .expect("Failed to sync");
}

pub async fn mine_pending_blocks<P: consensus::Parameters + Send + Sync + 'static>(
    fcbl: &mut FakeCompactBlockList,
    data: &Arc<RwLock<TestServerData<P>>>,
    lc: &LightClient<P>,
) {
    let cbs = fcbl.into_compact_blocks();
    println!("mining pending block {}", cbs[0].vtx[0].outputs.len());

    data.write()
        .await
        .add_blocks(cbs.clone());
    let mut v = fcbl.into_txns();

    // Add all the t-addr spend's t-addresses into the maps, so the test grpc server
    // knows to serve this tx when the txns for this particular taddr are requested.
    for (t, _h, taddrs) in v.iter_mut() {
        if let Some(t_bundle) = t.transparent_bundle() {
            for vin in &t_bundle.vin {
                let prev_txid = WalletTx::new_txid(&vin.prevout.hash().to_vec());
                if let Some(wtx) = lc
                    .wallet
                    .txs
                    .read()
                    .await
                    .current
                    .get(&prev_txid)
                {
                    if let Some(utxo) = wtx
                        .utxos
                        .iter()
                        .find(|u| u.output_index as u32 == vin.prevout.n())
                    {
                        if !taddrs.contains(&utxo.address) {
                            taddrs.push(utxo.address.clone());
                        }
                    }
                }
            }
        }
    }

    data.write().await.add_txns(v);

    lc.do_sync(true)
        .await
        .expect("Failed to sync");
}

#[derive(Debug)]
pub struct TestServerData<P> {
    pub blocks: Vec<CompactBlock>,
    pub txns: HashMap<TxId, (Vec<String>, RawTransaction)>,
    pub sent_txns: Vec<RawTransaction>,
    pub config: LightClientConfig<P>,
    pub zec_price: f64,
    pub tree_states: Vec<(u64, String, String)>,
}

impl<P: consensus::Parameters> TestServerData<P> {
    pub fn new(config: LightClientConfig<P>) -> Self {
        let data = Self {
            blocks: vec![],
            txns: HashMap::new(),
            sent_txns: vec![],
            config,
            zec_price: 140.5,
            tree_states: vec![],
        };

        data
    }

    pub fn add_txns(
        &mut self,
        txns: Vec<(Transaction, u64, Vec<String>)>,
    ) {
        for (tx, height, taddrs) in txns {
            let mut rtx = RawTransaction::default();
            let mut data = vec![];
            tx.write(&mut data)
                .expect("Failed to write transaction");
            rtx.data = data;
            rtx.height = height;
            self.txns
                .insert(tx.txid(), (taddrs, rtx));
        }
    }

    pub fn add_blocks(
        &mut self,
        cbs: Vec<CompactBlock>,
    ) {
        if cbs.is_empty() {
            panic!("No blocks");
        }

        if cbs.len() > 1 && cbs.first().unwrap().height < cbs.last().unwrap().height {
            panic!(
                "Blocks are in the wrong order. First={} Last={}",
                cbs.first().unwrap().height,
                cbs.last().unwrap().height
            );
        }

        if !self.blocks.is_empty() && self.blocks.first().unwrap().height + 1 != cbs.last().unwrap().height {
            panic!(
                "New blocks are in wrong order. expecting={}, got={}",
                self.blocks.first().unwrap().height + 1,
                cbs.last().unwrap().height
            );
        }

        for blk in cbs.into_iter().rev() {
            self.blocks.insert(0, blk);
        }
    }
}

#[derive(Debug)]
pub struct TestGRPCService<P> {
    data: Arc<RwLock<TestServerData<P>>>,
}

impl<P: consensus::Parameters> TestGRPCService<P> {
    pub fn new(config: LightClientConfig<P>) -> (Self, Arc<RwLock<TestServerData<P>>>) {
        let test_server_data = Arc::new(RwLock::new(TestServerData::new(config)));
        let test_grpc_service = Self { data: test_server_data.clone() };

        (test_grpc_service, test_server_data)
    }

    async fn wait_random() {
        let msecs = OsRng.gen_range(0 .. 100);
        sleep(std::time::Duration::from_millis(msecs)).await;
    }
}

#[tonic::async_trait]
impl<P: consensus::Parameters + Send + Sync + 'static> CompactTxStreamer for TestGRPCService<P> {
    async fn get_latest_block(
        &self,
        _request: Request<ChainSpec>,
    ) -> Result<Response<BlockId>, Status> {
        Self::wait_random().await;

        match self
            .data
            .read()
            .await
            .blocks
            .iter()
            .max_by_key(|b| b.height)
        {
            Some(latest_block) => {
                Ok(Response::new(BlockId { height: latest_block.height, hash: latest_block.hash.clone() }))
            },
            None => Err(Status::unavailable("No blocks")),
        }
    }

    async fn get_block(
        &self,
        request: Request<BlockId>,
    ) -> Result<Response<CompactBlock>, Status> {
        Self::wait_random().await;

        let height = request.into_inner().height;

        match self
            .data
            .read()
            .await
            .blocks
            .iter()
            .find(|b| b.height == height)
        {
            Some(b) => Ok(Response::new(b.clone())),
            None => Err(Status::unavailable(format!("Block {} not found", height))),
        }
    }

    type GetBlockRangeStream = Pin<Box<dyn Stream<Item = Result<CompactBlock, Status>> + Send + Sync>>;
    async fn get_block_range(
        &self,
        request: Request<BlockRange>,
    ) -> Result<Response<Self::GetBlockRangeStream>, Status> {
        let request = request.into_inner();
        let start = request.start.unwrap().height;
        let end = request.end.unwrap().height;

        let rev = start < end;

        let (tx, rx) = mpsc::channel(self.data.read().await.blocks.len());

        let blocks = self.data.read().await.blocks.clone();
        tokio::spawn(async move {
            let (iter, min, max) =
                if rev { (blocks.iter().rev().cloned().collect(), start, end) } else { (blocks, end, start) };
            for b in iter {
                if b.height >= min && b.height <= max {
                    Self::wait_random().await;
                    tx.send(Ok(b)).await.unwrap();
                }
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }

    async fn get_zec_price(
        &self,
        _request: Request<PriceRequest>,
    ) -> Result<Response<PriceResponse>, Status> {
        self.get_current_zec_price(Request::new(Empty {}))
            .await
    }

    async fn get_current_zec_price(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<PriceResponse>, Status> {
        Self::wait_random().await;

        let mut res = PriceResponse::default();
        res.currency = "USD".to_string();
        res.timestamp = now() as i64;
        res.price = self.data.read().await.zec_price;

        Ok(Response::new(res))
    }

    async fn get_transaction(
        &self,
        request: Request<TxFilter>,
    ) -> Result<Response<RawTransaction>, Status> {
        Self::wait_random().await;

        let txid = WalletTx::new_txid(&request.into_inner().hash);
        match self.data.read().await.txns.get(&txid) {
            Some((_taddrs, tx)) => Ok(Response::new(tx.clone())),
            None => Err(Status::invalid_argument(format!("Can't find txid {}", txid))),
        }
    }

    async fn send_transaction(
        &self,
        request: Request<RawTransaction>,
    ) -> Result<Response<SendResponse>, Status> {
        let rtx = request.into_inner();
        let txid = Transaction::read(
            &rtx.data[..],
            BranchId::for_height(&UnitTestNetwork, BlockHeight::from_u32(rtx.height as u32)),
        )
        .unwrap()
        .txid();

        self.data
            .write()
            .await
            .sent_txns
            .push(rtx);
        Ok(Response::new(SendResponse { error_message: txid.to_string(), error_code: 0 }))
    }

    type GetTaddressTxidsStream = Pin<Box<dyn Stream<Item = Result<RawTransaction, Status>> + Send + Sync>>;

    async fn get_taddress_txids(
        &self,
        request: Request<TransparentAddressBlockFilter>,
    ) -> Result<Response<Self::GetTaddressTxidsStream>, Status> {
        let buf_size = cmp::max(self.data.read().await.txns.len(), 1);
        let (tx, rx) = mpsc::channel(buf_size);

        let request = request.into_inner();
        let taddr = request.address;
        let start_block = request
            .range
            .as_ref()
            .unwrap()
            .start
            .as_ref()
            .unwrap()
            .height;
        let end_block = request
            .range
            .as_ref()
            .unwrap()
            .end
            .as_ref()
            .unwrap()
            .height;

        let txns = self.data.read().await.txns.clone();
        tokio::spawn(async move {
            let mut txns_to_send = txns
                .values()
                .filter_map(|(taddrs, rtx)| {
                    if taddrs.contains(&taddr) && rtx.height >= start_block && rtx.height <= end_block {
                        Some(rtx.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            txns_to_send.sort_by_key(|rtx| rtx.height);

            for rtx in txns_to_send {
                Self::wait_random().await;

                tx.send(Ok(rtx)).await.unwrap();
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }

    type GetAddressTxidsStream = Pin<Box<dyn Stream<Item = Result<RawTransaction, Status>> + Send + Sync>>;

    async fn get_address_txids(
        &self,
        request: Request<TransparentAddressBlockFilter>,
    ) -> Result<Response<Self::GetAddressTxidsStream>, Status> {
        self.get_taddress_txids(request).await
    }

    async fn get_taddress_balance(
        &self,
        _request: Request<AddressList>,
    ) -> Result<Response<Balance>, Status> {
        todo!()
    }

    async fn get_taddress_balance_stream(
        &self,
        _request: Request<tonic::Streaming<Address>>,
    ) -> Result<Response<Balance>, Status> {
        todo!()
    }

    type GetMempoolTxStream = Pin<Box<dyn Stream<Item = Result<CompactTx, Status>> + Send + Sync>>;

    async fn get_mempool_tx(
        &self,
        _request: Request<Exclude>,
    ) -> Result<Response<Self::GetMempoolTxStream>, Status> {
        todo!()
    }

    async fn get_tree_state(
        &self,
        request: Request<BlockId>,
    ) -> Result<Response<TreeState>, Status> {
        Self::wait_random().await;

        let block = request.into_inner();
        println!("Getting tree state at {}", block.height);

        // See if it is manually set.
        if let Some((height, hash, tree)) = self
            .data
            .read()
            .await
            .tree_states
            .iter()
            .find(|(h, _, _)| *h == block.height)
        {
            let mut ts = TreeState::default();
            ts.height = *height;
            ts.hash = hash.clone();
            ts.tree = tree.clone();

            return Ok(Response::new(ts));
        }

        let start_block = self
            .data
            .read()
            .await
            .blocks
            .last()
            .map(|b| b.height - 1)
            .unwrap_or(0);

        let start_tree = self
            .data
            .read()
            .await
            .tree_states
            .iter()
            .find(|(h, _, _)| *h == start_block)
            .map(|(_, _, t)| CommitmentTree::<Node>::read(&hex::decode(t).unwrap()[..]).unwrap())
            .unwrap_or(CommitmentTree::<Node>::empty());

        let tree = self
            .data
            .read()
            .await
            .blocks
            .iter()
            .rev()
            .take_while(|cb| cb.height <= block.height)
            .fold(start_tree, |mut tree, cb| {
                for tx in &cb.vtx {
                    for co in &tx.outputs {
                        tree.append(Node::new(co.cmu().unwrap().into()))
                            .unwrap();
                    }
                }

                tree
            });

        let mut ts = TreeState::default();
        let hash = if let Some(b) = self
            .data
            .read()
            .await
            .blocks
            .iter()
            .find(|cb| cb.height == block.height)
        {
            b.hash.clone()
        } else {
            [0u8; 32].to_vec()
        };

        ts.hash = BlockHash::from_slice(&hash[..]).to_string();
        ts.height = block.height;
        ts.tree = tree_to_string(&tree);
        ts.orchard_tree = orchardtree_to_string(&CommitmentTree::<MerkleHashOrchard>::empty());

        Ok(Response::new(ts))
    }

    async fn get_address_utxos(
        &self,
        _request: Request<GetAddressUtxosArg>,
    ) -> Result<Response<GetAddressUtxosReplyList>, Status> {
        todo!()
    }

    type GetAddressUtxosStreamStream = Pin<Box<dyn Stream<Item = Result<GetAddressUtxosReply, Status>> + Send + Sync>>;

    async fn get_address_utxos_stream(
        &self,
        _request: Request<GetAddressUtxosArg>,
    ) -> Result<Response<Self::GetAddressUtxosStreamStream>, Status> {
        todo!()
    }

    async fn get_lightd_info(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<LightdInfo>, Status> {
        Self::wait_random().await;

        let mut ld = LightdInfo::default();
        ld.version = "Test GRPC Server".to_string();
        ld.block_height = self
            .data
            .read()
            .await
            .blocks
            .iter()
            .map(|b| b.height)
            .max()
            .unwrap_or(0);
        ld.taddr_support = true;
        ld.chain_name = self
            .data
            .read()
            .await
            .config
            .chain_name
            .clone();
        ld.sapling_activation_height = self
            .data
            .read()
            .await
            .config
            .sapling_activation_height;

        Ok(Response::new(ld))
    }

    async fn ping(
        &self,
        _request: Request<Duration>,
    ) -> Result<Response<PingResponse>, Status> {
        todo!()
    }

    type GetMempoolStreamStream = Pin<Box<dyn Stream<Item = Result<RawTransaction, Status>> + Send + Sync>>;

    async fn get_mempool_stream(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::GetMempoolStreamStream>, tonic::Status> {
        todo!()
    }
}
