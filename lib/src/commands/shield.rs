use json::object;
use zcash_primitives::consensus;

use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct ShieldCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for ShieldCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Shield all your transparent funds");
        h.push("Usage:");
        h.push("shield [optional address]");
        h.push("");
        h.push("NOTE: The fee required to send this transaction (currently ZEC 0.0001) is additionally deducted from your balance.");
        h.push("Example:");
        h.push("shield");
        h.push("");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Shield your transparent ZEC into a sapling address".to_string()
    }
    fn exec(
        &self,
        args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        // Parse the address or amount
        let address = if args.len() > 0 { Some(args[0].to_string()) } else { None };
        RT.block_on(async move {
            match lightclient.do_shield(address).await {
                Ok(txid) => {
                    object! { "txid" => txid }
                },
                Err(e) => {
                    object! { "error" => e }
                },
            }
            .pretty(2)
        })
    }
}
