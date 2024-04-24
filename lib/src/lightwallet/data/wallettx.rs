use std::io;
use std::io::{Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use zcash_encoding::{Optional, Vector};
use zcash_primitives::consensus::BlockHeight;
use zcash_primitives::sapling;
use zcash_primitives::transaction::TxId;

use crate::lightwallet::data::notes::{OrchardNoteData, SaplingNoteData};
use crate::lightwallet::data::outgoingtx::OutgoingTxMetadata;
use crate::lightwallet::data::price::WalletZecPriceInfo;
use crate::lightwallet::data::utxo::Utxo;

pub struct WalletTx {
    // Block in which this tx was included
    pub block: BlockHeight,

    // Is this Tx unconfirmed (i.e., not yet mined)
    pub unconfirmed: bool,

    // Timestamp of Tx. Added in v4
    pub datetime: u64,

    // Txid of this transaction. It's duplicated here (It is also the Key in the HashMap that points to this
    // WalletTx in LightWallet::txs)
    pub txid: TxId,

    // List of all nullifiers spent in this Tx. These nullifiers belong to the wallet.
    pub s_spent_nullifiers: Vec<sapling::Nullifier>,

    // List of all orchard nullifiers spent in this Tx.
    pub o_spent_nullifiers: Vec<orchard::note::Nullifier>,

    // List of all notes received in this tx. Some of these might be change notes.
    pub s_notes: Vec<SaplingNoteData>,

    // List of all orchard notes recieved in this tx. Some of these might be change.
    pub o_notes: Vec<OrchardNoteData>,

    // List of all Utxos received in this Tx. Some of these might be change notes
    pub utxos: Vec<Utxo>,

    // Total value of all orchard nullifiers that were spent in this Tx
    pub total_orchard_value_spent: u64,

    // Total value of all the sapling nullifiers that were spent in this Tx
    pub total_sapling_value_spent: u64,

    // Total amount of transparent funds that belong to us that were spent in this Tx.
    pub total_transparent_value_spent: u64,

    // All outgoing sapling sends to addresses outside this wallet
    pub outgoing_metadata: Vec<OutgoingTxMetadata>,

    // Whether this TxID was downloaded from the server and scanned for Memos
    pub full_tx_scanned: bool,

    // Price of Zec when this Tx was created
    pub zec_price: Option<f64>,
}

impl WalletTx {
    pub fn serialized_version() -> u64 {
        return 23;
    }

    pub fn new_txid(txid: &Vec<u8>) -> TxId {
        let mut txid_bytes = [0u8; 32];
        txid_bytes.copy_from_slice(txid);
        TxId::from_bytes(txid_bytes)
    }

    pub fn get_price(
        datetime: u64,
        price: &WalletZecPriceInfo,
    ) -> Option<f64> {
        match price.zec_price {
            None => None,
            Some((t, p)) => {
                // If the price was fetched within 24 hours of this Tx, we use the "current"
                // price else, we mark it as None, for the historical price
                // fetcher to get
                if (t as i64 - datetime as i64).abs() < 24 * 60 * 60 {
                    Some(p)
                } else {
                    None
                }
            },
        }
    }

    pub fn new(
        height: BlockHeight,
        datetime: u64,
        txid: &TxId,
        unconfirmed: bool,
    ) -> Self {
        WalletTx {
            block: height,
            unconfirmed,
            datetime,
            txid: txid.clone(),
            o_spent_nullifiers: vec![],
            s_spent_nullifiers: vec![],
            s_notes: vec![],
            o_notes: vec![],
            utxos: vec![],
            total_transparent_value_spent: 0,
            total_sapling_value_spent: 0,
            total_orchard_value_spent: 0,
            outgoing_metadata: vec![],
            full_tx_scanned: false,
            zec_price: None,
        }
    }

    pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let version = reader.read_u64::<LittleEndian>()?;

        let block = BlockHeight::from_u32(reader.read_i32::<LittleEndian>()? as u32);

        let unconfirmed = if version <= 20 { false } else { reader.read_u8()? == 1 };

        let datetime = if version >= 4 { reader.read_u64::<LittleEndian>()? } else { 0 };

        let mut txid_bytes = [0u8; 32];
        reader.read_exact(&mut txid_bytes)?;

        let txid = TxId::from_bytes(txid_bytes);

        let s_notes = Vector::read(&mut reader, |r| SaplingNoteData::read(r))?;
        let utxos = Vector::read(&mut reader, |r| Utxo::read(r))?;

        let total_orchard_value_spent = if version <= 22 { 0 } else { reader.read_u64::<LittleEndian>()? };
        let total_sapling_value_spent = reader.read_u64::<LittleEndian>()?;
        let total_transparent_value_spent = reader.read_u64::<LittleEndian>()?;

        // Outgoing metadata was only added in version 2
        let outgoing_metadata = Vector::read(&mut reader, |r| OutgoingTxMetadata::read(r))?;

        let full_tx_scanned = reader.read_u8()? > 0;

        let zec_price =
            if version <= 4 { None } else { Optional::read(&mut reader, |r| r.read_f64::<LittleEndian>())? };

        let s_spent_nullifiers = if version <= 5 {
            vec![]
        } else {
            Vector::read(&mut reader, |r| {
                let mut n = [0u8; 32];
                r.read_exact(&mut n)?;
                Ok(sapling::Nullifier(n))
            })?
        };

        let o_notes = if version <= 21 { vec![] } else { Vector::read(&mut reader, |r| OrchardNoteData::read(r))? };

        let o_spent_nullifiers = if version <= 21 {
            vec![]
        } else {
            Vector::read(&mut reader, |r| {
                let mut rho_bytes = [0u8; 32];
                r.read_exact(&mut rho_bytes)?;
                Ok(orchard::note::Nullifier::from_bytes(&rho_bytes).unwrap())
            })?
        };

        Ok(Self {
            block,
            unconfirmed,
            datetime,
            txid,
            s_notes,
            o_notes,
            utxos,
            s_spent_nullifiers,
            o_spent_nullifiers,
            total_sapling_value_spent,
            total_orchard_value_spent,
            total_transparent_value_spent,
            outgoing_metadata,
            full_tx_scanned,
            zec_price,
        })
    }

    pub fn write<W: Write>(
        &self,
        mut writer: W,
    ) -> io::Result<()> {
        writer.write_u64::<LittleEndian>(Self::serialized_version())?;

        let block: u32 = self.block.into();
        writer.write_i32::<LittleEndian>(block as i32)?;

        writer.write_u8(if self.unconfirmed { 1 } else { 0 })?;

        writer.write_u64::<LittleEndian>(self.datetime)?;

        writer.write_all(self.txid.as_ref())?;

        Vector::write(&mut writer, &self.s_notes, |w, nd| nd.write(w))?;
        Vector::write(&mut writer, &self.utxos, |w, u| u.write(w))?;

        writer.write_u64::<LittleEndian>(self.total_orchard_value_spent)?;
        writer.write_u64::<LittleEndian>(self.total_sapling_value_spent)?;
        writer.write_u64::<LittleEndian>(self.total_transparent_value_spent)?;

        // Write the outgoing metadata
        Vector::write(&mut writer, &self.outgoing_metadata, |w, om| om.write(w))?;

        writer.write_u8(if self.full_tx_scanned { 1 } else { 0 })?;

        Optional::write(&mut writer, self.zec_price, |w, p| w.write_f64::<LittleEndian>(p))?;

        Vector::write(&mut writer, &self.s_spent_nullifiers, |w, n| w.write_all(&n.0))?;

        Vector::write(&mut writer, &self.o_notes, |w, n| n.write(w))?;

        Vector::write(&mut writer, &self.o_spent_nullifiers, |w, n| w.write_all(&n.to_bytes()))?;

        Ok(())
    }

    pub fn total_funds_spent(&self) -> u64 {
        self.total_orchard_value_spent + self.total_sapling_value_spent + self.total_transparent_value_spent
    }
}
