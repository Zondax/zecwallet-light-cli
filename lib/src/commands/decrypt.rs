use json::object;
use zcash_primitives::consensus;

use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct DecryptCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for DecryptCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Completely remove wallet encryption, storing the wallet in plaintext on disk");
        h.push(
            "Note 1: This will decrypt the seed and the sapling and transparent private keys and store them on disk.",
        );
        h.push("        Use 'unlock' to temporarily unlock the wallet for spending");
        h.push("Note 2: If you've forgotten the password, the only way to recover the wallet is to restore");
        h.push("        from the seed phrase.");
        h.push("Usage:");
        h.push("decrypt password");
        h.push("");
        h.push("Example:");
        h.push("decrypt my_strong_password");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Completely remove wallet encryption".to_string()
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
            match lightclient
                .wallet
                .remove_encryption(passwd)
                .await
            {
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
