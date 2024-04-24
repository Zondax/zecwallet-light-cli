use zcash_primitives::consensus;
use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct TransactionsCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for TransactionsCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("List all incoming and outgoing transactions from this wallet");
        h.push("Usage:");
        h.push("list [allmemos]");
        h.push("");
        h.push("If you include the 'allmemos' argument, all memos are returned in their raw hex format");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "List all transactions in the wallet".to_string()
    }
    fn exec(
        &self,
        args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        if args.len() > 1 {
            return format!("Didn't understand arguments\n{}", Command::<P>::help(self));
        }

        let include_memo_hex = if args.len() == 1 {
            if args[0] == "allmemos" || args[0] == "true" || args[0] == "yes" {
                true
            } else {
                return format!("Couldn't understand first argument '{}'\n{}", args[0], Command::<P>::help(self));
            }
        } else {
            false
        };

        RT.block_on(async move {
            format!(
                "{}",
                lightclient
                    .do_list_transactions(include_memo_hex)
                    .await
                    .pretty(2)
            )
        })
    }
}
