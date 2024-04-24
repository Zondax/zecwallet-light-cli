use zcash_primitives::consensus;
use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct InfoCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for InfoCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Get info about the lightwalletd we're connected to");
        h.push("Usage:");
        h.push("info");
        h.push("");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Get the lightwalletd server's info".to_string()
    }
    fn exec(
        &self,
        _args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        RT.block_on(async move { lightclient.do_info().await })
    }
}
