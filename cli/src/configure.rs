#[macro_export]
macro_rules! configure_clapapp {
    ( $freshapp: expr ) => {
    $freshapp.version(VERSION)
            .arg(Arg::with_name("nosync")
                .help("By default, zecwallet-cli will sync the wallet at startup. Pass --nosync to prevent the automatic sync at startup.")
                .long("nosync")
                .short('n')
                .takes_value(false))
            .arg(Arg::with_name("recover")
                .long("recover")
                .help("Attempt to recover the seed from the wallet")
                .takes_value(false))
            .arg(Arg::with_name("password")
                .long("password")
                .help("When recovering seed, specify a password for the encrypted wallet")
                .takes_value(true))
            .arg(Arg::with_name("seed")
                .short('s')
                .long("seed")
                .value_name("seed_phrase")
                .help("Create a new wallet with the given 24-word seed phrase. Will fail if wallet already exists")
                .takes_value(true))
            .arg(Arg::with_name("ledger")
                 .long("ledger")
                 .value_name("ledger")
                 .help("Create a new wallet by connecting to a ledger")
                 .takes_value(false))
            .arg(Arg::with_name("birthday")
                .long("birthday")
                .value_name("birthday")
                .help("Specify wallet birthday when restoring from seed. This is the earlist block height where the wallet has a transaction.")
                .takes_value(true))
            .arg(Arg::with_name("server")
                .long("server")
                .value_name("server")
                .help("Lightwalletd server to connect to.")
                .takes_value(true)
                .default_value(lightclient::config::DEFAULT_SERVER)
                .takes_value(true))
            .arg(Arg::with_name("data-dir")
                .long("data-dir")
                .value_name("data-dir")
                .help("Absolute path to use as data directory")
                .takes_value(true))
            .arg(Arg::with_name("COMMAND")
                .help("Command to execute. If a command is not specified, zecwallet-cli will start in interactive mode.")
                .required(false)
                .index(1))
            .arg(Arg::with_name("PARAMS")
                .help("Params to execute command with. Run the 'help' command to get usage help.")
                .required(false)
                .multiple(true))
    };
}
