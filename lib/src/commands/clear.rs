use zcash_primitives::consensus;
use json::object;
use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct ClearCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for ClearCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Clear the wallet state, rolling back the wallet to an empty state.");
        h.push("Usage:");
        h.push("clear");
        h.push("");
        h.push("This command will clear all notes, utxos and transactions from the wallet, setting up the wallet to be synced from scratch.");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Clear the wallet state, rolling back the wallet to an empty state.".to_string()
    }
    fn exec(
        &self,
        _args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        RT.block_on(async move {
            lightclient.clear_state().await;

            let result = object! { "result" => "success" };
            result.pretty(2)
        })
    }
}
