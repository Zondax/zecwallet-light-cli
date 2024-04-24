use zcash_primitives::consensus;
use json::object;
use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct HeightCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for HeightCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Get the latest block height that the wallet is at.");
        h.push("Usage:");
        h.push("height");
        h.push("");
        h.push("Pass 'true' (default) to sync to the server to get the latest block height. Pass 'false' to get the latest height in the wallet without checking with the server.");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Get the latest block height that the wallet is at".to_string()
    }
    fn exec(
        &self,
        _args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        RT.block_on(async move {
            format!("{}", object! { "height" => lightclient.wallet.last_scanned_height().await}.pretty(2))
        })
    }
}
