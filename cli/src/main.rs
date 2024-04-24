use std::io;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

mod configure;
mod utils;

mod version;

use log::{error, info};
use zecwalletlitelib::lightclient::LightClient;
use zecwalletlitelib::{
    do_user_command,
    lightclient::{self, config::LightClientConfig},
    MainNetwork, Parameters,
};

use self::utils::{attempt_recover_seed, report_permission_error};
use crate::version::VERSION;

pub fn main() {
    // Get command line arguments
    use clap::{App, Arg};
    let fresh_app = App::new("Zecwallet CLI");
    let configured_app = configure_clapapp!(fresh_app);
    let matches = configured_app.get_matches();

    if matches.is_present("recover") {
        // Create a Light Client Config in an attempt to recover the file.
        attempt_recover_seed(
            matches
                .value_of("password")
                .map(|s| s.to_string()),
        );
        return;
    }

    let command = matches.value_of("COMMAND");
    let params = matches
        .values_of("PARAMS")
        .map(|v| v.collect())
        .or(Some(vec![]))
        .unwrap();

    let maybe_server = matches
        .value_of("server")
        .map(|s| s.to_string());

    let maybe_data_dir = matches
        .value_of("data-dir")
        .map(|s| s.to_string());

    let seed = matches
        .value_of("seed")
        .map(|s| s.to_string());
    let ledger = matches.is_present("ledger");
    let maybe_birthday = matches.value_of("birthday");

    if seed.is_some() && maybe_birthday.is_none() {
        eprintln!("ERROR!");
        eprintln!("Please specify the wallet birthday (eg. '--birthday 600000') to restore from seed.");
        eprintln!("This should be the block height where the wallet was created. If you don't remember the block height, you can pass '--birthday 0' to scan from the start of the blockchain.");
        return;
    }

    let birthday = match maybe_birthday
        .unwrap_or("0")
        .parse::<u64>()
    {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Couldn't parse birthday. This should be a block number. Error={}", e);
            return;
        },
    };

    let server = LightClientConfig::<MainNetwork>::get_server_or_default(maybe_server);

    // Test to make sure the server has all schemes, host and port
    if server.scheme_str().is_none() || server.host().is_none() || server.port().is_none() {
        eprintln!("Please provide the --server parameter as [scheme]://[host]:[port].\nYou provided: {}", server);
        return;
    }

    let nosync = matches.is_present("nosync");

    let startup_chan = startup(server, seed, birthday, maybe_data_dir, !nosync, command.is_none(), ledger);

    let (command_tx, resp_rx) = match startup_chan {
        Ok(c) => c,
        Err(e) => {
            let emsg = format!("Error during startup: {}\nIf you repeatedly run into this issue, you might have to restore your wallet from your seed phrase.", e);
            eprintln!("{}", emsg);
            error!("{}", emsg);
            if cfg!(target_os = "unix") {
                match e.raw_os_error() {
                    Some(13) => report_permission_error(),
                    _ => {},
                }
            };
            return;
        },
    };

    if command.is_none() {
        start_interactive(command_tx, resp_rx);
    } else {
        command_tx
            .send((
                command.unwrap().to_string(),
                params
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>(),
            ))
            .unwrap();

        match resp_rx.recv() {
            Ok(s) => println!("{}", s),
            Err(e) => {
                let e = format!("Error executing command {}: {}", command.unwrap(), e);
                eprintln!("{}", e);
                error!("{}", e);
            },
        }

        // Save before exit
        command_tx
            .send(("save".to_string(), vec![]))
            .unwrap();
        resp_rx.recv().unwrap();
    }
}

