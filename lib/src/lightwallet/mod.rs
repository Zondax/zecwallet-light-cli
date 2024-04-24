use std::convert::TryInto;
use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt};
use futures::Future;
use incrementalmerkletree::Hashable;
use incrementalmerkletree::Tree;
use zcash_address::unified::Encoding;
use zcash_primitives::merkle_tree::HashSer;
use zcash_primitives::sapling::prover::TxProver;

pub(crate) mod data;
mod extended_key;
pub(crate) mod keys;
pub(crate) mod lightwallet;
pub(crate) mod message;
pub(crate) mod options;
mod send_progress;
mod tests;
pub(crate) mod utils;
pub(crate) mod wallet_txns;
pub mod walletkeys;

// Enum to refer to the first or last position of the Node
pub enum NodePosition {
    Oldest,
    Highest,
}
