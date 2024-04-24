use ripemd160::Digest;
use zcash_primitives::{
    consensus::BlockHeight,
    sapling::PaymentAddress,
    zip32::{ChildIndex, ExtendedSpendingKey},
};

mod builders;
pub use builders::{Builder, SaplingMetadata, TxProver};

// GAT workaround, see:
// https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats#the-better-gats
mod sealed {
    pub trait Sealed: Sized {}
    pub struct Bounds<T>(T);
    impl<T> Sealed for Bounds<T> {}
}

pub trait KeystoreBuilderLifetime<'this, ImplicitBounds: sealed::Sealed = sealed::Bounds<&'this Self>> {
    type Builder: builders::Builder;
}

#[async_trait::async_trait]
pub trait Keystore
where
    Self: for<'this> KeystoreBuilderLifetime<'this>,
{
    type Error;

    /// Retrieve the unshielded public key for a given path
    async fn get_t_pubkey(
        &self,
        path: &[ChildIndex],
    ) -> Result<secp256k1::PublicKey, Self::Error>;

    /// Retrieve the shielded payment address for a given path
    async fn get_z_payment_address(
        &self,
        path: &[ChildIndex],
    ) -> Result<PaymentAddress, Self::Error>;

    /// Retrieve an initialized builder for the current keystore
    fn txbuilder(
        &mut self,
        target_height: BlockHeight,
    ) -> Result<<Self as KeystoreBuilderLifetime<'_>>::Builder, Self::Error>;
}

#[async_trait::async_trait]
pub trait InsecureKeystore {
    type Error;

    /// Retrieve bip39 seed phrase used in key generation
    #[allow(dead_code)]
    async fn get_seed_phrase(&self) -> Result<String, Self::Error>;

    /// Retrieve the shielded spending key for a given path
    async fn get_z_private_spending_key(
        &self,
        path: &[ChildIndex],
    ) -> Result<ExtendedSpendingKey, Self::Error>;

    /// Retrieve the unshielded secret key for a given path
    async fn get_t_secret_key(
        &self,
        path: &[ChildIndex],
    ) -> Result<secp256k1::SecretKey, Self::Error>;
}

pub use in_memory::InMemoryKeys;

#[cfg(feature = "ledger-support")]
mod ledger;
#[cfg(feature = "ledger-support")]
pub use ledger::LedgerKeystore;

pub mod data;
pub mod extended_key;
mod in_memory;
pub(crate) mod keystores;
pub(crate) mod utils;
