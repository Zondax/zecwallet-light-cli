use std::convert::TryFrom;
use std::io::{Read, Write};
use std::{io, usize};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use incrementalmerkletree::Position;
use orchard::keys::FullViewingKey;
use orchard::note::RandomSeed;
use orchard::value::NoteValue;
use orchard::Address;
use zcash_encoding::{Optional, Vector};
use zcash_primitives::memo::{Memo, MemoBytes};
use zcash_primitives::merkle_tree::IncrementalWitness;
use zcash_primitives::sapling;
use zcash_primitives::sapling::{Diversifier, Node, Rseed};
use zcash_primitives::transaction::TxId;
use zcash_primitives::zip32::{ExtendedFullViewingKey, ExtendedSpendingKey};

use crate::lightwallet::data::utils;
use crate::lightwallet::data::witnesscache::WitnessCache;

pub struct OrchardNoteData {
    pub(in crate::lightwallet) fvk: FullViewingKey,

    pub note: orchard::Note,

    // (Block number, tx_num, output_num)
    pub created_at: (u64, usize, u32),
    pub witness_position: Option<Position>,

    // Info needed to recreate note
    pub spent: Option<(TxId, u32)>, // If this note was confirmed spent

    // If this note was spent in a send, but has not yet been confirmed.
    // Contains the txid and height at which it was broadcast
    pub unconfirmed_spent: Option<(TxId, u32)>,
    pub memo: Option<Memo>,
    pub is_change: bool,

    // If the spending key is available in the wallet (i.e., whether to keep witness up-to-date)
    pub have_spending_key: bool,
}

impl OrchardNoteData {
    fn serialized_version() -> u64 {
        22
    }

    // Reading a note also needs the corresponding address to read from.
    pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let version = reader.read_u64::<LittleEndian>()?;
        assert!(version <= Self::serialized_version());

        let fvk = FullViewingKey::read(&mut reader)?;

        // Read the parts of the note
        // Raw address bytes is 43
        let mut address_bytes = [0u8; 43];
        reader.read_exact(&mut address_bytes)?;
        let note_address = Address::from_raw_address_bytes(&address_bytes).unwrap();
        let note_value = reader.read_u64::<LittleEndian>()?;
        let mut rho_bytes = [0u8; 32];
        reader.read_exact(&mut rho_bytes)?;
        let note_rho = orchard::note::Nullifier::from_bytes(&rho_bytes).unwrap();
        let mut note_rseed_bytes = [0u8; 32];
        reader.read_exact(&mut note_rseed_bytes)?;
        let note_rseed = RandomSeed::from_bytes(note_rseed_bytes, &note_rho).unwrap();

        let note = orchard::Note::from_parts(note_address, NoteValue::from_raw(note_value), note_rho, note_rseed).unwrap();

        let witness_position = Optional::read(&mut reader, |r| {
            let pos = r.read_u64::<LittleEndian>()?;
            Ok(Position::from(pos as usize))
        })?;

        let spent = Optional::read(&mut reader, |r| {
            let mut txid_bytes = [0u8; 32];
            r.read_exact(&mut txid_bytes)?;
            let height = r.read_u32::<LittleEndian>()?;
            Ok((TxId::from_bytes(txid_bytes), height))
        })?;

        let unconfirmed_spent = Optional::read(&mut reader, |r| {
            let mut txbytes = [0u8; 32];
            r.read_exact(&mut txbytes)?;

            let height = r.read_u32::<LittleEndian>()?;
            Ok((TxId::from_bytes(txbytes), height))
        })?;

        let memo = Optional::read(&mut reader, |r| {
            let mut memo_bytes = [0u8; 512];
            r.read_exact(&mut memo_bytes)?;

            // Attempt to read memo, first as text, else as arbitrary 512 bytes
            match MemoBytes::from_bytes(&memo_bytes) {
                Ok(mb) => match Memo::try_from(mb.clone()) {
                    Ok(m) => Ok(m),
                    Err(_) => Ok(Memo::Future(mb)),
                },
                Err(e) => Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Couldn't create memo: {}", e))),
            }
        })?;

        let is_change: bool = reader.read_u8()? > 0;

        let have_spending_key = reader.read_u8()? > 0;

