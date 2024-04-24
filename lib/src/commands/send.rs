use zcash_primitives::consensus;
use zcash_primitives::transaction::components::amount::DEFAULT_FEE;
use json::object;
use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::lightwallet::keys::Keystores;
use crate::RT;

pub struct SendCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for SendCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Send ZEC to a given address(es)");
        h.push("Usage:");
        h.push("send <address> <amount in zatoshis || \"entire-verified-zbalance\"> \"optional_memo\"");
        h.push("OR");
        h.push("send '[{'address': <address>, 'amount': <amount in zatoshis>, 'memo': <optional memo>}, ...]'");
        h.push("");
        h.push("NOTE: The fee required to send this transaction (currently ZEC 0.0001) is additionally deducted from your balance.");
        h.push("Example:");
        h.push("send ztestsapling1x65nq4dgp0qfywgxcwk9n0fvm4fysmapgr2q00p85ju252h6l7mmxu2jg9cqqhtvzd69jwhgv8d 200000 \"Hello from the command line\"");
        h.push("");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Send ZEC to the given address".to_string()
    }
    fn exec(
        &self,
        args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        // Parse the args. There are two argument types.
        // 1 - A set of 2(+1 optional) arguments for a single address send representing
        // address, value, memo? 2 - A single argument in the form of a JSON
        // string that is "[{address: address, value: value, memo: memo},...]"
        if args.len() < 1 || args.len() > 3 {
            return Command::<P>::help(self);
        }

        RT.block_on(async move {
            // Check for a single argument that can be parsed as JSON
            let send_args = if args.len() == 1 {
                let arg_list = args[0];

                let json_args = match json::parse(&arg_list) {
                    Ok(j) => j,
                    Err(e) => {
                        let es = format!("Couldn't understand JSON: {}", e);
                        return format!("{}\n{}", es, Command::<P>::help(self));
                    },
                };

                if !json_args.is_array() {
                    return format!("Couldn't parse argument as array\n{}", Command::<P>::help(self));
                }

                let fee = u64::from(DEFAULT_FEE);
                let all_zbalance = lightclient
                    .wallet
                    .verified_zbalance(None)
                    .await
                    .checked_sub(fee);

                let maybe_send_args = json_args
                    .members()
                    .map(|j| {
                        if !j.has_key("address") || !j.has_key("amount") {
                            Err(format!("Need 'address' and 'amount'\n"))
                        } else {
                            let amount = match j["amount"].as_str() {
                                Some("entire-verified-zbalance") => all_zbalance,
                                _ => Some(j["amount"].as_u64().unwrap()),
                            };

                            match amount {
                                Some(amt) => Ok((
                                    j["address"]
                                        .as_str()
                                        .unwrap()
                                        .to_string()
                                        .clone(),
                                    amt,
                                    j["memo"]
                                        .as_str()
                                        .map(|s| s.to_string().clone()),
                                )),
                                None => Err(format!("Not enough in wallet to pay transaction fee of {}", fee)),
                            }
                        }
                    })
                    .collect::<Result<Vec<(String, u64, Option<String>)>, String>>();

                match maybe_send_args {
                    Ok(a) => a.clone(),
                    Err(s) => {
                        return format!("Error: {}\n{}", s, Command::<P>::help(self));
                    },
                }
            } else if args.len() == 2 || args.len() == 3 {
                let address = args[0].to_string();

                // Make sure we can parse the amount
                let value = match args[1].parse::<u64>() {
                    Ok(amt) => amt,
                    Err(e) => {
                        if args[1] == "entire-verified-zbalance" {
                            let fee = u64::from(DEFAULT_FEE);
                            match lightclient
                                .wallet
                                .verified_zbalance(None)
                                .await
                                .checked_sub(fee)
                            {
                                Some(amt) => amt,
                                None => return format!("Not enough in wallet to pay transaction fee of {}", fee),
                            }
                        } else {
                            return format!("Couldn't parse amount: {}", e);
                        }
                    },
                };

                let memo = if args.len() == 3 { Some(args[2].to_string()) } else { None };

                // Memo has to be None if not sending to a shileded address
                if memo.is_some() && !Keystores::is_shielded_address(&address, &lightclient.config.get_params()) {
                    return format!("Can't send a memo to the non-shielded address {}", address);
                }

                vec![(args[0].to_string(), value, memo)]
            } else {
                return Command::<P>::help(self);
            };

            // Convert to the right format. String -> &str.
            let tos = send_args
                .iter()
                .map(|(a, v, m)| (a.as_str(), *v, m.clone()))
                .collect::<Vec<_>>();
            match lightclient.do_send(tos).await {
                Ok(txid) => {
                    object! { "txid" => txid }
                },
                Err(e) => {
                    object! { "error" => e }
                },
            }
            .pretty(2)
        })
    }
}
