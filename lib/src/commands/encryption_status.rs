use zcash_primitives::consensus;
use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct EncryptionStatusCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for EncryptionStatusCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Check if the wallet is encrypted and if it is locked");
        h.push("Usage:");
        h.push("encryptionstatus");
        h.push("");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Check if the wallet is encrypted and if it is locked".to_string()
    }

    fn exec(
        &self,
        _args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        RT.block_on(async move {
            lightclient
                .do_encryption_status()
                .await
                .pretty(2)
        })
    }
}
