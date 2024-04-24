use std::io;
use std::io::{Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

#[derive(Debug, Clone, Copy)]
pub struct WalletOptions {
    pub(crate) download_memos: MemoDownloadOption,
    pub(crate) spam_threshold: i64,
}

impl Default for WalletOptions {
    fn default() -> Self {
        WalletOptions { download_memos: MemoDownloadOption::WalletMemos, spam_threshold: -1 }
    }
}

impl WalletOptions {
    pub fn serialized_version() -> u64 {
        return 2;
    }

    pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let version = reader.read_u64::<LittleEndian>()?;

        let download_memos = match reader.read_u8()? {
            0 => MemoDownloadOption::NoMemos,
            1 => MemoDownloadOption::WalletMemos,
            2 => MemoDownloadOption::AllMemos,
            v => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Bad download option {}", v)));
            },
        };

        let spam_threshold = if version <= 1 { -1 } else { reader.read_i64::<LittleEndian>()? };

        Ok(Self { download_memos, spam_threshold })
    }

    pub fn write<W: Write>(
        &self,
        mut writer: W,
    ) -> io::Result<()> {
        // Write the version
        writer.write_u64::<LittleEndian>(Self::serialized_version())?;

        writer.write_u8(self.download_memos as u8)?;

        writer.write_i64::<LittleEndian>(self.spam_threshold)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoDownloadOption {
    NoMemos = 0,
    WalletMemos,
    AllMemos,
}
