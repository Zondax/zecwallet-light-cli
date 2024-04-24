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

// pub mod blaze;
// pub mod compact_formats;
// pub mod grpc_connector;
// pub mod lightclient;
// pub mod lightwallet;

// use lightclient::LightClient;

// fn main() {
//     let seed = std::fs::read_to_string("./testdata/seed.txt").unwrap();
//     let lc = LightClient::new(Some(seed)).unwrap();
//     lc.start_sync();
// }
