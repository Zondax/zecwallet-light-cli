use std::collections::HashMap;
use std::io;
use std::io::ErrorKind;

use derive_more::From;
use jubjub::AffinePoint;
use orchard::keys::{FullViewingKey, IncomingViewingKey};
use secp256k1::PublicKey as SecpPublicKey;
use zcash_client_backend::encoding::encode_payment_address;
use zcash_primitives::consensus::{BlockHeight, Parameters};
use zcash_primitives::keys::OutgoingViewingKey;
use zcash_primitives::legacy::TransparentAddress;
use zcash_primitives::sapling::{Note, Nullifier, PaymentAddress, SaplingIvk};
use zcash_primitives::zip32::{ExtendedFullViewingKey, ExtendedSpendingKey};

use crate::lightclient::config::LightClientConfig;
use crate::lightwallet::keys::builders::Builders;
use crate::lightwallet::keys::utils::ToBase58Check;
use crate::lightwallet::keys::{InMemoryKeys, Keystore, LedgerKeystore};

pub enum KeystoresKind {
    Memory,
    #[cfg(feature = "ledger-support")]
    Ledger,
}

#[derive(From)]
/// Provide enum-based dispatch to different keystores
pub enum Keystores<P> {
    Memory(InMemoryKeys<P>),
    #[cfg(feature = "ledger-support")]
    Ledger(LedgerKeystore<P>),
}

impl<P: Parameters> Keystores<P> {
    pub fn in_memory(&self) -> Result<&InMemoryKeys<P>, io::Error> {
        match self {
            Self::Memory(this) => Ok(this),
            _ => Err(io::Error::new(ErrorKind::Unsupported, "incompatible keystore requested")),
        }
    }

    pub fn in_memory_mut(&mut self) -> Result<&mut InMemoryKeys<P>, io::Error> {
        match self {
            Self::Memory(this) => Ok(this),
            _ => Err(io::Error::new(ErrorKind::Unsupported, "incompatible keystore requested")),
        }
    }
}

#[cfg(feature = "ledger-support")]
impl<P: Parameters> Keystores<P> {
    pub fn ledger(&self) -> Result<&LedgerKeystore<P>, io::Error> {
        match self {
            Self::Ledger(this) => Ok(this),
            _ => Err(io::Error::new(ErrorKind::Unsupported, "incompatible keystore requested")),
        }
    }

    pub fn ledger_mut(&mut self) -> Result<&mut LedgerKeystore<P>, io::Error> {
        match self {
            Self::Ledger(this) => Ok(this),
            _ => Err(io::Error::new(ErrorKind::Unsupported, "incompatible keystore requested")),
        }
    }
}

