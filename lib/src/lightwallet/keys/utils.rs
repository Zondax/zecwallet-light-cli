use std::io;
use std::io::ErrorKind;

use base58::{FromBase58, ToBase58};
use hmac::digest::Digest;
use sha2::Sha256;

/// Sha256(Sha256(value))
pub fn double_sha256(payload: &[u8]) -> Vec<u8> {
    let h1 = Sha256::digest(payload);
    let h2 = Sha256::digest(&h1);
    h2.to_vec()
}

/// A trait for converting a [u8] to base58 encoded string.
pub trait ToBase58Check {
    /// Converts a value of `self` to a base58 value, returning the owned
    /// string. The version is a coin-specific prefix that is added.
    /// The suffix is any bytes that we want to add at the end (like the
    /// "iscompressed" flag for Secret key encoding)
    fn to_base58check(
        &self,
        version: &[u8],
        suffix: &[u8],
    ) -> String;
}

impl ToBase58Check for [u8] {
    fn to_base58check(
        &self,
        version: &[u8],
        suffix: &[u8],
    ) -> String {
        let mut payload: Vec<u8> = Vec::new();
        payload.extend_from_slice(version);
        payload.extend_from_slice(self);
        payload.extend_from_slice(suffix);

        let checksum = double_sha256(&payload);
        payload.append(&mut checksum[.. 4].to_vec());
        payload.to_base58()
    }
}

/// A trait for converting base58check encoded values.
pub trait FromBase58Check {
    /// Convert a value of `self`, interpreted as base58check encoded data, into
    /// the tuple with version and payload as bytes vector.
    fn from_base58check(&self) -> io::Result<(u8, Vec<u8>)>;
}

impl FromBase58Check for str {
    fn from_base58check(&self) -> io::Result<(u8, Vec<u8>)> {
        let mut payload: Vec<u8> = match self.from_base58() {
            Ok(payload) => payload,
            Err(error) => return Err(io::Error::new(ErrorKind::InvalidData, format!("{:?}", error))),
        };
        if payload.len() < 5 {
            return Err(io::Error::new(ErrorKind::InvalidData, "Invalid Checksum length".to_string()));
        }

        let checksum_index = payload.len() - 4;
        let provided_checksum = payload.split_off(checksum_index);
        let checksum = double_sha256(&payload)[.. 4].to_vec();
        if checksum != provided_checksum {
            return Err(io::Error::new(ErrorKind::InvalidData, "Invalid Checksum".to_string()));
        }
        Ok((payload[0], payload[1 ..].to_vec()))
    }
}
