use json::object;
use zcash_primitives::consensus;

use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::lightwallet::options::MemoDownloadOption;
use crate::RT;

pub struct SetOptionCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for SetOptionCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Set a wallet option");
        h.push("Usage:");
        h.push("setoption <optionname>=<optionvalue>");
        h.push("List of available options:");
        h.push("download_memos : none | wallet | all");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Set a wallet option".to_string()
    }
    fn exec(
        &self,
        args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        if args.len() != 1 {
            return format!("Error: Need exactly 1 argument\n\n{}", Command::<P>::help(self));
        }

        let option = args[0];
        let values: Vec<&str> = option.split("=").collect();

        if values.len() != 2 {
            return format!("Error: Please set option value like: <optionname>=<optionvalue>");
        }

        let option_name = values[0];
        let option_value = values[1];

        RT.block_on(async move {
            match option_name {
                "download_memos" => match option_value {
                    "none" => {
                        lightclient
                            .wallet
                            .set_download_memo(MemoDownloadOption::NoMemos)
                            .await
                    },
                    "wallet" => {
                        lightclient
                            .wallet
                            .set_download_memo(MemoDownloadOption::WalletMemos)
                            .await
                    },
                    "all" => {
                        lightclient
                            .wallet
                            .set_download_memo(MemoDownloadOption::AllMemos)
                            .await
                    },
                    _ => return format!("Error: Couldn't understand {} value {}", option_name, option_value),
                },
                "spam_filter_threshold" => {
                    let threshold = option_value.parse::<i64>().unwrap();
                    lightclient
                        .wallet
                        .set_spam_filter_threshold(threshold)
                        .await
                },
                _ => return format!("Error: Couldn't understand {}", option_name),
            }

            let r = object! {
                "success" => true
            };

            r.pretty(2)
        })
    }
}
