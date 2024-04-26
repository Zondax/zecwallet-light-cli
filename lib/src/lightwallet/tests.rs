#[cfg(test)]
mod test {
    use zcash_primitives::transaction::components::Amount;

    use crate::lightclient::blaze::test_utils::{incw_to_string, FakeCompactBlockList, FakeTransaction};
    use crate::lightclient::config::UnitTestNetwork;
    use crate::lightclient::{
        test_server::{create_test_server, mine_pending_blocks, mine_random_blocks},
        LightClient,
    };

    #[tokio::test]
    async fn z_t_note_selection() {
        let (data, config, ready_rx, stop_tx, h1) = create_test_server(UnitTestNetwork).await;
        ready_rx.await.unwrap();

        let mut lc = LightClient::test_new(&config, None, 0)
            .await
            .unwrap();

        let mut fcbl = FakeCompactBlockList::new(0);

        // 1. Mine 10 blocks
        mine_random_blocks(&mut fcbl, &data, &lc, 10).await;
        assert_eq!(lc.wallet.last_scanned_height().await, 10);

        // 2. Send an incoming tx to fill the wallet
        let extfvk1 = lc
            .wallet
            .in_memory_keys()
            .await
            .expect("in memory keystore")
            .get_all_extfvks()[0]
            .clone();
        let value = 100_000;
        let (tx, _height, _) = fcbl.add_tx_paying(&extfvk1, value);
        mine_pending_blocks(&mut fcbl, &data, &lc).await;

        assert_eq!(lc.wallet.last_scanned_height().await, 11);

        // 3. With one confirmation, we should be able to select the note
        let amt = Amount::from_u64(10_000).unwrap();
        // Reset the anchor offsets
        lc.wallet.config.anchor_offset = 0;
        let (_, notes, utxos, selected) = lc
            .wallet
            .select_notes_and_utxos(amt, false, false)
            .await;
        assert!(selected >= amt);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].note.value().inner(), value);
        assert_eq!(utxos.len(), 0);
        assert_eq!(
            incw_to_string(&notes[0].witness),
            incw_to_string(
                lc.wallet
                    .txs
                    .read()
                    .await
                    .current
                    .get(&tx.txid())
                    .unwrap()
                    .s_notes[0]
                    .witnesses
                    .last()
                    .unwrap()
            )
        );

        // With min anchor_offset at 1, we can't select any notes
        lc.wallet.config.anchor_offset = 1;
        let (_, notes, utxos, _selected) = lc
            .wallet
            .select_notes_and_utxos(amt, false, false)
            .await;
        assert_eq!(notes.len(), 0);
        assert_eq!(utxos.len(), 0);

        // Mine 1 block, then it should be selectable
        mine_random_blocks(&mut fcbl, &data, &lc, 1).await;

        let (_, notes, utxos, selected) = lc
            .wallet
            .select_notes_and_utxos(amt, false, false)
            .await;
        assert!(selected >= amt);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].note.value().inner(), value);
        assert_eq!(utxos.len(), 0);
        assert_eq!(
            incw_to_string(&notes[0].witness),
            incw_to_string(
                lc.wallet
                    .txs
                    .read()
                    .await
                    .current
                    .get(&tx.txid())
                    .unwrap()
                    .s_notes[0]
                    .witnesses
                    .get_from_last(1)
                    .unwrap()
            )
        );

        // Mine 15 blocks, then selecting the note should result in witness only 10
        // blocks deep
        mine_random_blocks(&mut fcbl, &data, &lc, 15).await;
        lc.wallet.config.anchor_offset = 9;
        let (_, notes, utxos, selected) = lc
            .wallet
            .select_notes_and_utxos(amt, false, true)
            .await;
        assert!(selected >= amt);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].note.value().inner(), value);
        assert_eq!(utxos.len(), 0);
        assert_eq!(
            incw_to_string(&notes[0].witness),
            incw_to_string(
                lc.wallet
                    .txs
                    .read()
                    .await
                    .current
                    .get(&tx.txid())
                    .unwrap()
                    .s_notes[0]
                    .witnesses
                    .get_from_last(9)
                    .unwrap()
            )
        );

        // Trying to select a large amount will fail
        let amt = Amount::from_u64(1_000_000).unwrap();
        let (_, _, _, selected) = lc
            .wallet
            .select_notes_and_utxos(amt, false, false)
            .await;
        assert!(selected < amt);

        // 4. Get an incoming tx to a t address
        let sk = lc
            .wallet
            .in_memory_keys()
            .await
            .expect("in memory kesytore")
            .tkeys[0]
            .clone();
        let pk = sk.pubkey().unwrap();
        let taddr = sk.address;
        let tvalue = 100_000;

        let mut ftx = FakeTransaction::new();
        ftx.add_t_output(&pk, taddr.clone(), tvalue);
        let (_ttx, _) = fcbl.add_fake_tx(ftx);
        mine_pending_blocks(&mut fcbl, &data, &lc).await;

        // Trying to select a large amount will now succeed
        let amt = Amount::from_u64(value + tvalue - 10_000).unwrap();
        let (_, notes, utxos, selected) = lc
            .wallet
            .select_notes_and_utxos(amt, false, true)
            .await;
        assert_eq!(selected, Amount::from_u64(value + tvalue).unwrap());
        assert_eq!(notes.len(), 1);
        assert_eq!(utxos.len(), 1);

        // If we set transparent-only = true, only the utxo should be selected
        let amt = Amount::from_u64(tvalue - 10_000).unwrap();
        let (_, notes, utxos, selected) = lc
            .wallet
            .select_notes_and_utxos(amt, true, true)
            .await;
        assert_eq!(selected, Amount::from_u64(tvalue).unwrap());
        assert_eq!(notes.len(), 0);
        assert_eq!(utxos.len(), 1);

        // Set min confs to 5, so the sapling note will not be selected
        lc.wallet.config.anchor_offset = 4;
        let amt = Amount::from_u64(tvalue - 10_000).unwrap();
        let (_, notes, utxos, selected) = lc
            .wallet
            .select_notes_and_utxos(amt, false, true)
            .await;
        assert_eq!(selected, Amount::from_u64(tvalue).unwrap());
        assert_eq!(notes.len(), 0);
        assert_eq!(utxos.len(), 1);

        // Shutdown everything cleanly
        stop_tx.send(true).unwrap();
        h1.await.unwrap();
    }

    #[tokio::test]
    async fn multi_z_note_selection() {
        let (data, config, ready_rx, stop_tx, h1) = create_test_server(UnitTestNetwork).await;
        ready_rx.await.unwrap();

        let mut lc = LightClient::test_new(&config, None, 0)
            .await
            .unwrap();

        let mut fcbl = FakeCompactBlockList::new(0);

        // 1. Mine 10 blocks
        mine_random_blocks(&mut fcbl, &data, &lc, 10).await;
        assert_eq!(lc.wallet.last_scanned_height().await, 10);

        // 2. Send an incoming tx to fill the wallet
        let extfvk1 = lc
            .wallet
            .in_memory_keys()
            .await
            .expect("in memory keystore")
            .get_all_extfvks()[0]
            .clone();
        let value1 = 100_000;
        let (tx, _height, _) = fcbl.add_tx_paying(&extfvk1, value1);
        mine_pending_blocks(&mut fcbl, &data, &lc).await;

        assert_eq!(lc.wallet.last_scanned_height().await, 11);

        // 3. With one confirmation, we should be able to select the note
        let amt = Amount::from_u64(10_000).unwrap();
        // Reset the anchor offsets
        lc.wallet.config.anchor_offset = 0;
        let (_, notes, utxos, selected) = lc
            .wallet
            .select_notes_and_utxos(amt, false, false)
            .await;
        assert!(selected >= amt);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].note.value().inner(), value1);
        assert_eq!(utxos.len(), 0);
        assert_eq!(
            incw_to_string(&notes[0].witness),
            incw_to_string(
                lc.wallet
                    .txs
                    .read()
                    .await
                    .current
                    .get(&tx.txid())
                    .unwrap()
                    .s_notes[0]
                    .witnesses
                    .last()
                    .unwrap()
            )
        );

        // Mine 5 blocks
        mine_random_blocks(&mut fcbl, &data, &lc, 5).await;

        // 4. Send another incoming tx.
        let value2 = 200_000;
        let (_tx, _height, _) = fcbl.add_tx_paying(&extfvk1, value2);
        mine_pending_blocks(&mut fcbl, &data, &lc).await;

        // Now, try to select a small amount, it should prefer the older note
        let amt = Amount::from_u64(10_000).unwrap();
        let (_, notes, utxos, selected) = lc
            .wallet
            .select_notes_and_utxos(amt, false, false)
            .await;
        assert!(selected >= amt);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].note.value().inner(), value2);
        assert_eq!(utxos.len(), 0);

        // Selecting a bigger amount should select both notes
        let amt = Amount::from_u64(value1 + value2).unwrap();
        let (_, notes, utxos, selected) = lc
            .wallet
            .select_notes_and_utxos(amt, false, false)
            .await;
        assert!(selected == amt);
        assert_eq!(notes.len(), 2);
        assert_eq!(utxos.len(), 0);

        // Shutdown everything cleanly
        stop_tx.send(true).unwrap();
        h1.await.unwrap();
    }
}
