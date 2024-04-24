use std::io;
use std::io::{Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use prost::Message;
use zcash_encoding::Vector;
use zcash_primitives::merkle_tree::CommitmentTree;
use zcash_primitives::sapling::Node;

use crate::grpc::CompactBlock;

#[derive(Clone)]
// todo: this should be part of the light client
pub struct BlockData {
    pub(crate) ecb: Vec<u8>,
    pub height: u64,
}

impl BlockData {
    pub fn serialized_version() -> u64 {
        20
    }

    pub(crate) fn new_with(
        height: u64,
        hash: &str,
    ) -> Self {
        let mut cb = CompactBlock::default();
        cb.hash = hex::decode(hash)
            .unwrap()
            .into_iter()
            .rev()
            .collect::<Vec<_>>();

        let mut ecb = vec![];
        cb.encode(&mut ecb).unwrap();

        Self { ecb, height }
    }

    pub(crate) fn new(mut cb: CompactBlock) -> Self {
        for ctx in &mut cb.vtx {
            for co in &mut ctx.outputs {
                co.ciphertext.clear();
                co.epk.clear();
            }
        }

        cb.header.clear();
        let height = cb.height;

        let mut ecb = vec![];
        cb.encode(&mut ecb).unwrap();

        Self { ecb, height }
    }

    pub(crate) fn cb(&self) -> CompactBlock {
        let b = self.ecb.clone();
        CompactBlock::decode(&b[..]).unwrap()
    }

    pub(crate) fn hash(&self) -> String {
        self.cb().hash().to_string()
    }

    pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let height = reader.read_i32::<LittleEndian>()? as u64;

        let mut hash_bytes = [0; 32];
        reader.read_exact(&mut hash_bytes)?;
        hash_bytes.reverse();
        let hash = hex::encode(hash_bytes);

        // We don't need this, but because of a quirk, the version is stored later, so
        // we can't actually detect the version here. So we write an empty tree
        // and read it back here
        let tree = CommitmentTree::<Node>::read(&mut reader)?;
        let _tree = if tree.size() == 0 { None } else { Some(tree) };

        let version = reader.read_u64::<LittleEndian>()?;

        let ecb = if version <= 11 { vec![] } else { Vector::read(&mut reader, |r| r.read_u8())? };

        if ecb.is_empty() {
            Ok(BlockData::new_with(height, hash.as_str()))
        } else {
            Ok(BlockData { ecb, height })
        }
    }

    pub fn write<W: Write>(
        &self,
        mut writer: W,
    ) -> io::Result<()> {
        writer.write_i32::<LittleEndian>(self.height as i32)?;

        let hash_bytes: Vec<_> = hex::decode(self.hash())
            .unwrap()
            .into_iter()
            .rev()
            .collect();
        writer.write_all(&hash_bytes[..])?;

        CommitmentTree::<Node>::empty().write(&mut writer)?;
        writer.write_u64::<LittleEndian>(Self::serialized_version())?;

        // Write the ecb as well
        Vector::write(&mut writer, &self.ecb, |w, b| w.write_u8(*b))?;

        Ok(())
    }
}
