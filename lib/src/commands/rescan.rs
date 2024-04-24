use zcash_primitives::consensus;
use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct RescanCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for RescanCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Rescan the wallet, rescanning all blocks for new transactions");
        h.push("Usage:");
        h.push("rescan");
        h.push("");
        h.push("This command will download all blocks since the initial block again from the light client server");
        h.push("and attempt to scan each block for transactions belonging to the wallet.");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Rescan the wallet, downloading and scanning all blocks and transactions".to_string()
    }
    fn exec(
        &self,
        _args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        RT.block_on(async move {
            match lightclient.do_rescan().await {
                Ok(j) => j.pretty(2),
                Err(e) => e,
            }
        })
    }
}
