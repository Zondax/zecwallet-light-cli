use zcash_primitives::consensus;

use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct ImportCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for ImportCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Import an external spending or viewing key into the wallet");
        h.push("Usage:");
        h.push("import <spending_key | viewing_key> <birthday> [norescan]");
        h.push("OR");
        h.push("import '{'key': <spending_key or viewing_key>, 'birthday': <birthday>, 'norescan': <true>}'");
        h.push("");
        h.push("Birthday is the earliest block number that has transactions belonging to the imported key. Rescanning will start from this block. If not sure, you can specify '0', which will start rescanning from the first sapling block.");
        h.push("Note that you can import only the full spending (private) key or the full viewing key.");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Import spending or viewing keys into the wallet".to_string()
    }
    fn exec(
        &self,
        args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        if args.len() == 0 || args.len() > 3 {
            return format!("Insufficient arguments\n\n{}", Command::<P>::help(self));
        }

        let (key, birthday, rescan) = if args.len() == 1 {
            // If only one arg, parse it as JSON
            let json_args = match json::parse(&args[0]) {
                Ok(j) => j,
                Err(e) => {
                    let es = format!("Couldn't understand JSON: {}", e);
                    return format!("{}\n{}", es, Command::<P>::help(self));
                },
            };

            if !json_args.is_object() {
                return format!("Couldn't parse argument as a JSON object\n{}", Command::<P>::help(self));
            }

            if !json_args.has_key("key") {
                return format!(
                    "'key' field is required in the JSON, containing the spending or viewing key to import\n{}",
                    Command::<P>::help(self)
                );
            }

            if !json_args.has_key("birthday") {
                return format!("'birthday' field is required in the JSON, containing the birthday of the spending or viewing key\n{}",Command::<P>::help(self));
            }

            (
                json_args["key"]
                    .as_str()
                    .unwrap()
                    .to_string(),
                json_args["birthday"].as_u64().unwrap(),
                !json_args["norescan"]
                    .as_bool()
                    .unwrap_or(false),
            )
        } else {
            let key = args[0];
            let birthday = match args[1].parse::<u64>() {
                Ok(b) => b,
                Err(_) => {
                    return format!("Couldn't parse {} as birthday. Please specify an integer. Ok to use '0'", args[1])
                },
            };

            let rescan = if args.len() == 3 {
                if args[2] == "norescan" || args[2] == "false" || args[2] == "no" {
                    false
                } else {
                    return format!(
                        "Couldn't undestand the argument '{}'. Please pass 'norescan' to prevent rescanning the wallet",
                        args[2]
                    );
                }
            } else {
                true
            };

            (key.to_string(), birthday, rescan)
        };

        RT.block_on(async move {
            let r = match lightclient
                .do_import_key(key, birthday)
                .await
            {
                Ok(r) => r.pretty(2),
                Err(e) => return format!("Error: {}", e),
            };

            if rescan {
                match lightclient.do_rescan().await {
                    Ok(_) => {},
                    Err(e) => return format!("Error: Rescan failed: {}", e),
                };
            }

            return r;
        })
    }
}
