use json::object;
use zcash_primitives::consensus;

use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct EncryptCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for EncryptCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Encrypt the wallet with a password");
        h.push("Note 1: This will encrypt the seed and the sapling and transparent private keys.");
        h.push("        Use 'unlock' to temporarily unlock the wallet for spending or 'decrypt' ");
        h.push("        to permanatly remove the encryption");
        h.push("Note 2: If you forget the password, the only way to recover the wallet is to restore");
        h.push("        from the seed phrase.");
        h.push("Usage:");
        h.push("encrypt password");
        h.push("");
        h.push("Example:");
        h.push("encrypt my_strong_password");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Encrypt the wallet with a password".to_string()
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
            match lightclient.wallet.encrypt(passwd).await {
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