pub fn startup(
    server: http::Uri,
    seed: Option<String>,
    birthday: u64,
    data_dir: Option<String>,
    first_sync: bool,
    print_updates: bool,
    ledger: bool,
) -> io::Result<(Sender<(String, Vec<String>)>, Receiver<String>)> {
    // Try to get the configuration
    let (config, latest_block_height) = LightClientConfig::new(MainNetwork, server.clone(), data_dir)?;

    let lightclient = match seed {
        Some(phrase) => Arc::new(LightClient::new_from_phrase(phrase, &config, birthday, false)?),
        None if ledger => Arc::new(LightClient::with_ledger(&config, birthday)?),
        None => {
            if config.wallet_exists() {
                Arc::new(LightClient::read_from_disk(&config)?)
            } else {
                println!("Creating a new wallet");
                // Create a wallet with height - 100, to protect against reorgs
                Arc::new(LightClient::new(&config, latest_block_height.saturating_sub(100))?)
            }
        },
    };

    // Initialize logging
    lightclient.init_logging()?;

    // Print startup Messages
    info!(""); // Blank line
    info!("Starting Zecwallet-CLI");
    info!("Light Client config {:?}", config);

    if print_updates {
        println!("Lightclient connecting to {}", config.server);
    }

    // At startup, run a sync.
    if first_sync {
        let update = do_user_command("sync", &vec![], lightclient.as_ref());
        if print_updates {
            println!("{}", update);
        }
    }

    // Start the command loop
    let (command_tx, resp_rx) = command_loop(lightclient.clone());

    Ok((command_tx, resp_rx))
}

pub fn start_interactive(
    command_tx: Sender<(String, Vec<String>)>,
    resp_rx: Receiver<String>,
) {
    // `()` can be used when no completer is required
    let mut rl = rustyline::Editor::<()>::new();

    println!("Ready!");

    let send_command = |cmd: String, args: Vec<String>| -> String {
        command_tx
            .send((cmd.clone(), args))
            .unwrap();
        match resp_rx.recv() {
            Ok(s) => s,
            Err(e) => {
                let e = format!("Error executing command {}: {}", cmd, e);
                eprintln!("{}", e);
                error!("{}", e);
                return "".to_string();
            },
        }
    };

    let info = send_command("info".to_string(), vec![]);
    let chain_name = match json::parse(&info) {
        Ok(s) => s["chain_name"]
            .as_str()
            .unwrap()
            .to_string(),
        Err(e) => {
            error!("{}", e);
            eprintln!("Couldn't get chain name. {}", e);
            return;
        },
    };

    loop {
        // Read the height first
        let height = json::parse(&send_command("height".to_string(), vec!["false".to_string()])).unwrap()["height"]
            .as_i64()
            .unwrap();

        let readline = rl.readline(&format!("({}) Block:{} (type 'help') >> ", chain_name, height));
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                // Parse command line arguments
                let mut cmd_args = match shellwords::split(&line) {
                    Ok(args) => args,
                    Err(_) => {
                        println!("Mismatched Quotes");
                        continue;
                    },
                };

                if cmd_args.is_empty() {
                    continue;
                }

                let cmd = cmd_args.remove(0);
                let args: Vec<String> = cmd_args;

                println!("{}", send_command(cmd, args));

                // Special check for Quit command.
                if line == "quit" {
                    break;
                }
            },
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("CTRL-C");
                info!("CTRL-C");
                println!("{}", send_command("save".to_string(), vec![]));
                break;
            },
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("CTRL-D");
                info!("CTRL-D");
                println!("{}", send_command("save".to_string(), vec![]));
                break;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            },
        }
    }
}

pub fn command_loop<P: Parameters + Send + Sync + 'static>(
    lightclient: Arc<LightClient<P>>
) -> (Sender<(String, Vec<String>)>, Receiver<String>) {
    let (command_tx, command_rx) = channel::<(String, Vec<String>)>();
    let (resp_tx, resp_rx) = channel::<String>();

    let lc = lightclient.clone();
    std::thread::spawn(move || {
        LightClient::start_mempool_monitor(lc.clone());

        loop {
            if let Ok((cmd, args)) = command_rx.recv() {
                let args = args
                    .iter()
                    .map(|s| s.as_ref())
                    .collect();

                let cmd_response = do_user_command(&cmd, &args, lc.as_ref());
                resp_tx.send(cmd_response).unwrap();

                if cmd == "quit" {
                    info!("Quit");
                    break;
                }
            } else {
                break;
            }
        }
    });

    (command_tx, resp_rx)
}
