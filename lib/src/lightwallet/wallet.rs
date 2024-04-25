use std::collections::{BTreeMap, HashMap};
use std::convert::TryInto;
use std::future::Future;
use std::io::{Error, ErrorKind, Read, Write};
use std::sync::atomic::AtomicU64;
use std::sync::{mpsc, Arc};
use std::{cmp, io};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use incrementalmerkletree::bridgetree::{BridgeTree, Checkpoint};
use incrementalmerkletree::{Hashable, Position, Tree};
use log::{error, info, warn};
use orchard::tree::{MerkleHashOrchard, MerklePath};
use orchard::{Address, Anchor};
use tokio::sync::RwLock;
use zcash_address::unified::{Address as UnifiedAddress, Encoding, Receiver};
use zcash_client_backend::address;
use zcash_client_backend::encoding::{
    decode_extended_full_viewing_key, decode_extended_spending_key, encode_payment_address,
};
use zcash_encoding::{Optional, Vector};
use zcash_primitives::consensus;
use zcash_primitives::consensus::BlockHeight;
use zcash_primitives::legacy::Script;
use zcash_primitives::memo::{Memo, MemoBytes};
use zcash_primitives::merkle_tree::incremental::{
    read_bridge, read_leu64_usize, read_position, write_bridge, write_position, write_usize_leu64,
};
use zcash_primitives::merkle_tree::HashSer;
use zcash_primitives::sapling::prover::TxProver;
use zcash_primitives::transaction::builder::Builder;
use zcash_primitives::transaction::components::amount::DEFAULT_FEE;
use zcash_primitives::transaction::components::{Amount, OutPoint, TxOut};
use zcash_primitives::zip32::ExtendedFullViewingKey;

use crate::grpc::TreeState;
use crate::lightclient::blaze::fetch_full_tx::FetchFullTxns;
use crate::lightclient::config::LightClientConfig;
use crate::lightclient::MERKLE_DEPTH;
use crate::lightwallet::data::blockdata::BlockData;
use crate::lightwallet::data::message::Message;
use crate::lightwallet::data::notes::{SaplingNoteData, SpendableOrchardNote, SpendableSaplingNote};
use crate::lightwallet::data::options::{MemoDownloadOption, WalletOptions};
use crate::lightwallet::data::price::WalletZecPriceInfo;
use crate::lightwallet::data::utxo::Utxo;
use crate::lightwallet::data::wallettx::WalletTx;
use crate::lightwallet::data::wallettxs::WalletTxs;
use crate::lightwallet::keys::data::tkey::WalletTKey;
use crate::lightwallet::keys::data::zkey::{WalletZKey, WalletZKeyType};
use crate::lightwallet::keys::keystores::Keystores;
use crate::lightwallet::keys::InMemoryKeys;
use crate::lightwallet::send_progress::SendProgress;
use crate::lightwallet::utils;

pub struct LightWallet<P> {
    // All the keys in the wallet
    // todo: rename to keyring
    keystores: Arc<RwLock<Keystores<P>>>,

    // The block at which this wallet was born. Rescans will start from here.
    birthday: AtomicU64,

    // Progress of an outgoing tx
    send_progress: Arc<RwLock<SendProgress>>,

    // The last 100 blocks, used if something gets re-organized
    pub(crate) blocks: Arc<RwLock<Vec<BlockData>>>,

    // List of all txns
    pub(crate) txs: Arc<RwLock<WalletTxs>>,

    // Wallet options
    pub(crate) options: Arc<RwLock<WalletOptions>>,

    // Non-serialized fields
    pub(crate) config: LightClientConfig<P>,

    // Highest verified block
    pub(crate) verified_tree: Arc<RwLock<Option<TreeState>>>,

    // The Orchard incremental tree
    pub(crate) orchard_witnesses: Arc<RwLock<Option<BridgeTree<MerkleHashOrchard, MERKLE_DEPTH>>>>,

    // The current price of ZEC. (time_fetched, price in USD)
    pub price_info: Arc<RwLock<WalletZecPriceInfo>>,
}

