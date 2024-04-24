use zcash_primitives::consensus;

use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct LastTxIdCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for LastTxIdCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Show the latest TxId in the wallet");
        h.push("Usage:");
        h.push("lasttxid");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Show the latest TxId in the wallet".to_string()
    }
    fn exec(
        &self,
        _args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        RT.block_on(async move {
            format!(
                "{}",
                lightclient
                    .do_last_txid()
                    .await
                    .pretty(2)
            )
        })
    }
}
