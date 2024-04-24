use json::object;
use zcash_primitives::consensus;

use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct SaveCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for SaveCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Save the wallet to disk");
        h.push("Usage:");
        h.push("save");
        h.push("");
        h.push("The wallet is saved to disk. The wallet is periodically saved to disk (and also saved upon exit)");
        h.push("but you can use this command to explicitly save it to disk");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Save wallet file to disk".to_string()
    }
    fn exec(
        &self,
        _args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        RT.block_on(async move {
            match lightclient.do_save(true).await {
                Ok(_) => {
                    let r = object! { "result" => "success" };
                    r.pretty(2)
                },
                Err(e) => {
                    let r = object! {
                        "result" => "error",
                        "error" => e
                    };
                    r.pretty(2)
                },
            }
        })
    }
}