impl<P: consensus::Parameters + Send + Sync + 'static> LightWallet<P> {
    pub fn serialized_version() -> u64 {
        25
    }

    pub fn new(
        config: LightClientConfig<P>,
        seed_phrase: Option<String>,
        height: u64,
        num_zaddrs: u32,
        num_oaddrs: u32,
    ) -> io::Result<Self> {
        let keys = InMemoryKeys::<P>::new(&config, seed_phrase, num_zaddrs, num_oaddrs)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

        Ok(Self {
            keystores: Arc::new(RwLock::new(keys.into())),
            txs: Arc::new(RwLock::new(WalletTxs::new())),
            blocks: Arc::new(RwLock::new(vec![])),
            options: Arc::new(RwLock::new(WalletOptions::default())),
            config,
            orchard_witnesses: Arc::new(RwLock::new(None)),
            birthday: AtomicU64::new(height),
            verified_tree: Arc::new(RwLock::new(None)),
            send_progress: Arc::new(RwLock::new(SendProgress::new(0))),
            price_info: Arc::new(RwLock::new(WalletZecPriceInfo::new())),
        })
    }

    pub fn with_keystore(
        config: LightClientConfig<P>,
        height: u64,
        keystore: impl Into<Keystores<P>>,
    ) -> Self {
        Self {
            keystores: Arc::new(RwLock::new(keystore.into())),
            txs: Default::default(),
            blocks: Default::default(),
            options: Default::default(),
            config,
            orchard_witnesses: Arc::new(RwLock::new(None)),
            birthday: AtomicU64::new(height),
            verified_tree: Default::default(),
            send_progress: Arc::new(RwLock::new(SendProgress::new(0))),
            price_info: Default::default(),
        }
    }

    pub fn read_tree<H: Hashable + HashSer + Ord + Clone, R: Read>(mut reader: R) -> io::Result<BridgeTree<H, 32>> {
        let _version = reader.read_u64::<LittleEndian>()?;

        let prior_bridges = Vector::read(&mut reader, |r| read_bridge(r))?;
        let current_bridge = Optional::read(&mut reader, |r| read_bridge(r))?;
        let saved: BTreeMap<Position, usize> =
            Vector::read_collected(&mut reader, |mut r| Ok((read_position(&mut r)?, read_leu64_usize(&mut r)?)))?;

        let checkpoints = Vector::read_collected(&mut reader, |r| Self::read_checkpoint_v2(r))?;
        let max_checkpoints = read_leu64_usize(&mut reader)?;

        BridgeTree::from_parts(prior_bridges, current_bridge, saved, checkpoints, max_checkpoints).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Consistency violation found when attempting to deserialize Merkle tree: {:?}", err),
            )
        })
    }

    fn write_tree<H: Hashable + HashSer + Ord, W: Write>(
        mut writer: W,
        tree: &BridgeTree<H, MERKLE_DEPTH>,
    ) -> io::Result<()> {
        writer.write_u64::<LittleEndian>(Self::serialized_version())?;

        Vector::write(&mut writer, tree.prior_bridges(), |mut w, b| write_bridge(w, b))?;
        Optional::write(&mut writer, tree.current_bridge().as_ref(), |mut w, b| write_bridge(w, b))?;
        Vector::write_sized(&mut writer, tree.witnessed_indices().iter(), |mut w, (pos, i)| {
            write_position(&mut w, *pos)?;
            write_usize_leu64(&mut w, *i)
        })?;
        Vector::write(&mut writer, tree.checkpoints(), |w, c| Self::write_checkpoint_v2(w, c))?;
        write_usize_leu64(&mut writer, tree.max_checkpoints())?;

        Ok(())
    }

    pub fn write_checkpoint_v2<W: Write>(
        mut writer: W,
        checkpoint: &Checkpoint,
    ) -> io::Result<()> {
        write_usize_leu64(&mut writer, checkpoint.bridges_len())?;
        writer.write_u8(if checkpoint.is_witnessed() { 1 } else { 0 })?;
        Vector::write_sized(&mut writer, checkpoint.witnessed().iter(), |w, p| write_position(w, *p))?;
        Vector::write_sized(&mut writer, checkpoint.forgotten().iter(), |mut w, (pos, idx)| {
            write_position(&mut w, *pos)?;
            write_usize_leu64(&mut w, *idx)
        })?;

        Ok(())
    }

    pub fn read_checkpoint_v2<R: Read>(mut reader: R) -> io::Result<Checkpoint> {
        Ok(Checkpoint::from_parts(
            read_leu64_usize(&mut reader)?,
            reader.read_u8()? == 1,
            Vector::read_collected(&mut reader, |r| read_position(r))?,
            Vector::read_collected(&mut reader, |mut r| Ok((read_position(&mut r)?, read_leu64_usize(&mut r)?)))?,
        ))
    }

    pub async fn read<R: Read>(
        mut reader: R,
        config: &LightClientConfig<P>,
    ) -> io::Result<Self> {
        let version = reader.read_u64::<LittleEndian>()?;
        if version > Self::serialized_version() {
            let e = format!("Don't know how to read wallet version {}. Do you have the latest version?", version);
            error!("{}", e);
            return Err(io::Error::new(ErrorKind::InvalidData, e));
        }

        info!("Reading wallet version {}", version);

        // FIXME: this is not abstracted away correctly
        let keys = if version <= 14 {
            InMemoryKeys::<P>::read_old(version, &mut reader, config).map(Into::into)
        } else if version <= 24 {
            InMemoryKeys::<P>::read(&mut reader, config).map(Into::into)
        } else {
            Keystores::read(&mut reader, config).await
        }?;

        let mut blocks = Vector::read(&mut reader, |r| BlockData::read(r))?;
        if version <= 14 {
            // Reverse the order, since after version 20, we need highest-block-first
            blocks = blocks.into_iter().rev().collect();
        }

        let mut txns = if version <= 14 { WalletTxs::read_old(&mut reader) } else { WalletTxs::read(&mut reader) }?;

        let chain_name = utils::read_string(&mut reader)?;

        if chain_name != config.chain_name {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Wallet chain name {} doesn't match expected {}", chain_name, config.chain_name),
            ));
        }

        let wallet_options = if version <= 23 { WalletOptions::default() } else { WalletOptions::read(&mut reader)? };

        let birthday = reader.read_u64::<LittleEndian>()?;

        if version <= 22 {
            let _sapling_tree_verified = if version <= 12 { true } else { reader.read_u8()? == 1 };
        }

        let verified_tree = if version <= 21 {
            None
        } else {
            Optional::read(&mut reader, |r| {
                use prost::Message;

                let buf = Vector::read(r, |r| r.read_u8())?;
                TreeState::decode(&buf[..])
                    .map_err(|e| io::Error::new(ErrorKind::InvalidData, format!("Read Error: {}", e)))
            })?
        };

        // If version <= 8, adjust the "is_spendable" status of each note data
        if version <= 8 {
            // Collect all spendable keys
            let spendable_keys: Vec<_> = keys
                .get_all_extfvks()
                .await
                .filter(|extfvk| futures::executor::block_on(keys.have_sapling_spending_key(extfvk)))
                .collect();

            txns.adjust_spendable_status(spendable_keys);
        }
        let price = if version <= 13 { WalletZecPriceInfo::new() } else { WalletZecPriceInfo::read(&mut reader)? };

        // Reach the orchard tree
        let orchard_witnesses = if version <= 24 { None } else { Optional::read(&mut reader, |r| Self::read_tree(r))? };

        let mut lw = Self {
            keystores: Arc::new(RwLock::new(keys)),
            txs: Arc::new(RwLock::new(txns)),
            blocks: Arc::new(RwLock::new(blocks)),
            config: config.clone(),
            options: Arc::new(RwLock::new(wallet_options)),
            orchard_witnesses: Arc::new(RwLock::new(orchard_witnesses)),
            birthday: AtomicU64::new(birthday),
            verified_tree: Arc::new(RwLock::new(verified_tree)),
            send_progress: Arc::new(RwLock::new(SendProgress::new(0))),
            price_info: Arc::new(RwLock::new(price)),
        };

        // For old wallets, remove unused addresses
        if version <= 14 {
            lw.remove_unused_taddrs().await;
            lw.remove_unused_zaddrs().await;
        }

        if version <= 14 {
            lw.set_witness_block_heights().await;
        }

        // Also make sure we have at least 1 unified address
        if lw
            .keys()
            .read()
            .await
            .get_orchard_fvk(0)
            .await
            .is_none()
        {
            lw.keys().write().await.add_oaddr();
        }

        Ok(lw)
    }

    pub async fn write<W: Write>(
        &self,
        mut writer: W,
    ) -> io::Result<()> {
        {
            // enclose in scope to avoid holding read lock after these checks
            let keys = self.keys().read().await;

            if !keys.writable() {
                return Err(Error::new(ErrorKind::InvalidInput, "Wallet wasn't ready to be written.".to_string()));
            }

            // Write the version
            writer.write_u64::<LittleEndian>(Self::serialized_version())?;

            // Write the keystore
            keys.write(&mut writer).await?;
        }

        Vector::write(&mut writer, &self.blocks.read().await, |w, b| b.write(w))?;

        self.txs
            .read()
            .await
            .write(&mut writer)?;

        utils::write_string(&mut writer, &self.config.chain_name)?;

        self.options
            .read()
            .await
            .write(&mut writer)?;

        // While writing the birthday, get it from the fn so we recalculate it properly
        // in case of rescans etc...
        writer.write_u64::<LittleEndian>(self.get_birthday().await)?;

        Optional::write(&mut writer, self.verified_tree.read().await.as_ref(), |w, t| {
            use prost::Message;
            let mut buf = vec![];

            t.encode(&mut buf)?;
            Vector::write(w, &buf, |w, b| w.write_u8(*b))
        })?;

        // Price info
        self.price_info
            .read()
            .await
            .write(&mut writer)?;

        // Write the Tree
        Optional::write(
            &mut writer,
            self.orchard_witnesses
                .read()
                .await
                .as_ref(),
            |w, o| Self::write_tree(w, o),
        )?;

        Ok(())
    }

    // Before version 20, witnesses didn't store their height, so we need to update
    // them.
    pub async fn set_witness_block_heights(&mut self) {
        let top_height = self.last_scanned_height().await;
        self.txs
            .write()
            .await
            .current
            .iter_mut()
            .for_each(|(_, wtx)| {
                wtx.s_notes.iter_mut().for_each(|nd| {
                    nd.witnesses.top_height = top_height;
                });
            });
    }

    pub fn keys(&self) -> &RwLock<Keystores<P>> {
        &self.keystores
    }

    pub fn keys_clone(&self) -> Arc<RwLock<Keystores<P>>> {
        self.keystores.clone()
    }

    pub fn txns(&self) -> Arc<RwLock<WalletTxs>> {
        self.txs.clone()
    }

    pub async fn set_blocks(
        &self,
        new_blocks: Vec<BlockData>,
    ) {
        let mut blocks = self.blocks.write().await;
        blocks.clear();
        blocks.extend_from_slice(&new_blocks[..]);
    }

    /// Return a copy of the blocks currently in the wallet, needed to process
    /// possible reorgs
    pub async fn get_blocks(&self) -> Vec<BlockData> {
        self.blocks
            .read()
            .await
            .iter()
            .cloned()
            .collect()
    }

    pub fn sapling_note_address(
        hrp: &str,
        note: &SaplingNoteData,
    ) -> Option<String> {
        note.extfvk
            .fvk
            .vk
            .to_payment_address(note.diversifier)
            .map(|pa| encode_payment_address(hrp, &pa))
    }

    pub fn orchard_ua_address(
        config: &LightClientConfig<P>,
        address: &Address,
    ) -> String {
        let orchard_container = Receiver::Orchard(address.to_raw_address_bytes());
        let unified_address = UnifiedAddress::try_from_items(vec![orchard_container]).unwrap();
        unified_address.encode(&config.get_network())
    }

    pub async fn set_download_memo(
        &self,
        value: MemoDownloadOption,
    ) {
        self.options
            .write()
            .await
            .download_memos = value;
    }

    pub async fn set_spam_filter_threshold(
        &self,
        value: i64,
    ) {
        self.options
            .write()
            .await
            .spam_threshold = value;
    }

    pub async fn get_birthday(&self) -> u64 {
        let birthday = self
            .birthday
            .load(std::sync::atomic::Ordering::SeqCst);
        if birthday == 0 {
            self.get_first_tx_block().await
        } else {
            cmp::min(self.get_first_tx_block().await, birthday)
        }
    }

    pub async fn set_latest_zec_price(
        &self,
        price: f64,
    ) {
        if price <= 0 as f64 {
            warn!("Tried to set a bad current zec price {}", price);
            return;
        }

        self.price_info.write().await.price = Some((utils::now(), price));
        info!("Set current ZEC Price to USD {}", price);
    }

    // Get the current sending status.
    pub async fn get_send_progress(&self) -> SendProgress {
        self.send_progress.read().await.clone()
    }

    // Set the previous send status as an error
    async fn set_send_error(
        &self,
        e: String,
    ) {
        let mut p = self.send_progress.write().await;

        p.is_send_in_progress = false;
        p.last_error = Some(e);
    }

    // Set the previous send's status as success
    async fn set_send_success(
        &self,
        txid: String,
    ) {
        let mut p = self.send_progress.write().await;

        p.is_send_in_progress = false;
        p.last_txid = Some(txid);
    }

    // Reset the send progress status to blank
    async fn reset_send_progress(&self) {
        let mut g = self.send_progress.write().await;
        let next_id = g.id + 1;

        // Discard the old value, since we are replacing it
        let _ = std::mem::replace(&mut *g, SendProgress::new(next_id));
    }

    pub async fn is_unlocked_for_spending(&self) -> bool {
        match self.in_memory_keys().await {
            Ok(ks) => ks.is_unlocked_for_spending(),
            // for now if it's not in-memory just assume it's unlocked
            // TODO: do appropriate work here for other keystores
            _ => true,
        }
    }

    pub async fn is_encrypted(&self) -> bool {
        match self.in_memory_keys().await {
            Ok(ks) => ks.is_encrypted(),
            // for now if it's not in-memory just assume it's unlocked
            // TODO: do appropriate work here for other keystores
            _ => false,
        }
    }

    // Get the first block that this wallet has a tx in. This is often used as the
    // wallet's "birthday" If there are no Txns, then the actual birthday (which
    // is recorder at wallet creation) is returned If no birthday was recorded,
    // return the sapling activation height
    pub async fn get_first_tx_block(&self) -> u64 {
        // Find the first transaction
        let earliest_block = self
            .txs
            .read()
            .await
            .current
            .values()
            .map(|wtx| u64::from(wtx.block))
            .min();

        let birthday = self
            .birthday
            .load(std::sync::atomic::Ordering::SeqCst);
        earliest_block // Returns optional, so if there's no txns, it'll get the activation height
            .unwrap_or(cmp::max(birthday, self.config.sapling_activation_height))
    }

    fn adjust_wallet_birthday(
        &self,
        new_birthday: u64,
    ) {
        let mut wallet_birthday = self
            .birthday
            .load(std::sync::atomic::Ordering::SeqCst);
        if new_birthday < wallet_birthday {
            wallet_birthday = cmp::max(new_birthday, self.config.sapling_activation_height);
            self.birthday
                .store(wallet_birthday, std::sync::atomic::Ordering::SeqCst);
        }
    }

    pub async fn add_imported_tk(
        &self,
        sk: String,
    ) -> String {
        let sk = match WalletTKey::from_sk_string(&self.config, sk) {
            Err(e) => return format!("Error: {}", e),
            Ok(k) => k,
        };

        let address = sk.address.clone();

        let mut keys = match self.in_memory_keys_mut().await {
            Ok(k) => k,
            Err(e) => return format!("Error: {}", e),
        };

        if keys.encrypted {
            return "Error: Can't import transparent address key while wallet is encrypted".to_string();
        }

        if keys
            .tkeys
            .iter()
            .any(|tk| tk.address == address)
        {
            return "Error: Key already exists".to_string();
        }

        keys.tkeys.push(sk);
        address
    }

    // Add a new imported spending key to the wallet
    /// NOTE: This will not rescan the wallet
    pub async fn add_imported_sk(
        &self,
        sk: String,
        birthday: u64,
    ) -> String {
        // we don't need to acquire write access immediately
        // but in the general case we do want write access
        let mut keys = match self.in_memory_keys_mut().await {
            Ok(k) => k,
            Err(e) => return format!("Error: {}", e),
        };

        if keys.encrypted {
            return "Error: Can't import spending key while wallet is encrypted".to_string();
        }

        // First, try to interpret the key
        let extsk = match decode_extended_spending_key(self.config.hrp_sapling_private_key(), &sk) {
            Ok(Some(k)) => k,
            Ok(None) => return "Error: Couldn't decode spending key".to_string(),
            Err(e) => return format!("Error importing spending key: {}", e),
        };

        // Make sure the key doesn't already exist
        if keys
            .zkeys
            .iter()
            .any(|wk| wk.extsk.is_some() && wk.extsk.as_ref().unwrap() == &extsk.clone())
        {
            return "Error: Key already exists".to_string();
        }

        let extfvk = ExtendedFullViewingKey::from(&extsk);
        let zaddress = {
            let zkeys = &mut keys.zkeys;
            let maybe_existing_zkey = zkeys
                .iter_mut()
                .find(|wk| wk.extfvk == extfvk);

            // If the viewing key exists, and is now being upgraded to the spending key,
            // replace it in-place
            if maybe_existing_zkey.is_some() {
                let existing_zkey = maybe_existing_zkey.unwrap();
                existing_zkey.extsk = Some(extsk);
                existing_zkey.keytype = WalletZKeyType::ImportedSpendingKey;
                existing_zkey.zaddress.clone()
            } else {
                let newkey = WalletZKey::new_imported_sk(extsk);
                zkeys.push(newkey.clone());
                newkey.zaddress
            }
        };

        // Adjust wallet birthday
        self.adjust_wallet_birthday(birthday);

        encode_payment_address(self.config.hrp_sapling_address(), &zaddress)
    }

    // Add a new imported viewing key to the wallet
    /// NOTE: This will not rescan the wallet
    pub async fn add_imported_vk(
        &self,
        vk: String,
        birthday: u64,
    ) -> String {
        // we don't need to acquire write access immediately
        // but in the general case we do want write access
        let mut keys = match self.in_memory_keys_mut().await {
            Ok(k) => k,
            Err(e) => return format!("Error: {}", e),
        };

        if !keys.unlocked {
            return "Error: Can't add key while wallet is locked".to_string();
        }

        // First, try to interpret the key
        let extfvk = match decode_extended_full_viewing_key(self.config.hrp_sapling_viewing_key(), &vk) {
            Ok(Some(k)) => k,
            Ok(None) => return "Error: Couldn't decode viewing key".to_string(),
            Err(e) => return format!("Error importing viewing key: {}", e),
        };

        // Make sure the key doesn't already exist
        if keys
            .zkeys
            .iter()
            .any(|wk| wk.extfvk == extfvk.clone())
        {
            return "Error: Key already exists".to_string();
        }

        let newkey = WalletZKey::new_imported_viewkey(extfvk);
        keys.zkeys.push(newkey.clone());

        // Adjust wallet birthday
        self.adjust_wallet_birthday(birthday);

        encode_payment_address(self.config.hrp_sapling_address(), &newkey.zaddress)
    }

    /// Clears all the downloaded blocks and resets the state back to the
    /// initial block. After this, the wallet's initial state will need to
    /// be set and the wallet will need to be rescanned
    pub async fn clear_all(&self) {
        self.blocks.write().await.clear();
        self.txs.write().await.clear();
        self.verified_tree.write().await.take();
        self.orchard_witnesses
            .write()
            .await
            .take();
    }

    pub async fn set_initial_block(
        &self,
        height: u64,
        hash: &str,
        _sapling_tree: &str,
    ) -> bool {
        let mut blocks = self.blocks.write().await;
        if !blocks.is_empty() {
            return false;
        }

        blocks.push(BlockData::new_with(height, hash));

        true
    }

    /// Clears all the downloaded blocks and resets the state to the specified
    /// block.
    pub async fn clear_all_and_set_initial_block(
        &self,
        height: u64,
        hash: &str,
        _tree: &str,
    ) {
        let mut blocks_guard = self.blocks.write().await;
        let mut txns_guard = self.txs.write().await;

        blocks_guard.clear();
        txns_guard.clear();

        blocks_guard.push(BlockData::new_with(height, hash));
    }

    pub async fn last_scanned_height(&self) -> u64 {
        self.blocks
            .read()
            .await
            .first()
            .map(|block| block.height)
            .unwrap_or(self.config.sapling_activation_height - 1)
    }

    pub async fn last_scanned_hash(&self) -> String {
        self.blocks
            .read()
            .await
            .first()
            .map(|block| block.hash())
            .unwrap_or_default()
    }

    async fn get_target_height(&self) -> Option<u32> {
        self.blocks
            .read()
            .await
            .first()
            .map(|block| block.height as u32 + 1)
    }

    /// Determines the target height for a transaction, and the offset from
    /// which to select anchors, based on the current synchronised block
    /// chain.
    async fn get_target_height_and_anchor_offset(&self) -> Option<(u32, usize)> {
        let res = {
            let blocks = self.blocks.read().await;
            (
                blocks
                    .last()
                    .map(|block| block.height as u32),
                blocks
                    .first()
                    .map(|block| block.height as u32),
            )
        };
        match res {
            (Some(min_height), Some(max_height)) => {
                let target_height = max_height + 1;

                // Select an anchor ANCHOR_OFFSET back from the target block,
                // unless that would be before the earliest block we have.
                let anchor_height = cmp::max(target_height.saturating_sub(self.config.anchor_offset), min_height);

                Some((target_height, (target_height - anchor_height) as usize))
            },
            _ => None,
        }
    }

    /// Get the height of the anchor block
    pub async fn get_anchor_height(&self) -> u32 {
        match self
            .get_target_height_and_anchor_offset()
            .await
        {
            Some((height, anchor_offset)) => height - anchor_offset as u32 - 1,
            None => 0,
        }
    }

    pub fn memo_str(memo: Option<Memo>) -> Option<String> {
        match memo {
            Some(Memo::Text(m)) => Some(m.to_string()),
            _ => None,
        }
    }

    pub async fn uabalance(
        &self,
        addr: Option<String>,
    ) -> u64 {
        self.txs
            .read()
            .await
            .current
            .values()
            .map(|tx| {
                tx.o_notes
                    .iter()
                    .filter(|nd| match addr.as_ref() {
                        Some(a) => *a == LightWallet::<P>::orchard_ua_address(&self.config, &nd.note.recipient()),
                        None => true,
                    })
                    .map(
                        |nd| {
                            if nd.spent.is_none() && nd.unconfirmed_spent.is_none() {
                                nd.note.value().inner()
                            } else {
                                0
                            }
                        },
                    )
                    .sum::<u64>()
            })
            .sum::<u64>()
    }

    pub async fn zbalance(
        &self,
        addr: Option<String>,
    ) -> u64 {
        self.txs
            .read()
            .await
            .current
            .values()
            .map(|tx| {
                tx.s_notes
                    .iter()
                    .filter(|nd| match addr.as_ref() {
                        Some(a) => {
                            *a == encode_payment_address(
                                self.config.hrp_sapling_address(),
                                &nd.extfvk
                                    .fvk
                                    .vk
                                    .to_payment_address(nd.diversifier)
                                    .unwrap(),
                            )
                        },
                        None => true,
                    })
                    .map(|nd| if nd.spent.is_none() && nd.unconfirmed_spent.is_none() { nd.note.value().inner() } else { 0 })
                    .sum::<u64>()
            })
            .sum::<u64>()
    }

    // Get all (unspent) utxos. Unconfirmed spent utxos are included
    pub async fn get_utxos(&self) -> Vec<Utxo> {
        self.txs
            .read()
            .await
            .current
            .values()
            .flat_map(|tx| {
                tx.utxos
                    .iter()
                    .filter(|utxo| utxo.spent.is_none())
            })
            .cloned()
            .collect::<Vec<Utxo>>()
    }

    pub async fn tbalance(
        &self,
        addr: Option<String>,
    ) -> u64 {
        self.get_utxos()
            .await
            .iter()
            .filter(|utxo| match addr.as_ref() {
                Some(a) => utxo.address == *a,
                None => true,
            })
            .map(|utxo| utxo.value)
            .sum::<u64>()
    }

    pub async fn unverified_zbalance(
        &self,
        addr: Option<String>,
    ) -> u64 {
        let anchor_height = self.get_anchor_height().await;

        // TODO: allow any keystore (see usage)
        let keys = self.keys().read().await;

        let txns = self.txs.read().await;
        let txns = txns.current.values();

        let mut sum = 0;
        for tx in txns {
            for nd in tx
                .s_notes
                .iter()
                .filter(|nd| nd.spent.is_none() && nd.unconfirmed_spent.is_none())
                .filter(|nd| match addr.clone() {
                    Some(a) => {
                        a == encode_payment_address(
                            self.config.hrp_sapling_address(),
                            &nd.extfvk
                                .fvk
                                .vk
                                .to_payment_address(nd.diversifier)
                                .unwrap(),
                        )
                    },
                    None => true,
                })
            {
                // Check to see if we have this note's spending key.
                if keys
                    .have_sapling_spending_key(&nd.extfvk)
                    .await
                    && tx.block > BlockHeight::from_u32(anchor_height)
                {
                    // If confirmed but dont have anchor yet, it is unconfirmed
                    sum += nd.note.value
                }
            }
        }

        sum
    }

    pub async fn verified_zbalance(
        &self,
        addr: Option<String>,
    ) -> u64 {
        let anchor_height = self.get_anchor_height().await;

        self.txs
            .read()
            .await
            .current
            .values()
            .map(|tx| {
                if tx.block <= BlockHeight::from_u32(anchor_height) {
                    tx.s_notes
                        .iter()
                        .filter(|nd| nd.spent.is_none() && nd.unconfirmed_spent.is_none())
                        .filter(|nd| match addr.as_ref() {
                            Some(a) => {
                                *a == encode_payment_address(
                                    self.config.hrp_sapling_address(),
                                    &nd.extfvk
                                        .fvk
                                        .vk
                                        .to_payment_address(nd.diversifier)
                                        .unwrap(),
                                )
                            },
                            None => true,
                        })
                        .map(|nd| nd.note.value)
                        .sum::<u64>()
                } else {
                    0
                }
            })
            .sum::<u64>()
    }

    pub async fn spendable_zbalance(
        &self,
        addr: Option<String>,
    ) -> u64 {
        let anchor_height = self.get_anchor_height().await;

        // TODO: allow any keystore (see usage)
        let keys = self.keys().read().await;

        let mut sum = 0;
        let txns = self.txs.read().await;
        let txns = txns.current.values();

        for tx in txns {
            if tx.block <= BlockHeight::from_u32(anchor_height) {
                for nd in tx
                    .s_notes
                    .iter()
                    .filter(|nd| nd.spent.is_none() && nd.unconfirmed_spent.is_none())
                    .filter(|nd| match addr.as_ref() {
                        Some(a) => {
                            *a == encode_payment_address(
                                self.config.hrp_sapling_address(),
                                &nd.extfvk
                                    .fvk
                                    .vk
                                    .to_payment_address(nd.diversifier)
                                    .unwrap(),
                            )
                        },
                        None => true,
                    })
                {
                    // Check to see if we have this note's spending key and witnesses
                    if keys
                        .have_sapling_spending_key(&nd.extfvk)
                        .await
                        && !nd.witnesses.is_empty()
                    {
                        sum += nd.note.value;
                    }
                }
            }
        }

        sum
    }

    pub async fn remove_unused_taddrs(&self) {
        // TODO: allow any keystore (see usage)
        let mut keys = self
            .in_memory_keys_mut()
            .await
            .expect("in memory keystore");

        let taddrs = keys.get_all_taddrs();
        if taddrs.len() <= 1 {
            return;
        }

        let highest_account = self
            .txs
            .read()
            .await
            .current
            .values()
            .flat_map(|wtx| {
                wtx.utxos.iter().map(|u| {
                    taddrs
                        .iter()
                        .position(|taddr| *taddr == u.address)
                        .unwrap_or(taddrs.len())
                })
            })
            .max();

        if highest_account.is_none() {
            return;
        }

        if highest_account.unwrap() == 0 {
            // Remove unused addresses
            keys.tkeys.truncate(1);
        }
    }

    pub async fn remove_unused_zaddrs(&self) {
        // TODO: allow any keystore (see usage)
        let mut keys = self
            .in_memory_keys_mut()
            .await
            .expect("in memory keystore");

        let zaddrs: Vec<String> = self
            .keystores
            .read()
            .await
            .get_all_zaddresses()
            .await
            .collect();

        if zaddrs.len() <= 1 {
            return;
        }

        let highest_account = self
            .txs
            .read()
            .await
            .current
            .values()
            .flat_map(|wtx| {
                wtx.s_notes.iter().map(|n| {
                    let (_, pa) = n.extfvk.default_address();
                    let zaddr = encode_payment_address(self.config.hrp_sapling_address(), &pa);
                    zaddrs
                        .iter()
                        .position(|za| *za == zaddr)
                        .unwrap_or(zaddrs.len())
                })
            })
            .max();

        if highest_account.is_none() {
            return;
        }

        if highest_account.unwrap() == 0 {
            // Remove unused addresses
            keys.zkeys.truncate(1);
        }
    }

    pub async fn decrypt_message(
        &self,
        enc: Vec<u8>,
    ) -> Option<Message> {
        // Collect all the ivks in the wallet
        let ivks: Vec<_> = self
            .keystores
            .read()
            .await
            .get_all_extfvks()
            .await
            .map(|extfvk| extfvk.fvk.vk.ivk())
            .collect();

        // Attempt decryption with all available ivks, one at a time. This is pretty
        // fast, so no need for fancy multithreading
        for ivk in ivks {
            if let Ok(msg) = Message::decrypt(&enc, &ivk) {
                // If decryption succeeded for this IVK, return the decrypted memo and the
                // matched address
                return Some(msg);
            }
        }

        // If nothing matched
        None
    }

    // Add the spent_at_height for each sapling note that has been spent. This field
    // was added in wallet version 8, so for older wallets, it will need to be
    // added
    pub async fn fix_spent_at_height(&self) {
        // First, build an index of all the txids and the heights at which they were
        // spent.
        let spent_txid_map: HashMap<_, _> = self
            .txs
            .read()
            .await
            .current
            .iter()
            .map(|(txid, wtx)| (*txid, wtx.block))
            .collect();

        // Go over all the sapling notes that might need updating
        self.txs
            .write()
            .await
            .current
            .values_mut()
            .for_each(|wtx| {
                wtx.s_notes
                    .iter_mut()
                    .filter(|nd| nd.spent.is_some() && nd.spent.unwrap().1 == 0)
                    .for_each(|nd| {
                        let txid = nd.spent.unwrap().0;
                        if let Some(height) = spent_txid_map.get(&txid).copied() {
                            nd.spent = Some((txid, height.into()));
                        }
                    })
            });

        // Go over all the Utxos that might need updating
        self.txs
            .write()
            .await
            .current
            .values_mut()
            .for_each(|wtx| {
                wtx.utxos
                    .iter_mut()
                    .filter(|utxo| utxo.spent.is_some() && utxo.spent_at_height.is_none())
                    .for_each(|utxo| {
                        utxo.spent_at_height = spent_txid_map
                            .get(&utxo.spent.unwrap())
                            .map(|b| u32::from(*b) as i32);
                    })
            });
    }

    async fn select_orchard_notes(
        &self,
        target_amount: Amount,
    ) -> Vec<SpendableOrchardNote> {
        let keys = self.keystores.read().await;
        let owt = self.orchard_witnesses.read().await;
        let orchard_witness_tree = owt.as_ref().unwrap();

        let mut candidate_notes = self
            .txs
            .read()
            .await
            .current
            .iter()
            .flat_map(|(txid, tx)| {
                tx.o_notes
                    .iter()
                    .map(move |note| (*txid, note))
            })
            .filter(|(_, note)| note.note.value().inner() > 0)
            .filter_map(|(txid, note)| {
                // Filter out notes that are already spent
                if note.spent.is_some() || note.unconfirmed_spent.is_some() {
                    None
                } else {
                    // Get the spending key for the selected fvk, if we have it
                    let maybe_sk = keys.get_orchard_sk_for_fvk(&note.fvk);
                    if maybe_sk.is_none() || note.witness_position.is_none() {
                        None
                    } else {
                        let auth_path = orchard_witness_tree.authentication_path(
                            note.witness_position.unwrap(),
                            &orchard_witness_tree
                                .root(self.config.anchor_offset as usize)
                                .unwrap(),
                        );

                        if auth_path.is_none() {
                            None
                        } else {
                            let merkle_path = MerklePath::from_parts(
                                usize::from(note.witness_position.unwrap()) as u32,
                                auth_path.unwrap().try_into().unwrap(),
                            );

                            Some(SpendableOrchardNote { txid, sk: maybe_sk.unwrap(), note: note.note, merkle_path })
                        }
                    }
                }
            })
            .collect::<Vec<_>>();
        candidate_notes.sort_by(|a, b| {
            b.note
                .value()
                .inner()
                .cmp(&a.note.value().inner())
        });

        // Select the minimum number of notes required to satisfy the target value
        let o_notes = candidate_notes
            .into_iter()
            .scan(Amount::zero(), |running_total, spendable| {
                if *running_total >= target_amount {
                    None
                } else {
                    *running_total += Amount::from_u64(spendable.note.value().inner()).unwrap();
                    Some(spendable)
                }
            })
            .collect::<Vec<_>>();

        o_notes
    }

    async fn select_sapling_notes(
        &self,
        target_amount: Amount,
    ) -> Vec<SpendableSaplingNote> {
        let keys = self.keystores.read().await;

        let mut candidate_notes = self
            .txs
            .read()
            .await
            .current
            .iter()
            .flat_map(|(txid, tx)| {
                tx.s_notes
                    .iter()
                    .map(move |note| (*txid, note))
            })
            .filter(|(_, note)| note.note.value > 0)
            .filter_map(|(txid, note)| {
                // Filter out notes that are already spent
                if note.spent.is_some() || note.unconfirmed_spent.is_some() {
                    None
                } else {
                    // Get the spending key for the selected fvk, if we have it
                    futures::executor::block_on(keys.get_extsk_for_extfvk(&note.extfvk))
                        .next()
                        .and_then(|extsk| {
                            SpendableSaplingNote::from(txid, note, self.config.anchor_offset as usize, &Some(extsk))
                        })
                }
            })
            .collect::<Vec<_>>();

        candidate_notes.sort_by(|a, b| b.note.value.cmp(&a.note.value));

        // Select the minimum number of notes required to satisfy the target value
        let s_notes = candidate_notes
            .into_iter()
            .scan(Amount::zero(), |running_total, spendable| {
                if *running_total >= target_amount {
                    None
                } else {
                    *running_total += Amount::from_u64(spendable.note.value).unwrap();
                    Some(spendable)
                }
            })
            .collect::<Vec<_>>();

        let sapling_value_selected = s_notes
            .iter()
            .fold(Amount::zero(), |prev, sn| (prev + Amount::from_u64(sn.note.value).unwrap()).unwrap());

        if sapling_value_selected >= target_amount {
            return s_notes;
        }

        // If we couldn't select enough, return whatever we selected
        s_notes
    }

    // noinspection RsExternalLinter
    // noinspection RsExternalLinter
    // noinspection RsExternalLinter
    pub(crate) async fn select_notes_and_utxos(
        &self,
        target_amount: Amount,
        transparent_only: bool,
        prefer_orchard: bool,
    ) -> (Vec<SpendableOrchardNote>, Vec<SpendableSaplingNote>, Vec<Utxo>, Amount) {
        // First, we pick all the transparent values, which allows the auto shielding
        let utxos = self
            .get_utxos()
            .await
            .iter()
            .filter(|utxo| utxo.unconfirmed_spent.is_none() && utxo.spent.is_none())
            .cloned()
            .collect::<Vec<_>>();

        // Check how much we've selected
        let transparent_value_selected = utxos
            .iter()
            .fold(Amount::zero(), |prev, utxo| (prev + Amount::from_u64(utxo.value).unwrap()).unwrap());

        // If we are allowed only transparent funds or we've selected enough then return
        if transparent_only || transparent_value_selected >= target_amount {
            return (vec![], vec![], utxos, transparent_value_selected);
        }

        let mut orchard_value_selected = Amount::zero();
        let mut sapling_value_selected = Amount::zero();

        let mut o_notes = vec![];
        let mut s_notes = vec![];

        let mut remaining_amount = target_amount - transparent_value_selected;

        if prefer_orchard {
            todo!("Not implemented")
        } else {
            // Collect sapling notes first
            s_notes = self
                .select_sapling_notes(remaining_amount.unwrap())
                .await;
            sapling_value_selected = s_notes
                .iter()
                .fold(Amount::zero(), |prev, sn| (prev + Amount::from_u64(sn.note.value).unwrap()).unwrap());

            // If we've selected enough, just return
            let selected_value = (sapling_value_selected + transparent_value_selected).unwrap();
            if selected_value > target_amount {
                return (vec![], s_notes, utxos, selected_value);
            }
        }

        // If we still don't have enough, then select across the other pool
        remaining_amount =
            target_amount - (transparent_value_selected + orchard_value_selected + sapling_value_selected).unwrap();

        if prefer_orchard {
            // Select sapling notes
            s_notes = self
                .select_sapling_notes(remaining_amount.unwrap())
                .await;
            sapling_value_selected = s_notes
                .iter()
                .fold(Amount::zero(), |prev, sn| (prev + Amount::from_u64(sn.note.value).unwrap()).unwrap());
        } else {
            todo!("Not implemented")
        }

        // Return whatever we have selected, even if it is not enough, so the caller can
        // display a proper error
        let total_value_selected =
            (orchard_value_selected + sapling_value_selected + transparent_value_selected).unwrap();

        (o_notes, s_notes, utxos, total_value_selected)
    }

    pub async fn send_to_address<F, Fut, PR: TxProver + Send + Sync>(
        &self,
        prover: PR,
        transparent_only: bool,
        tos: Vec<(&str, u64, Option<String>)>,
        broadcast_fn: F,
    ) -> Result<(String, Vec<u8>), String>
    where
        F: Fn(Box<[u8]>) -> Fut,
        Fut: Future<Output = Result<String, String>>,
    {
        // Reset the progress to start. Any errors will get recorded here
        self.reset_send_progress().await;

        // Call the internal function
        match self
            .send_to_address_internal(prover, transparent_only, tos, broadcast_fn)
            .await
        {
            Ok((txid, rawtx)) => {
                self.set_send_success(txid.clone())
                    .await;
                Ok((txid, rawtx))
            },
            Err(e) => {
                self.set_send_error(e.to_string()).await;
                Err(e)
            },
        }
    }

    async fn send_to_address_internal<F, Fut, PR: TxProver + Send + Sync>(
        &self,
        prover: PR,
        transparent_only: bool,
        tos: Vec<(&str, u64, Option<String>)>,
        broadcast_fn: F,
    ) -> Result<(String, Vec<u8>), String>
    where
        F: Fn(Box<[u8]>) -> Fut,
        Fut: Future<Output = Result<String, String>>,
    {
        if !self.is_unlocked_for_spending().await {
            return Err("Cannot spend while wallet is locked".to_string());
        }

        let start_time = utils::now();
        if tos.is_empty() {
            return Err("Need at least one destination address".to_string());
        }

        let total_value = tos.iter().map(|to| to.1).sum::<u64>();
        println!("0: Creating transaction sending {} ztoshis to {} addresses", total_value, tos.len());

        // Convert address (str) to RecipientAddress and value to Amount
        let recipients = tos
            .iter()
            .map(|to| {
                let ra = match address::RecipientAddress::decode(&self.config.get_params(), to.0) {
                    Some(to) => to,
                    None => {
                        let e = format!("Invalid recipient address: '{}'", to.0);
                        error!("{}", e);
                        return Err(e);
                    },
                };

                let value = Amount::from_u64(to.1).unwrap();

                Ok((ra, value, to.2.clone()))
            })
            .collect::<Result<Vec<(address::RecipientAddress, Amount, Option<String>)>, String>>()?;

        // Calculate how much we're sending to each type of address
        let (_t_out, s_out, _o_out) = recipients
            .iter()
            .map(|(to, value, _)| match to {
                address::RecipientAddress::Unified(_) => (0, 0, value.into()),
                address::RecipientAddress::Shielded(_) => (0, value.into(), 0),
                address::RecipientAddress::Transparent(_) => (value.into(), 0, 0),
            })
            .reduce(|(t, s, o), (t2, s2, o2)| (t + t2, s + s2, o + o2))
            .unwrap_or((0, 0, 0));

        // Select notes to cover the target value
        println!("{}: Selecting notes", utils::now() - start_time);

        let target_amount = (Amount::from_u64(total_value).unwrap() + DEFAULT_FEE).unwrap();
        let target_height = match self.get_target_height().await {
            Some(h) => BlockHeight::from_u32(h),
            None => return Err("No blocks in wallet to target, please sync first".to_string()),
        };

        let (progress_notifier, progress_notifier_rx) = mpsc::channel();

        let orchard_anchor = Anchor::from(
            self.orchard_witnesses
                .read()
                .await
                .as_ref()
                .unwrap()
                .root(self.config.anchor_offset as usize)
                .unwrap(),
        );

        let mut builder = Builder::new_with_orchard(self.config.get_params().clone(), target_height, orchard_anchor);
        builder.with_progress_notifier(progress_notifier);

        // Create a map from address -> sk for all taddrs, so we can spend from the
        // right address
        let (address_to_key, (first_zkey_ovk, first_zkey_addr)) = {
            let (map, first) = {
                let guard = self.keystores.read().await;
                tokio::join!(guard.get_taddr_to_key_map(), guard.first_zkey())
            };

            // create one if it doesn't exist already
            let first = match first {
                Some(first) => first,
                None => {
                    let mut guard = self.keystores.write().await;
                    guard.add_zaddr().await;
                    guard.first_zkey().await.unwrap()
                },
            };

            (map, first)
        };

        // Prefer orchard if there are no sapling outputs
        let prefer_orchard = s_out == 0;

        let (o_notes, s_notes, utxos, selected_value) = self
            .select_notes_and_utxos(target_amount, transparent_only, prefer_orchard)
            .await;
        if selected_value < target_amount {
            let e = format!(
                "Insufficient verified funds. Have {} zats, need {} zats. NOTE: funds need at least {} confirmations before they can be spent.",
                u64::from(selected_value),
                u64::from(target_amount),
                self.config.anchor_offset + 1
            );
            error!("{}", e);
            return Err(e);
        }

        // Create the transaction
        println!(
            "{}: Adding {} o_notes {} s_notes and {} utxos",
            utils::now() - start_time,
            o_notes.len(),
            s_notes.len(),
            utxos.len()
        );

        let mut change = 0u64;

        // Add all tinputs
        utxos
            .iter()
            .map(|utxo| {
                let outpoint: OutPoint = utxo.to_outpoint();

                let coin =
                    TxOut { value: Amount::from_u64(utxo.value).unwrap(), script_pubkey: Script(utxo.script.clone()) };

                match address_to_key.get(&utxo.address) {
                    Some(pk) => {
                        todo!("Implement correctly")
                        // builder
                        // .add_transparent_input(*pk, outpoint.clone(),
                        // coin.clone()) .map(|_| ())
                        // .map_err(|_|
                        // zcash_primitives::transaction::builder::Error::InvalidAmount)
                    },
                    None => {
                        // Something is very wrong
                        let e = format!("Couldn't find the key for taddr {}", utxo.address);
                        error!("{}", e);

                        Err::<(), _>(zcash_primitives::transaction::builder::Error::InvalidAmount)
                    },
                }
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("{:?}", e))?;

        // Add Orchard notes
        for selected in o_notes.iter() {
            if let Err(e) = builder.add_orchard_spend(selected.sk, selected.note, selected.merkle_path.clone()) {
                let e = format!("Error adding orchard note: {:?}", e);
                error!("{}", e);
                return Err(e);
            } else {
                change += selected.note.value().inner();
            }
        }

        // Add Sapling notes
        for selected in s_notes.iter() {
            if let Err(e) = builder.add_sapling_spend(
                selected.extsk.clone(),
                selected.diversifier,
                selected.note.clone(),
                selected.witness.path().unwrap(),
            ) {
                let e = format!("Error adding sapling note: {:?}", e);
                error!("{}", e);
                return Err(e);
            } else {
                change += selected.note.value;
            }
        }

        // If no Sapling notes were added, add the change address manually. That is,
        // send the change to our sapling address manually. Note that if a sapling note
        // was spent, the builder will automatically send change to that address
        if s_notes.is_empty() {
            builder.send_change_to(first_zkey_ovk, first_zkey_addr);
        }

        // We'll use the first ovk to encrypt outgoing Txns
        let s_ovk = first_zkey_ovk;
        let mut total_z_recepients = 0u32;
        let mut total_o_recepients = 0u32;
        for (to, value, memo) in recipients {
            // Compute memo if it exists
            let encoded_memo = match memo {
                None => MemoBytes::empty(),
                Some(s) => {
                    // If the string starts with an "0x", and contains only hex chars ([a-f0-9]+)
                    // then interpret it as a hex
                    match utils::interpret_memo_string(s) {
                        Ok(m) => m,
                        Err(e) => {
                            error!("{}", e);
                            return Err(e);
                        },
                    }
                },
            };

            println!("{}: Adding output", utils::now() - start_time);

            if let Err(e) = match to {
                address::RecipientAddress::Unified(_to) => {
                    todo!("TODO")
                },
                address::RecipientAddress::Shielded(to) => {
                    total_z_recepients += 1;
                    change -= u64::from(value);
                    builder.add_sapling_output(Some(s_ovk), to.clone(), value, encoded_memo)
                },
                address::RecipientAddress::Transparent(to) => {
                    change -= u64::from(value);
                    builder.add_transparent_output(&to, value)
                },
            } {
                let e = format!("Error adding output: {:?}", e);
                error!("{}", e);
                return Err(e);
            }
        }

        // Set up a channel to receive updates on the progress of building the
        // transaction
        let progress = self.send_progress.clone();

        // Use a separate thread to handle sending from std::mpsc to tokio::sync::mpsc
        let (tx2, mut rx2) = tokio::sync::mpsc::unbounded_channel();
        std::thread::spawn(move || {
            while let Ok(r) = progress_notifier_rx.recv() {
                tx2.send(r.cur()).unwrap();
            }
        });

        let progress_handle = tokio::spawn(async move {
            while let Some(r) = rx2.recv().await {
                println!("Progress: {}", r);
                progress.write().await.progress = r;
            }

            progress
                .write()
                .await
                .is_send_in_progress = false;
        });

        {
            // TODO(orchard): Orchard building progress
            let mut p = self.send_progress.write().await;
            p.is_send_in_progress = true;
            p.progress = 0;
            p.total = s_notes.len() as u32 + total_z_recepients + total_o_recepients;
        }

        let mut keys = self.keystores.write().await;

        println!("{}: Building transaction", utils::now() - start_time);
        let (tx, _) = match builder.build(&prover) {
            Ok(res) => {
                // stop holding a WriteGuard to the keys
                std::mem::drop(keys);
                res
            },
            Err(e) => {
                let e = format!("Error creating transaction: {:?}", e);
                error!("{}", e);
                self.send_progress
                    .write()
                    .await
                    .is_send_in_progress = false;
                return Err(e);
            },
        };

        // Wait for all the progress to be updated
        progress_handle.await.unwrap();

        println!("{}: Transaction created", utils::now() - start_time);
        println!("Transaction ID: {}", tx.txid());

        {
            self.send_progress
                .write()
                .await
                .is_send_in_progress = false;
        }

        // Create the TX bytes
        let mut raw_tx = vec![];
        tx.write(&mut raw_tx).unwrap();

        let txid = broadcast_fn(raw_tx.clone().into_boxed_slice()).await?;

        // Mark notes as spent.
        {
            // Mark sapling and orchard notes as unconfirmed spent
            let mut txs = self.txs.write().await;
            for selected in o_notes {
                let mut spent_note = txs
                    .current
                    .get_mut(&selected.txid)
                    .unwrap()
                    .o_notes
                    .iter_mut()
                    .find(|nd| {
                        nd.note.nullifier(&nd.fvk)
                            == selected
                                .note
                                .nullifier(&orchard::keys::FullViewingKey::from(&selected.sk))
                    })
                    .unwrap();
                spent_note.unconfirmed_spent = Some((tx.txid(), u32::from(target_height)));
            }

            for selected in s_notes {
                let mut spent_note = txs
                    .current
                    .get_mut(&selected.txid)
                    .unwrap()
                    .s_notes
                    .iter_mut()
                    .find(|nd| nd.nullifier == selected.nullifier)
                    .unwrap();
                spent_note.unconfirmed_spent = Some((tx.txid(), u32::from(target_height)));
            }

            // Mark this utxo as unconfirmed spent
            for utxo in utxos {
                let mut spent_utxo = txs
                    .current
                    .get_mut(&utxo.txid)
                    .unwrap()
                    .utxos
                    .iter_mut()
                    .find(|u| utxo.txid == u.txid && utxo.output_index == u.output_index)
                    .unwrap();
                spent_utxo.unconfirmed_spent = Some((tx.txid(), u32::from(target_height)));
            }
        }

        // Add this Tx to the mempool structure
        {
            let price = self.price_info.read().await.clone();

            FetchFullTxns::<P>::scan_full_tx(
                self.config.clone(),
                tx,
                target_height,
                true,
                utils::now() as u32,
                self.keystores.clone(),
                self.txs.clone(),
                WalletTx::get_price(utils::now(), &price),
            )
            .await;
        }

        Ok((txid, raw_tx))
    }

    pub async fn encrypt(
        &self,
        passwd: String,
    ) -> io::Result<()> {
        match self.in_memory_keys_mut().await {
            Ok(mut ks) => ks.encrypt(passwd),
            // for now if it's not in-memory just assume it's unlocked
            // TODO: do appropriate work here for other keystores
            _ => Ok(()),
        }
    }

    pub async fn lock(&self) -> io::Result<()> {
        match self.in_memory_keys_mut().await {
            Ok(mut ks) => ks.lock(),
            // for now if it's not in-memory just assume it's unlocked
            // TODO: do appropriate work here for other keystores
            _ => Ok(()),
        }
    }

    pub async fn unlock(
        &self,
        passwd: String,
    ) -> io::Result<()> {
        match self.in_memory_keys_mut().await {
            Ok(mut ks) => ks.unlock(passwd),
            // for now if it's not in-memory just assume it's unlocked
            // TODO: do appropriate work here for other keystores
            _ => Ok(()),
        }
    }

    pub async fn remove_encryption(
        &self,
        passwd: String,
    ) -> io::Result<()> {
        match self.in_memory_keys_mut().await {
            Ok(mut ks) => ks.remove_encryption(passwd),
            // for now if it's not in-memory just assume it's unlocked
            // TODO: do appropriate work here for other keystores
            _ => Ok(()),
        }
    }

    pub async fn in_memory_keys<'this>(
        &'this self
    ) -> Result<impl std::ops::Deref<Target = InMemoryKeys<P>> + 'this, io::Error> {
        let keys = self.keystores.read().await;
        tokio::sync::RwLockReadGuard::try_map(keys, |keys| match keys {
            Keystores::Memory(keys) => Some(keys),
            _ => None,
        })
        .map_err(|_| io::Error::new(ErrorKind::Unsupported, "incompatible keystore requested"))
    }

    pub async fn in_memory_keys_mut<'this>(
        &'this self
    ) -> Result<impl std::ops::DerefMut<Target = InMemoryKeys<P>> + 'this, io::Error> {
        let keys = self.keystores.write().await;
        tokio::sync::RwLockWriteGuard::try_map(keys, |keys| match keys {
            Keystores::Memory(keys) => Some(keys),
            _ => None,
        })
        .map_err(|_| io::Error::new(ErrorKind::Unsupported, "incompatible keystore requested"))
    }
    pub fn fee(
        &self,
        tins_n: usize,
        touts_n: usize,
        sapling_spends_n: usize,
        sapling_outputs_n: usize,
        orchard_n: usize,
    ) -> u64 {
        use std::cmp::max;

        let logical_actions = max(tins_n, touts_n) + max(sapling_spends_n, sapling_outputs_n) + orchard_n;

        const MARGINAL_FEE: u64 = 5000;
        const GRACE_ACTIONS: usize = 2;

        let actions = max(GRACE_ACTIONS, logical_actions);

        MARGINAL_FEE * (actions as u64)
    }
}
