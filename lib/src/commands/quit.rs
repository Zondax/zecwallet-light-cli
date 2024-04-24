use zcash_primitives::consensus;
use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct QuitCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for QuitCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Save the wallet to disk and quit");
        h.push("Usage:");
        h.push("quit");
        h.push("");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Quit the lightwallet, saving state to disk".to_string()
    }
    fn exec(
        &self,
        _args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        RT.block_on(async move {
            match lightclient.do_save(true).await {
                Ok(_) => "".to_string(),
                Err(e) => e,
            }
        })
    }
}
