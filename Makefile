# Define the server URL and birthday as variables
SERVER_URL = https://lwd5.zcash-infra.com:9067
BIRTHDAY = 2466807

# quit - Quit the lightwallet, saving state to disk
# info - Get the lightwalletd server's info
# help - Lists all available commands

# zecprice - Get the latest ZEC price in the wallet's currency (USD)
# defaultfee - Returns the minumum fee in zats for outgoing transactions

# todo: ?? what are possible options
# getoption - Get a wallet option
#	download_memos (none, wallet, all
#	spam_filter_threshold
# setoption - Set a wallet option
#	download_memos { none, wallet, all }
#	spam_filter_threshold

##### Chain/Sync commands

# rescan - Rescan the wallet, downloading and scanning all blocks and transactions
# syncstatus - Get the sync status of the wallet
# lasttxid - Show the latest TxId in the wallet
# height - Get the latest block height that the wallet is at
# sync - Download CompactBlocks and sync to the server

##### Generic wallet commands

# balance - Show the current ZEC balance in the wallet
# list - List all transactions in the wallet

# shield - Shield your transparent ZEC into a sapling address
# notes - List all sapling notes and utxos in the wallet
# clear - Clear the wallet state, rolling back the wallet to an empty state.
# addresses - List all addresses in the wallet
# send - Send ZEC to the given address
# sendprogress - Get the progress of any send transactions that are currently computing
# encryptmessage - Encrypt a memo to be sent to a z-address offline
# decryptmessage - Attempt to decrypt a message with all the view keys in the wallet.
# import - Import spending or viewing keys into the wallet
# new - Create a new address in this wallet					????????

##### Soft wallet commands

# encrypt - Encrypt the wallet with a password
# decrypt - Completely remove wallet encryption
# encryptionstatus - Check if the wallet is encrypted and if it is locked
# lock - Lock a wallet that's been temporarily unlocked
# unlock - Unlock wallet encryption for spending
# export - Export private key for wallet addresses
# seed - Display the seed phrase
# save - Save wallet file to disk					????????

.PHONY: shell run
build:
	@cargo build --release

shell:
	@cargo run -- --server $(SERVER_URL) --ledger --birthday $(BIRTHDAY)

run:
	@cargo run --quiet -- --server $(SERVER_URL) --ledger --birthday $(BIRTHDAY) $(filter-out $@,$(MAKECMDGOALS))

%:
	@:

test:
	@make run seed