impl<P: Parameters + Send + Sync + 'static> Keystores<P> {
    pub fn as_kind(&self) -> KeystoresKind {
        match self {
            Self::Memory(_) => KeystoresKind::Memory,
            #[cfg(feature = "ledger-support")]
            Self::Ledger(_) => KeystoresKind::Ledger,
        }
    }

    pub fn config(&self) -> LightClientConfig<P> {
        match self {
            Self::Memory(this) => this.config(),
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => this.config.clone(),
        }
    }

    pub async fn get_all_extfvks(&self) -> impl Iterator<Item = ExtendedFullViewingKey> {
        std::iter::once_with(|| todo!("not implemented"))
    }

    pub fn get_orchard_sk_for_fvk(
        &self,
        _fvk: &orchard::keys::FullViewingKey,
    ) -> Option<orchard::keys::SpendingKey> {
        todo!("not implemented")
    }

    pub async fn get_extsk_for_extfvk(
        &self,
        _extfvk: &ExtendedFullViewingKey,
    ) -> impl Iterator<Item = ExtendedSpendingKey> {
        std::iter::once_with(|| todo!("not implemented"))
    }

    // /// Retrieve all known IVKs in the keystore
    // pub async fn get_all_ivks(&self) -> impl Iterator<Item = SaplingIvk> {
    //     // this is some hard to read rust trickery, but in short we are
    //     // an iterator for all the ivks in the keystore
    //     // using options and iterator methods to unify the type to return into 1,
    //     // so we can use `impl` and not `dyn`
    //     //
    //     // To add future variants (if ever), add an element to the tuple,
    //     // set it to `None` on all branches except the new one where the
    //     // new variant is matched against
    //     //
    //     // Finally, add `.chain(new_tuple_item.into_iter().flatten())`
    //     // at the bottom expression
    //
    //     let (memory, ledger) = match self {
    //         Self::Memory(this) => (
    //             Some(
    //                 this.get_all_extfvks()
    //                     .into_iter()
    //                     .map(|key| key.fvk.vk.ivk()),
    //             ),
    //             None,
    //         ),
    //         #[cfg(feature = "ledger-support")]
    //         Self::Ledger(this) => (
    //             None,
    //             Some(
    //                 this.get_all_sapling_ivks()
    //                     .await
    //                     .map(|(ivk, _)| ivk),
    //             ),
    //         ),
    //     };
    //
    //     memory
    //         .into_iter()
    //         .flatten()
    //         .chain(ledger.into_iter().flatten())
    // }

    pub async fn get_sapling_extfvk(
        &self,
        idx: usize,
    ) -> Option<ExtendedFullViewingKey> {
        match self {
            Self::Memory(memory_keys) => memory_keys
                .zkeys
                .get(idx)
                .map(|zk| zk.extfvk().clone()),
            #[cfg(feature = "ledger-support")]
            Self::Ledger(_ledger_keys) => {
                todo!("not implemented")
            },
        }
    }

    pub async fn get_orchard_fvk(
        &self,
        idx: usize,
    ) -> Option<FullViewingKey> {
        match self {
            Self::Memory(memory_keys) => memory_keys
                .okeys
                .get(idx)
                .map(|zk| zk.fvk().clone()),
            #[cfg(feature = "ledger-support")]
            Self::Ledger(_ledger_keys) => {
                todo!("not implemented")
            },
        }
    }

    pub async fn get_all_sapling_ivks(&self) -> Vec<SaplingIvk> {
        match self {
            Self::Memory(memory_keys) => memory_keys
                .zkeys
                .iter()
                .map(|zk| zk.extfvk().fvk.vk.ivk())
                .collect(),
            #[cfg(feature = "ledger-support")]
            Self::Ledger(ledger_keys) => {
                // Retrieve Sapling IVKs from the Ledger keystore, filtering out the diversifier
                ledger_keys
                    .get_all_sapling_ivks()
                    .await
                    .map(|(ivk, _)| ivk)
                    .collect()
            },
        }
    }
    pub async fn get_all_orchard_ivks(&self) -> impl Iterator<Item = IncomingViewingKey> {
        std::iter::once_with(|| todo!("not implemented"))
    }

    /// Retrieve all known OVKs in the keystore
    pub async fn get_all_ovks(&self) -> impl Iterator<Item = OutgoingViewingKey> {
        // see comment inside `get_all_ivks`

        let (memory, ledger) = match self {
            Self::Memory(this) => (
                Some(
                    this.get_all_extfvks()
                        .into_iter()
                        .map(|k| k.fvk.ovk),
                ),
                None,
            ),
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => (None, Some(this.get_all_ovks().await)),
        };

        memory
            .into_iter()
            .flatten()
            .chain(ledger.into_iter().flatten())
    }

    /// Retrieve all known transparent addresses in the keystore
    pub async fn get_all_taddrs(&self) -> impl Iterator<Item = String> {
        // see comment inside `get_all_ivks`

        let (memory, ledger) = match self {
            Self::Memory(this) => (Some(this.get_all_taddrs().into_iter()), None),
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => (None, Some(this.get_all_taddrs().await)),
        };

        memory
            .into_iter()
            .flatten()
            .chain(ledger.into_iter().flatten())
    }

    /// Retrieve all known ZAddrs in the keystore
    pub async fn get_all_zaddresses(&self) -> impl Iterator<Item = String> + '_ {
        // see comment inside `get_all_ivks`

        let (memory, ledger) = match self {
            Self::Memory(this) => (Some(this.get_all_zaddresses().into_iter()), None),
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => (None, Some(this.get_all_zaddresses().await)),
        };

        memory
            .into_iter()
            .flatten()
            .chain(ledger.into_iter().flatten())
    }

    /// Retrieve all known UAddrs in the keystore
    pub async fn get_all_uaddresses(&self) -> impl Iterator<Item = String> + '_ {
        std::iter::empty()
    }

    /// Retrieve all ZAddrs in the keystore which we have the spending key for
    pub async fn get_all_spendable_zaddresses(&self) -> impl Iterator<Item = String> + '_ {
        // see comment inside `get_all_ivks`

        let (memory, ledger) = match self {
            Self::Memory(this) => (
                Some(
                    this.get_all_spendable_zaddresses()
                        .into_iter(),
                ),
                None,
            ),
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => (None, Some(this.get_all_zaddresses().await)),
        };

        memory
            .into_iter()
            .flatten()
            .chain(ledger.into_iter().flatten())
    }

    pub async fn get_all_spendable_oaddresses(&self) -> impl Iterator<Item = String> + '_ {
        std::iter::once_with(|| todo!("not implemented"))
    }

    // /// Retrieve all IVKs in the keystore which we have the spending key for
    // pub async fn get_all_spendable_ivks(&self) -> impl Iterator<Item =
    // SaplingIvk> {     // see comment inside `get_all_ivks`
    //
    //     let (memory, ledger) = match self {
    //         Self::Memory(this) => (
    //             Some(
    //                 this.get_all_extfvks()
    //                     .into_iter()
    //                     .map(|extfvk| extfvk.fvk.vk.ivk())
    //                     .filter(|key| this.have_sapling_spending_key(&key))
    //                     // we collect to avoid borrowing this
    //                     // and causing lifetime issues
    //                     .collect::<Vec<_>>()
    //                     .into_iter(),
    //             ),
    //             None,
    //         ),
    //         #[cfg(feature = "ledger-support")]
    //         // with the ledger all known ivks are spendable
    //         Self::Ledger(this) => (
    //             None,
    //             Some(
    //                 this.get_all_ivks()
    //                     .await
    //                     .map(|(ivk, _)| ivk),
    //             ),
    //         ),
    //     };
    //
    //     memory
    //         .into_iter()
    //         .flatten()
    //         .chain(ledger.into_iter().flatten())
    // }

    /// Retrieve a HashMap to lookup a public key from the transparent address
    pub async fn get_taddr_to_key_map(&self) -> HashMap<String, SecpPublicKey> {
        match self {
            Self::Memory(this) => this.get_taddr_to_key_map(),
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => this.taddr_to_key_map().await,
        }
    }

    /// Retrieve the transaction builder for the keystore
    pub fn tx_builder(
        &mut self,
        target_height: BlockHeight,
    ) -> Builders<'_, P> {
        match self {
            Self::Memory(this) => this
                .txbuilder(target_height)
                .expect("infallible")
                .into(),
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => this
                .txbuilder(target_height)
                .expect("infallible")
                .into(),
        }
    }

    /// Returns the first stored shielded OVK and payment address of the
    /// keystore
    pub async fn first_zkey(&self) -> Option<(OutgoingViewingKey, PaymentAddress)> {
        match self {
            Self::Memory(this) => this
                .zkeys
                .first()
                .map(|zk| (zk.extfvk.fvk.ovk, zk.zaddress.clone())),
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => {
                let path = this.first_shielded().await?;
                let ovk = this.get_ovk_of(&path).await?;
                let zaddr = this
                    .payment_address_from_path(&path)
                    .await
                    .expect("path must have been cached already");
                Some((ovk, zaddr))
            },
        }
    }

    /// Compute whether the given `addr` is a shielded address w.r.t. the given
    /// set of params
    pub fn is_shielded_address(
        addr: &String,
        params: &P,
    ) -> bool {
        use zcash_client_backend::address::RecipientAddress;

        match RecipientAddress::decode(params, addr) {
            Some(RecipientAddress::Shielded(_)) => true,
            _ => false,
        }
    }

    /// Perform bech32 encoding of the given address
    pub fn encode_zaddr(
        &self,
        addr: PaymentAddress,
    ) -> String {
        let config = match self {
            Keystores::Memory(this) => this.config(),
            #[cfg(feature = "ledger-support")]
            Keystores::Ledger(this) => this.config(),
        };
        let hrp = config.hrp_sapling_address();
        encode_payment_address(hrp, &addr)
    }

    /// Compute the transparent address of a pubkey hash
    pub fn address_from_pubkeyhash(
        &self,
        addr: Option<TransparentAddress>,
    ) -> Option<String> {
        let prefix = match self {
            Keystores::Memory(this) => this.config().base58_pubkey_address(),
            #[cfg(feature = "ledger-support")]
            Keystores::Ledger(this) => this.config.base58_pubkey_address(),
        };

        match addr {
            Some(TransparentAddress::PublicKey(hash)) | Some(TransparentAddress::Script(hash)) => {
                Some(hash.to_base58check(&prefix, &[]))
            },
            _ => None,
        }
    }

    /// Ensure we have available N generated keys via HD after the given address
    ///
    /// This is for the transparent addresses
    pub async fn ensure_hd_taddresses(
        &mut self,
        addr: &str,
    ) {
        match self {
            Keystores::Memory(this) => this.ensure_hd_taddresses(addr),
            #[cfg(feature = "ledger-support")]
            Keystores::Ledger(this) => this.ensure_hd_taddresses(addr).await,
        }
    }

    /// Ensure we have available N generated keys via HD after the given address
    ///
    /// This is for the shielded addresses
    pub async fn ensure_hd_zaddresses(
        &mut self,
        addr: &str,
    ) {
        match self {
            Self::Memory(this) => this.ensure_hd_zaddresses(addr),
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => this.ensure_hd_zaddresses(addr).await,
        }
    }

    /// Ensure we have the spending key of the given viewing key in the keystore
    pub async fn have_sapling_spending_key(
        &self,
        extfvk: &ExtendedFullViewingKey,
    ) -> bool {
        match self {
            Self::Memory(this) => this.have_sapling_spending_key(extfvk),
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => {
                let ivks = this.get_all_sapling_ivks().await;
                ivks.into_iter()
                    .any(|(ivk, _)| ivk.to_repr() == extfvk.fvk.vk.ivk().to_repr())
            },
        }
    }

    /// Ensure we have the spending key of the given viewing key in the keystore
    pub async fn have_orchard_spending_key(
        &self,
        _ivk: &FullViewingKey,
    ) -> bool {
        todo!("not implemented")
    }

    /// Create a new transparent address
    pub async fn add_taddr(&mut self) -> String {
        match self {
            Self::Memory(this) => this.add_taddr(),
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => this.add_taddr().await,
        }
    }

    /// Create a new shielded address
    pub async fn add_zaddr(&mut self) -> String {
        match self {
            Self::Memory(this) => this.add_zaddr(),
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => this.add_zaddr().await,
        }
    }

    pub async fn add_oaddr(&mut self) -> String {
        todo!("not implemented")
    }

    // this is the same code as Note's cm_full_point
    fn compute_note_commitment(note: &Note) -> jubjub::SubgroupPoint {
        use byteorder::{LittleEndian, WriteBytesExt};
        use group::GroupEncoding;
        use zcash_primitives::{
            constants::NOTE_COMMITMENT_RANDOMNESS_GENERATOR,
            sapling::pedersen_hash::{pedersen_hash, Personalization},
        };

        // Calculate the note contents, as bytes
        let mut note_contents = vec![];

        // Writing the value in little endian
        note_contents
            .write_u64::<LittleEndian>(note.value)
            .unwrap();

        // Write g_d
        note_contents.extend_from_slice(&note.g_d.to_bytes());

        // Write pk_d
        note_contents.extend_from_slice(&note.pk_d.to_bytes());

        assert_eq!(note_contents.len(), 32 + 32 + 8);

        // Compute the Pedersen hash of the note contents
        let hash_of_contents = pedersen_hash(
            Personalization::NoteCommitment,
            note_contents
                .into_iter()
                .flat_map(|byte| (0 .. 8).map(move |i| ((byte >> i) & 1) == 1)),
        );

        // Compute final commitment
        (NOTE_COMMITMENT_RANDOMNESS_GENERATOR * note.rcm()) + hash_of_contents
    }

    /// Compute the note nullifier
    pub async fn get_note_nullifier(
        &self,
        ivk: &SaplingIvk,
        position: u64,
        note: &Note,
    ) -> Result<Nullifier, String> {
        match self {
            Self::Memory(this) => {
                let extfvk = this
                    .get_all_extfvks()
                    .into_iter()
                    .find(|extfvk| extfvk.fvk.vk.ivk().to_repr() == ivk.to_repr())
                    .ok_or("Error: unknown key".to_string())?;

                Ok(note.nf(&extfvk.fvk.vk.nk, position))
            },
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => {
                let commitment = Self::compute_note_commitment(note);
                let commitment: &jubjub::ExtendedPoint = (&commitment).into();

                this.compute_nullifier(ivk, position, AffinePoint::from(commitment))
                    .await
                    .map_err(|e| format!("Error: unable to compute note nullifier: {:?}", e))
            },
        }
    }
}

