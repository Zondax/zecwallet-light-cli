mod default_fee;
mod address;
mod balance;
mod clear;
mod decrypt;
mod decrypt_message;
mod encrypt;
mod encrypt_message;
mod encryption_status;
mod export;
mod get_option;
mod height;
mod help;
mod import;
mod info;
mod last_tx_id;
mod lock;
mod new_address;
mod notes;
mod quit;
mod rescan;
mod save;
mod seed;
mod send;
mod send_progress;
mod set_option;
mod shield;
mod sync;
mod sync_status;
mod transactions;
mod unlock;
mod zecprice;

use zcash_primitives::consensus;
use std::collections::HashMap;
use crate::commands::address::AddressCommand;
use crate::commands::balance::BalanceCommand;
use crate::commands::clear::ClearCommand;
use crate::commands::decrypt::DecryptCommand;
use crate::commands::decrypt_message::DecryptMessageCommand;
use crate::commands::default_fee::DefaultFeeCommand;
use crate::commands::encrypt::EncryptCommand;
use crate::commands::encrypt_message::EncryptMessageCommand;
use crate::commands::encryption_status::EncryptionStatusCommand;
use crate::commands::export::ExportCommand;
use crate::commands::get_option::GetOptionCommand;
use crate::commands::height::HeightCommand;
use crate::commands::help::HelpCommand;
use crate::commands::import::ImportCommand;
use crate::commands::info::InfoCommand;
use crate::commands::last_tx_id::LastTxIdCommand;
use crate::commands::lock::LockCommand;
use crate::commands::new_address::NewAddressCommand;
use crate::commands::notes::NotesCommand;
use crate::commands::quit::QuitCommand;
use crate::commands::rescan::RescanCommand;
use crate::commands::save::SaveCommand;
use crate::commands::seed::SeedCommand;
use crate::commands::send::SendCommand;
use crate::commands::send_progress::SendProgressCommand;
use crate::commands::set_option::SetOptionCommand;
use crate::commands::shield::ShieldCommand;
use crate::commands::sync::SyncCommand;
use crate::commands::sync_status::SyncStatusCommand;
use crate::commands::transactions::TransactionsCommand;
use crate::commands::unlock::UnlockCommand;
use crate::commands::zecprice::ZecPriceCommand;
use crate::lightclient::LightClient;

pub trait Command<P> {
    fn help(&self) -> String;
    fn short_help(&self) -> String;

    fn exec(
        &self,
        _args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String;
}

pub fn get_commands<P: consensus::Parameters + Send + Sync + 'static>() -> Box<HashMap<String, Box<dyn Command<P>>>> {
    let mut map: HashMap<String, Box<dyn Command<P>>> = HashMap::new();

    map.insert("sync".to_string(), Box::new(SyncCommand {}));
    map.insert("syncstatus".to_string(), Box::new(SyncStatusCommand {}));
    map.insert("encryptionstatus".to_string(), Box::new(EncryptionStatusCommand {}));
    map.insert("encryptmessage".to_string(), Box::new(EncryptMessageCommand {}));
    map.insert("decryptmessage".to_string(), Box::new(DecryptMessageCommand {}));
    map.insert("rescan".to_string(), Box::new(RescanCommand {}));
    map.insert("clear".to_string(), Box::new(ClearCommand {}));
    map.insert("help".to_string(), Box::new(HelpCommand {}));
    map.insert("lasttxid".to_string(), Box::new(LastTxIdCommand {}));
    map.insert("balance".to_string(), Box::new(BalanceCommand {}));
    map.insert("addresses".to_string(), Box::new(AddressCommand {}));
    map.insert("height".to_string(), Box::new(HeightCommand {}));
    map.insert("sendprogress".to_string(), Box::new(SendProgressCommand {}));
    map.insert("setoption".to_string(), Box::new(SetOptionCommand {}));
    map.insert("getoption".to_string(), Box::new(GetOptionCommand {}));
    map.insert("import".to_string(), Box::new(ImportCommand {}));
    map.insert("export".to_string(), Box::new(ExportCommand {}));
    map.insert("info".to_string(), Box::new(InfoCommand {}));
    map.insert("zecprice".to_string(), Box::new(ZecPriceCommand {}));
    map.insert("send".to_string(), Box::new(SendCommand {}));
    map.insert("shield".to_string(), Box::new(ShieldCommand {}));
    map.insert("save".to_string(), Box::new(SaveCommand {}));
    map.insert("quit".to_string(), Box::new(QuitCommand {}));
    map.insert("list".to_string(), Box::new(TransactionsCommand {}));
    map.insert("notes".to_string(), Box::new(NotesCommand {}));
    map.insert("new".to_string(), Box::new(NewAddressCommand {}));
    map.insert("defaultfee".to_string(), Box::new(DefaultFeeCommand {}));
    map.insert("seed".to_string(), Box::new(SeedCommand {}));
    map.insert("encrypt".to_string(), Box::new(EncryptCommand {}));
    map.insert("decrypt".to_string(), Box::new(DecryptCommand {}));
    map.insert("unlock".to_string(), Box::new(UnlockCommand {}));
    map.insert("lock".to_string(), Box::new(LockCommand {}));

    Box::new(map)
}

pub fn do_user_command<P: consensus::Parameters + Send + Sync + 'static>(
    cmd: &str,
    args: &Vec<&str>,
    lightclient: &LightClient<P>,
) -> String {
    match get_commands().get(&cmd.to_ascii_lowercase()) {
        Some(cmd) => cmd.exec(args, lightclient),
        None => format!("Unknown command : {}. Type 'help' for a list of commands", cmd),
    }
}

#[cfg(test)]
pub mod tests {
    use lazy_static::lazy_static;
    use tokio::runtime::Runtime;
    use crate::commands::do_user_command;

    use crate::lightclient::{
        LightClient,
        lightclient_config::{LightClientConfig, UnitTestNetwork},
    };

    lazy_static! {
        static ref TEST_SEED: String = "youth strong sweet gorilla hammer unhappy congress stamp left stereo riot salute road tag clean toilet artefact fork certain leopard entire civil degree wonder".to_string();
    }

    pub fn test_command_caseinsensitive() {
        let lc = Runtime::new()
            .unwrap()
            .block_on(LightClient::test_new(
                &LightClientConfig::create_unconnected(UnitTestNetwork, None),
                Some(TEST_SEED.to_string()),
                0,
            ))
            .unwrap();

        assert_eq!(do_user_command("addresses", &vec![], &lc), do_user_command("AddReSSeS", &vec![], &lc));
        assert_eq!(do_user_command("addresses", &vec![], &lc), do_user_command("Addresses", &vec![], &lc));
    }

    #[test]
    pub fn test_nosync_commands() {
        // The following commands should run
    }
}
