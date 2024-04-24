use zcash_primitives::consensus;

use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct NotesCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for NotesCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Show all sapling notes and utxos in this wallet");
        h.push("Usage:");
        h.push("notes [all]");
        h.push("");
        h.push(
            "If you supply the \"all\" parameter, all previously spent sapling notes and spent utxos are also included",
        );

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "List all sapling notes and utxos in the wallet".to_string()
    }
    fn exec(
        &self,
        args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        // Parse the args.
        if args.len() > 1 {
            return Command::<P>::short_help(self);
        }

        // Make sure we can parse the amount
        let all_notes = if args.len() == 1 {
            match args[0] {
                "all" => true,
                a => return format!("Invalid argument \"{}\". Specify 'all' to include unspent notes", a),
            }
        } else {
            false
        };

        RT.block_on(async move {
            format!(
                "{}",
                lightclient
                    .do_list_notes(all_notes)
                    .await
                    .pretty(2)
            )
        })
    }
}
