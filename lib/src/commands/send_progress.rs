use zcash_primitives::consensus;

use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct SendProgressCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for SendProgressCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Get the progress of any send transactions that are currently computing");
        h.push("Usage:");
        h.push("sendprogress");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Get the progress of any send transactions that are currently computing".to_string()
    }
    fn exec(
        &self,
        _args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        RT.block_on(async move {
            match lightclient.do_send_progress().await {
                Ok(j) => j.pretty(2),
                Err(e) => e,
            }
        })
    }
}
