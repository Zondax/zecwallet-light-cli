use std::io::{Read, Write};

use zcash_primitives::merkle_tree::HashSer;

pub(crate) mod data;
pub(crate) mod keys;
mod send_progress;
mod tests;
pub(crate) mod utils;
pub(crate) mod wallet;

// Enum to refer to the first or last position of the Node
pub enum NodePosition {
    Oldest,
    Highest,
}
