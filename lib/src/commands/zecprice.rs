use zcash_primitives::consensus;

use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct ZecPriceCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for ZecPriceCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Get the latest ZEC price in the wallet's currency (USD)");
        h.push("Usage:");
        h.push("zecprice");
        h.push("");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Get the latest ZEC price in the wallet's currency (USD)".to_string()
    }
    fn exec(
        &self,
        _args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        RT.block_on(async move { lightclient.do_zec_price().await })
    }
}
