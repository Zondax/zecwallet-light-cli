use zcash_primitives::consensus;
use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct DecryptMessageCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for DecryptMessageCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Attempt to decrypt a message with all the view keys in the wallet.");
        h.push("Usage:");
        h.push("decryptmessage \"encrypted_message_base64\"");
        h.push("");
        h.push("Example:");
        h.push("decryptmessage RW5jb2RlIGFyYml0cmFyeSBvY3RldHMgYXMgYmFzZTY0LiBSZXR1cm5zIGEgU3RyaW5nLg==");
        h.push("");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Attempt to decrypt a message with all the view keys in the wallet.".to_string()
    }
    fn exec(
        &self,
        args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        if args.len() != 1 {
            return Command::<P>::help(self);
        }

        RT.block_on(async move {
            lightclient
                .do_decrypt_message(args[0].to_string())
                .await
                .pretty(2)
        })
    }
}
