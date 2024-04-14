# sendprogress - Get the progress of any send transactions that are currently computing
# zecprice - Get the latest ZEC price in the wallet's currency (USD)
# getoption - Get a wallet option
# encrypt - Encrypt the wallet with a password
# defaultfee - Returns the minumum fee in zats for outgoing transactions
# shield - Shield your transparent ZEC into a sapling address
# info - Get the lightwalletd server's info
# seed - Display the seed phrase
# quit - Quit the lightwallet, saving state to disk
# encryptionstatus - Check if the wallet is encrypted and if it is locked
# import - Import spending or viewing keys into the wallet
# list - List all transactions in the wallet
# unlock - Unlock wallet encryption for spending
# decryptmessage - Attempt to decrypt a message with all the view keys in the wallet.
# balance - Show the current ZEC balance in the wallet
# syncstatus - Get the sync status of the wallet
# lasttxid - Show the latest TxId in the wallet
# export - Export private key for wallet addresses
# height - Get the latest block height that the wallet is at
# sync - Download CompactBlocks and sync to the server
# decrypt - Completely remove wallet encryption
# setoption - Set a wallet option
# save - Save wallet file to disk
# notes - List all sapling notes and utxos in the wallet
# lock - Lock a wallet that's been temporarily unlocked
# rescan - Rescan the wallet, downloading and scanning all blocks and transactions
# encryptmessage - Encrypt a memo to be sent to a z-address offline
# help - Lists all available commands
# clear - Clear the wallet state, rolling back the wallet to an empty state.
# addresses - List all addresses in the wallet
# send - Send ZEC to the given address
# new - Create a new address in this wallet

.PHONY: shell run
build:
	@cargo build --release

shell:
	@cargo run -- --server https://lwd5.zcash-infra.com:9067 --ledger --birthday 2301761 

run:
	@cargo run --quiet -- --server https://lwd5.zcash-infra.com:9067 --ledger --birthday 2301761 $(filter-out $@,$(MAKECMDGOALS))
%:
	@:

# test:
# 	make run help
# 	make run help