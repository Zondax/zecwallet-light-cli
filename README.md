# Zecwallet CLI - A command line ZecWallet light client. 

`zecwallet-cli` is a command line ZecWallet light client. To use it, download the latest binary from the releases page and run `./zecwallet-cli`

This will launch the interactive prompt. Type `help` to get a list of commands. See the [Prompt Usage](PROMPT.md) document for more usage information. 

## Running in non-interactive mode:

You can also run `zecwallet-cli` in non-interactive mode by passing the command you want to run as an argument. For example, `zecwallet-cli addresses` will list all wallet addresses and exit. 
Run `zecwallet-cli help` to see a list of all commands. 

## Running using a Ledger device (BETA)

It's possible to use a Ledger device via the [Zondax app](https://github.com/Zondax/ledger-zcash). To do so, invoke the cli with `--ledger`, in addition to any other flags you wish to use.
Do note that not all functionality is available, namely encrypting the wallet data, exporting or importing secret keys and addresses.
The main thing to keep in mind when using a ledger device is that signing keys never leave the device, and for privacy viewing keys are not stored between sessions. 
Additionaly, some actions may require to be confirmed on the device, but this isn't signalled by the cli.

Currently interacting with the device is necessary when:

- generating sapling addresses (IVK and OVK
- starting up the wallet (for the above reason)
- sending funds, to review transaction details
- while syncing, to retrieve nullifiers

## Privacy 

* While all the keys and transaction detection happens on the client, the server can learn what blocks contain your shielded transactions.
* The server also learns other metadata about you like your ip address etc...
* Also remember that t-addresses don't provide any privacy protection.

## Notes:

* If you want to run your own server, please see [zecwallet lightwalletd](https://github.com/adityapk00/lightwalletd), and then run `./zecwallet-cli --server http://127.0.0.1:9067`.
* The log file is in `~/.zcash/zecwallet-light-wallet.debug.log`. Wallet is stored in `~/.zcash/zecwallet-light-wallet.dat`

### Note Management

Zecwallet-CLI does automatic note and utxo management, which means it doesn't allow you to manually select which address to send outgoing transactions from. It follows these principles:

* Defaults to sending shielded transactions, even if you're sending to a transparent address
* Sapling funds need at least 5 confirmations before they can be spent
* Can select funds from multiple shielded addresses in the same transaction
* Will automatically shield your transparent funds at the first opportunity
    * When sending an outgoing transaction to a shielded address, Zecwallet-CLI can decide to use the transaction to additionally shield your transparent funds (i.e., send your transparent funds to your own shielded address in the same transaction)

## Compiling from source

### Pre-requisites

* Rust v1.56 or higher.
    * Run `rustup update` to get the latest version of Rust if you already have it installed
* Rustfmt
    * Run `rustup component add rustfmt` to add rustfmt
* Build tools
    * Please install the build tools for your platform. On Ubuntu `sudo apt install build-essential gcc`

```bash
git clone https://github.com/Zondax/zecwallet-light-cli.git
cargo build --release
./target/release/zecwallet-cli
```

## Options

Here are some CLI arguments you can pass to `zecwallet-cli`. Please run `zecwallet-cli --help` for the full list. 

* `--server`: Connect to a custom zecwallet lightwalletd server. 
    * Example: `./zecwallet-cli --server 127.0.0.1:9067`
* `--seed`: Restore a wallet from a seed phrase. Note that this will fail if there is an existing wallet. Delete (or move) any existing wallet to restore from the 24-word seed phrase
    * Example: `./zecwallet-cli --seed "twenty four words seed phrase"`
 * `--recover`: Attempt to recover the seed phrase from a corrupted wallet
 
 * `--data-dir`: uses the specified path as data directory.
    * Example: `./zecwallet-cli --server 127.0.0.1:9067 --data-dir /Users/ZecWalletRocks/my-test-wallet` will use the provided directory to store `zecwallet-light-wallet.dat` and logs. If the provided directory does not exist, it will create it.


Example:

```bash
    cargo run -- --server https://lwd5.zcash-infra.com:9067 --nosync --ledger --birthday 2301761
```
