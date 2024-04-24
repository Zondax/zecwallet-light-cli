use zcash_primitives::consensus;
use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct BalanceCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for BalanceCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Show the current ZEC balance in the wallet");
        h.push("Usage:");
        h.push("balance");
        h.push("");
        h.push("Transparent and Shielded balances, along with the addresses they belong to are displayed");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Show the current ZEC balance in the wallet".to_string()
    }
    fn exec(
        &self,
        _args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        RT.block_on(async move { format!("{}", lightclient.do_balance().await.pretty(2)) })
    }
}
