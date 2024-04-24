use json::object;
use zcash_primitives::consensus;

use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct NewAddressCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for NewAddressCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Create a new address in this wallet");
        h.push("Usage:");
        h.push("new [u | z | t]");
        h.push("");
        h.push("Example:");
        h.push("To create a new z address:");
        h.push("new z");
        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Create a new address in this wallet".to_string()
    }
    fn exec(
        &self,
        args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        if args.len() != 1 {
            return format!("No address type specified\n{}", Command::<P>::help(self));
        }

        RT.block_on(async move {
            match lightclient
                .do_new_address(args[0])
                .await
            {
                Ok(j) => j,
                Err(e) => object! { "error" => e },
            }
            .pretty(2)
        })
    }
}
