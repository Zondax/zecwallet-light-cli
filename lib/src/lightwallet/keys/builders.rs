use std::sync::mpsc;

use async_trait::async_trait;
use derive_more::From;
use secp256k1::PublicKey as SecpPublicKey;
use thiserror::Error;
use zcash_primitives::consensus::Parameters;
use zcash_primitives::transaction::builder::Progress;
use zcash_primitives::{
    keys::OutgoingViewingKey,
    legacy::TransparentAddress,
    memo::MemoBytes,
    merkle_tree::MerklePath,
    sapling::{Diversifier, Node, Note, PaymentAddress, SaplingIvk},
    transaction::{
        components::{Amount, OutPoint, TxOut},
        Transaction,
    },
};

use crate::lightwallet::keys::in_memory::builder::BuilderError as InMemoryBuilderError;
use crate::lightwallet::keys::in_memory::InMemoryBuilder;
use crate::lightwallet::keys::ledger::{LedgerBuilder, LedgerError};

cfg_if::cfg_if! {
    if #[cfg(feature = "hsm-compat")] {
        mod txprover_trait {
            use zcash_primitives::sapling::prover::TxProver;
            use zcash_hsmbuilder::txprover::HsmTxProver;

            /// This trait is a marker trait used to identify tx provers
            /// that are HSM compatible as well as normal tx provers
            ///
            /// Automatically implemented by a type if the constraits are satisfied
            /// via blanket impl
            pub trait BothTxProver: TxProver + HsmTxProver {}

            impl<T: TxProver + HsmTxProver> BothTxProver for T {}
        }

        pub use txprover_trait::BothTxProver as TxProver;
        pub use zcash_hsmbuilder::txbuilder::SaplingMetadata;
    } else {
        pub use zcash_primitives::sapling::prover::TxProver;
        pub use zcash_primitives::transaction::builder::SaplingMetadata;
    }
}

/// This trait represents the functionality that a ZCash transaction builder
/// should expose
///
/// Will be used as common interface between
/// [`zcash_primitives::transaction::builder::Builder`] and other builders
#[async_trait::async_trait]
pub trait Builder {
    type Error;

    fn add_sapling_spend(
        &mut self,
        key: &SaplingIvk,
        diversifier: Diversifier,
        note: Note,
        merkle_path: MerklePath<Node>,
    ) -> Result<&mut Self, Self::Error>;

    fn add_sapling_output(
        &mut self,
        ovk: Option<OutgoingViewingKey>,
        to: PaymentAddress,
        value: Amount,
        memo: MemoBytes,
    ) -> Result<&mut Self, Self::Error>;

    fn add_transparent_input(
        &mut self,
        key: SecpPublicKey,
        utxo: OutPoint,
        coin: TxOut,
    ) -> Result<&mut Self, Self::Error>;

    fn add_transparent_output(
        &mut self,
        to: &TransparentAddress,
        value: Amount,
    ) -> Result<&mut Self, Self::Error>;

    fn send_change_to(
        &mut self,
        ovk: OutgoingViewingKey,
        to: PaymentAddress,
    ) -> &mut Self;

    /// Sets the notifier channel, where progress of building the transaction is
    /// sent.
    ///
    /// An update is sent after every Spend or Output is computed, and the `u32`
    /// sent represents the total steps completed so far. It will eventually
    /// send number of spends + outputs. If there's an error building the
    /// transaction, the channel is closed.
    fn with_progress_notifier(
        &mut self,
        progress_notifier: Option<mpsc::Sender<Progress>>,
    );

    /// This will take care of building the transaction with the inputs given so
    /// far
    ///
    /// The `progress` is an optional argument for a mpsc channel to allow the
    /// builder to send the number of items processed so far
    async fn build(
        mut self,
        prover: &(impl TxProver + Send + Sync),
        fee: u64,
    ) -> Result<(Transaction, SaplingMetadata), Self::Error>;
}

