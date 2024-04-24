use std::io;
use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt};
use zcash_primitives::sapling::Rseed;

// Reading a note also needs the corresponding address to read from.
pub fn read_rseed<R: Read>(mut reader: R) -> io::Result<Rseed> {
    let note_type = reader.read_u8()?;

    let mut r_bytes: [u8; 32] = [0; 32];
    reader.read_exact(&mut r_bytes)?;

    // todo: the note type should have been an enum for more clarity?
    let r = match note_type {
        1 => Rseed::BeforeZip212(jubjub::Fr::from_bytes(&r_bytes).unwrap()),
        2 => Rseed::AfterZip212(r_bytes),
        _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Bad note type")),
    };

    Ok(r)
}

pub fn write_rseed<W: Write>(
    mut writer: W,
    rseed: &Rseed,
) -> io::Result<()> {
    let note_type = match rseed {
        Rseed::BeforeZip212(_) => 1,
        Rseed::AfterZip212(_) => 2,
    };
    writer.write_u8(note_type)?;

    match rseed {
        Rseed::BeforeZip212(fr) => writer.write_all(&fr.to_bytes()),
        Rseed::AfterZip212(b) => writer.write_all(b),
    }
}
