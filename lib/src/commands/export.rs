use json::object;
use zcash_primitives::consensus;

use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct ExportCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for ExportCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Export private key for an individual wallet addresses.");
        h.push("Note: To backup the whole wallet, use the 'seed' command insted");
        h.push("Usage:");
        h.push("export [t-address or z-address]");
        h.push("");
        h.push("If no address is passed, private key for all addresses in the wallet are exported.");
        h.push("");
        h.push("Example:");
        h.push("export ztestsapling1x65nq4dgp0qfywgxcwk9n0fvm4fysmapgr2q00p85ju252h6l7mmxu2jg9cqqhtvzd69jwhgv8d");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Export private key for wallet addresses".to_string()
    }
    fn exec(
        &self,
        args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        if args.len() > 1 {
            return Command::<P>::help(self);
        }

        RT.block_on(async move {
            let address = if args.is_empty() { None } else { Some(args[0].to_string()) };
            match lightclient.do_export(address).await {
                Ok(j) => j,
                Err(e) => object! { "error" => e },
            }
            .pretty(2)
        })
    }
}
