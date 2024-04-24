use json::object;
use zcash_primitives::consensus;

use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct LockCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for LockCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Lock a wallet that's been temporarily unlocked. You should already have encryption enabled.");
        h.push("Note 1: This will remove all spending keys from memory. The wallet remains encrypted on disk");
        h.push("Note 2: If you've forgotten the password, the only way to recover the wallet is to restore");
        h.push("        from the seed phrase.");
        h.push("Usage:");
        h.push("lock");
        h.push("");
        h.push("Example:");
        h.push("lock");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Lock a wallet that's been temporarily unlocked".to_string()
    }
    fn exec(
        &self,
        args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        if !args.is_empty() {
            let mut h = vec![];
            h.push("Extra arguments to lock. Did you mean 'encrypt'?");
            h.push("");

            return format!("{}\n{}", h.join("\n"), Command::<P>::help(self));
        }

        RT.block_on(async move {
            match lightclient.wallet.lock().await {
                Ok(_) => object! { "result" => "success" },
                Err(e) => object! {
                    "result" => "error",
                    "error"  => e.to_string()
                },
            }
            .pretty(2)
        })
    }
}
