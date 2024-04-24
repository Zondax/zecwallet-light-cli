use zcash_primitives::consensus;
use std::convert::TryInto;
use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::lightwallet::utils;

pub struct EncryptMessageCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for EncryptMessageCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Encrypt a memo to be sent to a z-address offline");
        h.push("Usage:");
        h.push("encryptmessage <address> \"memo\"");
        h.push("OR");
        h.push("encryptmessage \"{'address': <address>, 'memo': <memo>}\" ");
        h.push("");
        h.push("NOTE: This command only returns the encrypted payload. It does not broadcast it. You are expected to send the encrypted payload to the recipient offline");
        h.push("Example:");
        h.push("encryptmessage ztestsapling1x65nq4dgp0qfywgxcwk9n0fvm4fysmapgr2q00p85ju252h6l7mmxu2jg9cqqhtvzd69jwhgv8d \"Hello from the command line\"");
        h.push("");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Encrypt a memo to be sent to a z-address offline".to_string()
    }
    fn exec(
        &self,
        args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        if args.len() < 1 || args.len() > 3 {
            return Command::<P>::help(self);
        }

        // Check for a single argument that can be parsed as JSON
        let (to, memo) = if args.len() == 1 {
            let arg_list = args[0];
            let j = match json::parse(&arg_list) {
                Ok(j) => j,
                Err(e) => {
                    let es = format!("Couldn't understand JSON: {}", e);
                    return format!("{}\n{}", es, Command::<P>::help(self));
                },
            };

            if !j.has_key("address") || !j.has_key("memo") {
                let es = format!("Need 'address' and 'memo'\n");
                return format!("{}\n{}", es, Command::<P>::help(self));
            }

            let memo = utils::interpret_memo_string(j["memo"].as_str().unwrap().to_string());
            if memo.is_err() {
                return format!("{}\n{}", memo.err().unwrap(), Command::<P>::help(self));
            }
            let to = j["address"]
                .as_str()
                .unwrap()
                .to_string();

            (to, memo.unwrap())
        } else if args.len() == 2 {
            let to = args[0].to_string();

            let memo = utils::interpret_memo_string(args[1].to_string());
            if memo.is_err() {
                return format!("{}\n{}", memo.err().unwrap(), Command::<P>::help(self));
            }

            (to, memo.unwrap())
        } else {
            return format!("Wrong number of arguments. Was expecting 1 or 2\n{}", Command::<P>::help(self));
        };

        if let Ok(m) = memo.try_into() {
            lightclient
                .do_encrypt_message(to, m)
                .pretty(2)
        } else {
            return format!("Couldn't encode memo");
        }
    }
}
