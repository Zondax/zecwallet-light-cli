use zcash_primitives::consensus;

use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct AddressCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for AddressCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("List current addresses in the wallet");
        h.push("Usage:");
        h.push("address");
        h.push("");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "List all addresses in the wallet".to_string()
    }
    fn exec(
        &self,
        _args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        RT.block_on(async move { format!("{}", lightclient.do_address().await.pretty(2)) })
    }
}
