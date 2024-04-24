use json::object;
use zcash_primitives::consensus;
use zcash_primitives::transaction::components::amount::DEFAULT_FEE;

use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct DefaultFeeCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for DefaultFeeCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Returns the default fee in zats for outgoing transactions");
        h.push("Usage:");
        h.push("defaultfee <optional_block_height>");
        h.push("");
        h.push("Example:");
        h.push("defaultfee");
        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Returns the default fee in zats for outgoing transactions".to_string()
    }
    fn exec(
        &self,
        args: &[&str],
        _client: &LightClient<P>,
    ) -> String {
        if args.len() > 1 {
            return format!("Was expecting at most 1 argument\n{}", Command::<P>::help(self));
        }

        RT.block_on(async move {
            let j = object! { "defaultfee" => u64::from(DEFAULT_FEE)};
            j.pretty(2)
        })
    }
}
