#[macro_use]
extern crate rust_embed;

pub use commands::do_user_command;
use lazy_static::lazy_static;
use tokio::runtime::Runtime;
pub use zcash_primitives::consensus::{MainNetwork, Parameters};

mod commands;
pub mod grpc;
pub mod lightclient;
pub mod lightwallet;
mod utils;
#[cfg(feature = "embed_params")]
#[derive(RustEmbed)]
#[folder = "zcash-params/"]
pub struct SaplingParams;

#[derive(RustEmbed)]
#[folder = "pubkey/"]
pub struct ServerCert;

lazy_static! {
    static ref RT: Runtime = tokio::runtime::Runtime::new().unwrap();
}