        Ok(OrchardNoteData {
            fvk,
            note,
            created_at: (0, 0, 0),
            witness_position,
            spent,
            unconfirmed_spent,
            memo,
            is_change,
            have_spending_key,
        })
    }

    pub fn write<W: Write>(
        &self,
        mut writer: W,
    ) -> io::Result<()> {
        // Write a version number first, so we can later upgrade this if needed.
        writer.write_u64::<LittleEndian>(Self::serialized_version())?;

        self.fvk.write(&mut writer)?;

        // Write the components of the note
        writer.write_all(
            &self
                .note
                .recipient()
                .to_raw_address_bytes(),
        )?;
        writer.write_u64::<LittleEndian>(self.note.value().inner())?;
        writer.write_all(&self.note.rho().to_bytes())?;
        writer.write_all(self.note.rseed().as_bytes())?;

        // We don't write the created_at, because it should be temporary
        Optional::write(&mut writer, self.witness_position, |w, p| w.write_u64::<LittleEndian>(p.into()))?;

        Optional::write(&mut writer, self.spent, |w, (txid, h)| {
            w.write_all(txid.as_ref())?;
            w.write_u32::<LittleEndian>(h)
        })?;

        Optional::write(&mut writer, self.unconfirmed_spent, |w, (txid, height)| {
            w.write_all(txid.as_ref())?;
            w.write_u32::<LittleEndian>(height)
        })?;

        Optional::write(&mut writer, self.memo.as_ref(), |w, m| w.write_all(m.encode().as_array()))?;

        writer.write_u8(if self.is_change { 1 } else { 0 })?;

        writer.write_u8(if self.have_spending_key { 1 } else { 0 })?;

        // Note that we don't write the unconfirmed_spent field, because if the wallet
        // is restarted, we don't want to be beholden to any expired txns

        Ok(())
    }
}

pub struct SaplingNoteData {
    // Technically, this should be recoverable from the account number,
    // but we're going to refactor this in the future, so I'll write it again here.
    pub(in crate::lightwallet) extfvk: ExtendedFullViewingKey,

    pub diversifier: Diversifier,
    pub note: sapling::Note,

    // Witnesses for the last 100 blocks. witnesses.last() is the latest witness
    pub(crate) witnesses: WitnessCache,
    pub(in crate::lightwallet) nullifier: sapling::Nullifier,
    pub spent: Option<(TxId, u32)>, // If this note was confirmed spent

    // If this note was spent in a send tx, but has not yet been confirmed.
    // Contains the txid and height at which it was broadcast
    pub unconfirmed_spent: Option<(TxId, u32)>,
    pub memo: Option<Memo>,
    pub is_change: bool,

    // If the spending key is available in the wallet (i.e., whether to keep witness up-to-date)
    pub have_spending_key: bool,
}

impl SaplingNoteData {
    fn serialized_version() -> u64 {
        20
    }

    // Reading a note also needs the corresponding address to read from.
    pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let version = reader.read_u64::<LittleEndian>()?;

        let _account = if version <= 5 { reader.read_u64::<LittleEndian>()? } else { 0 };

        let extfvk = ExtendedFullViewingKey::read(&mut reader)?;

        let mut diversifier_bytes = [0u8; 11];
        reader.read_exact(&mut diversifier_bytes)?;
        let diversifier = Diversifier(diversifier_bytes);

        // To recover the note, read the value and r, and then use the payment address
        // to recreate the note
        let (value, rseed) = if version <= 3 {
            let value = reader.read_u64::<LittleEndian>()?;

            let mut r_bytes: [u8; 32] = [0; 32];
            reader.read_exact(&mut r_bytes)?;

            let r = jubjub::Fr::from_bytes(&r_bytes).unwrap();

            (value, Rseed::BeforeZip212(r))
        } else {
            let value = reader.read_u64::<LittleEndian>()?;
            let rseed = utils::read_rseed(&mut reader)?;

            (value, rseed)
        };

        let note = extfvk
            .fvk
            .vk
            .to_payment_address(diversifier)
            .unwrap()
            .create_note(value, rseed);

        let witnesses_vec = Vector::read(&mut reader, |r| IncrementalWitness::<Node>::read(r))?;
        let top_height = if version < 20 { 0 } else { reader.read_u64::<LittleEndian>()? };
        let witnesses = WitnessCache::new(witnesses_vec, top_height);

        let mut nullifier = [0u8; 32];
        reader.read_exact(&mut nullifier)?;
        let nullifier = sapling::Nullifier(nullifier);

        // Note that this is only the spent field, we ignore the unconfirmed_spent
        // field. The reason is that unconfirmed spents are only in memory, and
        // we need to get the actual value of spent from the blockchain anyway.
        let spent = if version <= 5 {
            let spent = Optional::read(&mut reader, |r| {
                let mut txid_bytes = [0u8; 32];
                r.read_exact(&mut txid_bytes)?;
                Ok(TxId::from_bytes(txid_bytes))
            })?;

            let spent_at_height =
                if version >= 2 { Optional::read(&mut reader, |r| r.read_i32::<LittleEndian>())? } else { None };

            if spent.is_some() && spent_at_height.is_some() {
                Some((spent.unwrap(), spent_at_height.unwrap() as u32))
            } else {
                None
            }
        } else {
            Optional::read(&mut reader, |r| {
                let mut txid_bytes = [0u8; 32];
                r.read_exact(&mut txid_bytes)?;
                let height = r.read_u32::<LittleEndian>()?;
                Ok((TxId::from_bytes(txid_bytes), height))
            })?
        };