// serialization stuff
impl<P: Parameters + Send + Sync + 'static> Keystores<P> {
    /// Indicates whether the keystore is ready to be saved to file
    pub fn writable(&self) -> bool {
        match self {
            Self::Memory(this) => !(this.encrypted && this.unlocked),
            #[cfg(feature = "ledger-support")]
            Self::Ledger(_) => true,
        }
    }

    /// Serialize keystore to writer
    pub async fn write<W: io::Write>(
        &self,
        mut writer: W,
    ) -> io::Result<()> {
        use byteorder::WriteBytesExt;

        match self {
            Self::Memory(this) => {
                writer.write_u8(0)?;
                this.write(writer)
            },
            #[cfg(feature = "ledger-support")]
            Self::Ledger(this) => {
                writer.write_u8(1)?;
                this.write(writer).await
            },
        }
    }

    /// Deserialize keystore from reader
    pub async fn read<R: io::Read>(
        mut reader: R,
        config: &LightClientConfig<P>,
    ) -> io::Result<Self> {
        use byteorder::ReadBytesExt;
        let variant = reader.read_u8()?;

        match variant {
            0 => InMemoryKeys::<P>::read(reader, config).map(Into::into),
            #[cfg(feature = "ledger-support")]
            1 => LedgerKeystore::read(reader, config)
                .await
                .map(Into::into),
            _ => Err(io::Error::new(ErrorKind::InvalidData, "Unknown keystore variant".to_string())),
        }
    }
}
