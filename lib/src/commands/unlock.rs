use zcash_primitives::consensus;
use json::object;
use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct UnlockCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for UnlockCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Unlock the wallet's encryption in memory, allowing spending from this wallet.");
        h.push("Note 1: This will decrypt spending keys in memory only. The wallet remains encrypted on disk");
        h.push("        Use 'decrypt' to remove the encryption permanatly.");
        h.push("Note 2: If you've forgotten the password, the only way to recover the wallet is to restore");
        h.push("        from the seed phrase.");
        h.push("Usage:");
        h.push("unlock password");
        h.push("");
        h.push("Example:");
        h.push("unlock my_strong_password");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Unlock wallet encryption for spending".to_string()
    }
    fn exec(
        &self,
        args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        if args.len() != 1 {
            return Command::<P>::help(self);
        }

        let passwd = args[0].to_string();
        RT.block_on(async move {
            match lightclient.wallet.unlock(passwd).await {
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
