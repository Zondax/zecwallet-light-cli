# How to test ZecwalletLite using Ledger

First of all, clone the following repositories:
- https://github.com/Zondax/zecwallet-lite (Electron GUI)
- https://github.com/Zondax/zecwallet-light-cli (CLI)
- https://github.com/Zondax/ledger-zcash (Ledger)

Please checkout the following branches
- `master` for ZecwalletLite (GUI)
- `main` for zecwallet-light-cli (cli)
- `dev` or `main` for ledger-zcash (ledger)

# Build and install the Ledger app

Before testing the wallet integration it's necessary to build and install the ledger app onto the device.
Please read the instructions providede on the ledger-zcash repository for how to do so, but in short:
```
$ make
$ ./app/pkg/installer_<device>.sh load
```
where `<device>` is the shortname for your device. Note that altough an installer for Nano X is provided, 
the device doesn't allow sideloading of apps, such as in this case.

# Launch the wallet

After the app has been installed, the device has been connected and the app opened, the wallet can be launched.

Note that both GUI and CLI wallet provide the same set of features and rely on the same mechanism to interact with the ledger.
Therefore, the only difference in the 2 is the UI and the resulting UX, but otherwise all communcation with the device is done in the same manner.

## CLI

To simply launch the CLI wallet, in a terminal navigate to the appropriate repo and execute the following:
```
cargo run -- --ledger
```

There are a few flags that can be given to the wallet during start which influence its behaviour.
Here are listed a few flags and their values recommended to use for testing:
- `--server https://lwd1.zcash-infra.com:9067`
  This will connect the device to a recently made instance of LightwalletD syncronized with Mainnet.
  By default the value of the `--server` flag is `https://lwdv3.zecwallet.co` (which is unavailable at the time of writing)
- `--birthday 2291639`
  This will tell the wallet to avoid scanning the blocks before the given one.

Please refer to the `PROMPT.md` document in the `zecwallet-light-cli` for a few useful commands for testing.

## GUI

To launch the GUI, in a terminal navigate to the appropriate repo and execute the following.
Note that there may be a few system-specific dependencies needed, namely:
- yarn
- rustup (and cargo)
```
yarn install
yarn start
```