        let unconfirmed_spent = if version <= 4 {
            None
        } else {
            Optional::read(&mut reader, |r| {
                let mut txbytes = [0u8; 32];
                r.read_exact(&mut txbytes)?;

                let height = r.read_u32::<LittleEndian>()?;
                Ok((TxId::from_bytes(txbytes), height))
            })?
        };

        let memo = Optional::read(&mut reader, |r| {
            let mut memo_bytes = [0u8; 512];
            r.read_exact(&mut memo_bytes)?;

            // Attempt to read memo, first as text, else as arbitrary 512 bytes
            match MemoBytes::from_bytes(&memo_bytes) {
                Ok(mb) => match Memo::try_from(mb.clone()) {
                    Ok(m) => Ok(m),
                    Err(_) => Ok(Memo::Future(mb)),
                },
                Err(e) => Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Couldn't create memo: {}", e))),
            }
        })?;

        let is_change: bool = reader.read_u8()? > 0;

        let have_spending_key = if version <= 2 {
            true // Will get populated in the lightwallet::read() method, for
                 // now assume true
        } else {
            reader.read_u8()? > 0
        };

        Ok(SaplingNoteData {
            extfvk,
            diversifier,
            note,
            witnesses,
            nullifier,
            spent,
            unconfirmed_spent,
            memo,
            is_change,
            have_spending_key,
        })
    }

    pub fn write<W: Write>(
        &self,
        mut writer: W,
    ) -> io::Result<()> {
        // Write a version number first, so we can later upgrade this if needed.
        writer.write_u64::<LittleEndian>(Self::serialized_version())?;

        self.extfvk.write(&mut writer)?;

        writer.write_all(&self.diversifier.0)?;

        // Writing the note means writing the note.value and note.r. The Note is
        // recoverable from these 2 values and the Payment address.
        writer.write_u64::<LittleEndian>(self.note.value().inner())?;

        utils::write_rseed(&mut writer, &self.note.rseed())?;

        Vector::write(&mut writer, &self.witnesses.witnesses, |wr, wi| wi.write(wr))?;
        writer.write_u64::<LittleEndian>(self.witnesses.top_height)?;

        writer.write_all(&self.nullifier.0)?;

        Optional::write(&mut writer, self.spent, |w, (txid, h)| {
            w.write_all(txid.as_ref())?;
            w.write_u32::<LittleEndian>(h)
        })?;

        Optional::write(&mut writer, self.unconfirmed_spent, |w, (txid, height)| {
            w.write_all(txid.as_ref())?;
            w.write_u32::<LittleEndian>(height)
        })?;

        Optional::write(&mut writer, self.memo.as_ref(), |w, m| w.write_all(m.encode().as_array()))?;

        writer.write_u8(if self.is_change { 1 } else { 0 })?;

        writer.write_u8(if self.have_spending_key { 1 } else { 0 })?;

        // Note that we don't write the unconfirmed_spent field, because if the wallet
        // is restarted, we don't want to be beholden to any expired txns

        Ok(())
    }
}

pub struct SpendableOrchardNote {
    pub txid: TxId,
    pub sk: orchard::keys::SpendingKey,
    pub note: orchard::Note,
    pub merkle_path: orchard::tree::MerklePath,
}

pub struct SpendableSaplingNote {
    pub txid: TxId,
    pub nullifier: sapling::Nullifier,
    pub diversifier: Diversifier,
    pub note: sapling::Note,
    pub witness: IncrementalWitness<Node>,
    pub extsk: ExtendedSpendingKey,
}

impl SpendableSaplingNote {
    pub fn from(
        txid: TxId,
        nd: &SaplingNoteData,
        anchor_offset: usize,
        extsk: &Option<ExtendedSpendingKey>,
    ) -> Option<Self> {
        // Include only notes that haven't been spent, or haven't been included in an
        // unconfirmed spend yet.
        if nd.spent.is_none()
            && nd.unconfirmed_spent.is_none()
            && extsk.is_some()
            && nd.witnesses.len() >= (anchor_offset + 1)
        {
            let witness = nd
                .witnesses
                .get(nd.witnesses.len() - anchor_offset - 1);

            witness.map(|w| SpendableSaplingNote {
                txid,
                nullifier: nd.nullifier,
                diversifier: nd.diversifier,
                note: nd.note.clone(),
                witness: w.clone(),
                extsk: extsk.clone().unwrap(),
            })
        } else {
            None
        }
    }
}
