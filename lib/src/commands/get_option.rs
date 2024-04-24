use json::object;
use zcash_primitives::consensus;

use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::lightwallet::options::MemoDownloadOption;
use crate::RT;

pub struct GetOptionCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for GetOptionCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Get a wallet option");
        h.push("Usage:");
        h.push("getoption <optionname>");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Get a wallet option".to_string()
    }
    fn exec(
        &self,
        args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        if args.len() != 1 {
            return format!("Error: Need exactly 1 argument\n\n{}", Command::<P>::help(self));
        }

        let option_name = args[0];

        RT.block_on(async move {
            let value = match option_name {
                "download_memos" => match lightclient
                    .wallet
                    .wallet_options
                    .read()
                    .await
                    .download_memos
                {
                    MemoDownloadOption::NoMemos => "none",
                    MemoDownloadOption::WalletMemos => "wallet",
                    MemoDownloadOption::AllMemos => "all",
                }
                .to_string(),
                "spam_filter_threshold" => lightclient
                    .wallet
                    .wallet_options
                    .read()
                    .await
                    .spam_threshold
                    .to_string(),
                _ => return format!("Error: Couldn't understand {}", option_name),
            };

            let r = object! {
                option_name => value
            };

            r.pretty(2)
        })
    }
}
