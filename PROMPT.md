# Zecwallet CLI - Prompt usage

`zecwallet-cli` contains a variety of commands, each meant to be used for different purposes.

For the most up to date and relevant information pertaining the version in use please use the [`help`](#Help) command.

---

# Help 

Lists all available commands with a small description. Specify a command to view all information pertaining that command.

```
(chain) Block:... (type 'help') >> help
Available commands:
lasttxid - Show the latest TxId in the wallet
notes - List all sapling notes and utxos in the wallet
getoption - Get a wallet option
......
(chain) Block:... (type 'help') >> help addresses
List current addresses in the wallet
Usage:
addresses
...
```

# Generate an address

Create a new address via the wallet. If the address doesn't show funds that have been just received try `sync`ing the wallet or `rescan`ning the chain.
```
(chain) Block:... (type 'help') >> new t
[
  "generated address"
]
(chain) Block:... (type 'help') >> new z
[
  "a generated sapling address"
]
```

# List Addresses

List all known addresses, grouping them by type
```
(chain) Block:... (type 'help') >> addresses
{
  "z_addresses": [
    "a sapling address"
  ],
  "t_addresses": [
    "a transparent address",
    "another transparent address"
  ]
}
```

# Balance

Retrieve the current balance, specifying if it's confirmed, unconfirmed and which address holds how many zatoshis
```
(chain) Block:... (type 'help') >> balance
{
  "zbalance": 111912,
  "verified_zbalance": 111912,
  "spendable_zbalance": 111912,
  "unverified_zbalance": 0,
  "tbalance": 1477083,
  "z_addresses": [
    {
      "address": "sapling addr",
      "zbalance": 111912,
      "verified_zbalance": 111912,
      "spendable_zbalance": 111912,
      "unverified_zbalance": 0
    }
  ],
  "t_addresses": [
    {
      "address": "transparent addr",
      "balance": 60000
    },
    {
      "address": "other address",
      "balance": 1417083
    }
  ]
}
```


# Send funds

Use `send` to send funds to a transparent or a shielded address.
```
(chain) Block:.... (type 'help') >> send tmWcbLWx8ck3fGf31YGyFR5xvnHej9JVA9F 12345
0: Creating transaction sending 12345 ztoshis to 1 addresses
0: Selecting notes
0: Adding 0 notes and 3 utxos
0: Adding output
0: Building transaction
9: Transaction created
Transaction ID: ed8a8a725e9b97f5a6e58da6613c72709953ee0cc4237c2a060a91baab2be923
{
  "txid": "ed8a8a725e9b97f5a6e58da6613c72709953ee0cc4237c2a060a91baab2be923"
}
```

```
(test) Block:2583434 (type 'help') >> send ztestsapling1p55g4snrsfw3944zq7kn9nwel3vjztws9hr88qg2jej6rmrnjwzxxzkk4mc8g4jm6h4ww7et0wv 1501083
0: Creating transaction sending 1501083 ztoshis to 1 addresses
0: Selecting notes
0: Adding 1 notes and 1 utxos
0: Adding output
0: Building transaction
Progress: 1
Progress: 2
Progress: 3
39: Transaction created
Transaction ID: cc55da4c6f29f8447becc9c5b97386b19bcbd17968ceec59f3c303a58d0d732e
{
  "txid": "cc55da4c6f29f8447becc9c5b97386b19bcbd17968ceec59f3c303a58d0d732e"
}
```

# Rescan

Used to rescan the entire wallet (from a given birthday) to find funds sent to and addresses that was unknown to the wallet so far (for example, imported)

```
(test) Block:2583434 (type 'help') >> rescan
id: 2, batch: 0/2, blocks: 1000/1000, decryptions: 950, tx_scan: 0
id: 2, batch: 0/2, blocks: 1000/1000, decryptions: 950, tx_scan: 993
{
  "result": "success",
  "latest_block": 2583436,
  "total_blocks_synced": 336
}
```