#[derive(From)]
/// Enum based dispatcher for different transaction builders
///
/// Should be instantiated with [`Keystores::tx_builder`]
pub enum Builders<'ks, P: Parameters> {
    Memory(InMemoryBuilder<'ks, P>),
    #[cfg(feature = "ledger-support")]
    Ledger(LedgerBuilder<'ks, P>),
}

#[derive(Debug, Error)]
pub enum BuildersError {
    #[error(transparent)]
    Memory(#[from] InMemoryBuilderError),
    #[cfg(feature = "ledger-support")]
    #[error(transparent)]
    Ledger(#[from] LedgerError),
}

#[async_trait]
impl<'ks, P: Parameters + Send + Sync + 'static> Builder for Builders<'ks, P> {
    type Error = BuildersError;

    fn add_sapling_spend(
        &mut self,
        key: &SaplingIvk,
        diversifier: zcash_primitives::sapling::Diversifier,
        note: Note,
        merkle_path: zcash_primitives::merkle_tree::MerklePath<zcash_primitives::sapling::Node>,
    ) -> Result<&mut Self, Self::Error> {
        match self {
            Self::Memory(this) => this
                .add_sapling_spend(key, diversifier, note, merkle_path)
                .map(|_| ())?,
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => this
                .add_sapling_spend(key, diversifier, note, merkle_path)
                .map(|_| ())?,
        };

        Ok(self)
    }

    fn add_sapling_output(
        &mut self,
        ovk: Option<OutgoingViewingKey>,
        to: PaymentAddress,
        value: zcash_primitives::transaction::components::Amount,
        memo: zcash_primitives::memo::MemoBytes,
    ) -> Result<&mut Self, Self::Error> {
        match self {
            Self::Memory(this) => this
                .add_sapling_output(ovk, to, value, memo)
                .map(|_| ())?,
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => this
                .add_sapling_output(ovk, to, value, memo)
                .map(|_| ())?,
        };

        Ok(self)
    }

    fn add_transparent_input(
        &mut self,
        key: SecpPublicKey,
        utxo: zcash_primitives::transaction::components::OutPoint,
        coin: zcash_primitives::transaction::components::TxOut,
    ) -> Result<&mut Self, Self::Error> {
        match self {
            Self::Memory(this) => this
                .add_transparent_input(key, utxo, coin)
                .map(|_| ())?,
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => this
                .add_transparent_input(key, utxo, coin)
                .map(|_| ())?,
        };

        Ok(self)
    }

    fn add_transparent_output(
        &mut self,
        to: &TransparentAddress,
        value: zcash_primitives::transaction::components::Amount,
    ) -> Result<&mut Self, Self::Error> {
        match self {
            Self::Memory(this) => this
                .add_transparent_output(to, value)
                .map(|_| ())?,
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => this
                .add_transparent_output(to, value)
                .map(|_| ())?,
        };

        Ok(self)
    }

    fn send_change_to(
        &mut self,
        ovk: OutgoingViewingKey,
        to: PaymentAddress,
    ) -> &mut Self {
        match self {
            Self::Memory(this) => {
                this.send_change_to(ovk, to);
            },
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => {
                this.send_change_to(ovk, to);
            },
        };

        self
    }

    fn with_progress_notifier(
        &mut self,
        progress_notifier: Option<mpsc::Sender<Progress>>,
    ) {
        match self {
            Self::Memory(this) => this.with_progress_notifier(progress_notifier),
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => this.with_progress_notifier(progress_notifier),
        }
    }

    async fn build(
        mut self,
        prover: &(impl TxProver + Send + Sync),
        fee: u64,
    ) -> Result<(Transaction, SaplingMetadata), Self::Error> {
        match self {
            Self::Memory(this) => this
                .build(prover, fee)
                .await
                .map_err(Into::into),
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => this
                .build(prover, fee)
                .await
                .map_err(Into::into),
        }
    }
}
