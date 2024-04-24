use json::object;
use zcash_primitives::consensus;

use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct SeedCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for SeedCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Show the wallet's seed phrase");
        h.push("Usage:");
        h.push("seed");
        h.push("");
        h.push("Your wallet is entirely recoverable from the seed phrase. Please save it carefully and don't share it with anyone");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Display the seed phrase".to_string()
    }
    fn exec(
        &self,
        _args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        RT.block_on(async move {
            match lightclient.do_seed_phrase().await {
                Ok(j) => j,
                Err(e) => object! { "error" => e },
            }
            .pretty(2)
        })
    }
}
