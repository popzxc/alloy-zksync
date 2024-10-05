use alloy_consensus::SignableTransaction;
use alloy_network::{Network, NetworkWallet, TxSigner};
use alloy_primitives::Address;

use crate::network::{tx_envelope::TxEnvelope, unsigned_tx::TypedTransaction, Zksync};

use alloy_signer::Signature;
use async_trait::async_trait;
use std::{collections::BTreeMap, sync::Arc};

/// A wallet capable of signing any transaction for the Ethereum network.
#[derive(Clone, Default)]
pub struct ZksyncWallet {
    default: Address,
    signers: BTreeMap<Address, Arc<dyn TxSigner<Signature> + Send + Sync>>,
}

impl std::fmt::Debug for ZksyncWallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZksyncWallet")
            .field("default_signer", &self.default)
            .field("credentials", &self.signers.len())
            .finish()
    }
}

impl<S> From<S> for ZksyncWallet
where
    S: TxSigner<Signature> + Send + Sync + 'static,
{
    fn from(signer: S) -> Self {
        Self::new(signer)
    }
}

impl ZksyncWallet {
    /// Create a new signer with the given signer as the default signer.
    pub fn new<S>(signer: S) -> Self
    where
        S: TxSigner<Signature> + Send + Sync + 'static,
    {
        let mut this = Self::default();
        this.register_default_signer(signer);
        this
    }

    /// Register a new signer on this object. This signer will be used to sign
    /// [`TransactionRequest`] and [`TypedTransaction`] object that specify the
    /// signer's address in the `from` field.
    ///
    /// [`TransactionRequest`]: alloy_rpc_types_eth::TransactionRequest
    pub fn register_signer<S>(&mut self, signer: S)
    where
        S: TxSigner<Signature> + Send + Sync + 'static,
    {
        self.signers.insert(signer.address(), Arc::new(signer));
    }

    /// Register a new signer on this object, and set it as the default signer.
    /// This signer will be used to sign [`TransactionRequest`] and
    /// [`TypedTransaction`] objects that do not specify a signer address in the
    /// `from` field.
    ///
    /// [`TransactionRequest`]: alloy_rpc_types_eth::TransactionRequest
    pub fn register_default_signer<S>(&mut self, signer: S)
    where
        S: TxSigner<Signature> + Send + Sync + 'static,
    {
        self.default = signer.address();
        self.register_signer(signer);
    }

    /// Get the default signer.
    pub fn default_signer(&self) -> Arc<dyn TxSigner<Signature> + Send + Sync + 'static> {
        self.signers
            .get(&self.default)
            .cloned()
            .expect("invalid signer")
    }

    /// Get the signer for the given address.
    pub fn signer_by_address(
        &self,
        address: Address,
    ) -> Option<Arc<dyn TxSigner<Signature> + Send + Sync + 'static>> {
        self.signers.get(&address).cloned()
    }

    #[doc(alias = "sign_tx_inner")]
    async fn sign_transaction_inner(
        &self,
        sender: Address,
        tx: &mut dyn SignableTransaction<Signature>,
    ) -> alloy_signer::Result<Signature> {
        self.signer_by_address(sender)
            .ok_or_else(|| {
                alloy_signer::Error::other(format!("Missing signing credential for {}", sender))
            })?
            .sign_transaction(tx)
            .await
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<N> NetworkWallet<N> for ZksyncWallet
where
    N: Network<
        UnsignedTx = alloy_consensus::TypedTransaction,
        TxEnvelope = alloy_consensus::TxEnvelope,
    >,
{
    fn default_signer_address(&self) -> Address {
        self.default
    }

    fn has_signer_for(&self, address: &Address) -> bool {
        self.signers.contains_key(address)
    }

    fn signer_addresses(&self) -> impl Iterator<Item = Address> {
        self.signers.keys().copied()
    }

    #[doc(alias = "sign_tx_from")]
    async fn sign_transaction_from(
        &self,
        sender: Address,
        tx: alloy_consensus::TypedTransaction,
    ) -> alloy_signer::Result<alloy_consensus::TxEnvelope> {
        match tx {
            alloy_consensus::TypedTransaction::Legacy(mut t) => {
                let sig = self.sign_transaction_inner(sender, &mut t).await?;
                Ok(t.into_signed(sig).into())
            }
            alloy_consensus::TypedTransaction::Eip2930(mut t) => {
                let sig = self.sign_transaction_inner(sender, &mut t).await?;
                Ok(t.into_signed(sig).into())
            }
            alloy_consensus::TypedTransaction::Eip1559(mut t) => {
                let sig = self.sign_transaction_inner(sender, &mut t).await?;
                Ok(t.into_signed(sig).into())
            }
            alloy_consensus::TypedTransaction::Eip4844(mut t) => {
                let sig = self.sign_transaction_inner(sender, &mut t).await?;
                Ok(t.into_signed(sig).into())
            }
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl NetworkWallet<Zksync> for ZksyncWallet {
    fn default_signer_address(&self) -> Address {
        self.default
    }

    fn has_signer_for(&self, address: &Address) -> bool {
        self.signers.contains_key(address)
    }

    fn signer_addresses(&self) -> impl Iterator<Item = Address> {
        self.signers.keys().copied()
    }

    #[doc(alias = "sign_tx_from")]
    async fn sign_transaction_from(
        &self,
        sender: Address,
        tx: TypedTransaction,
    ) -> alloy_signer::Result<TxEnvelope> {
        match tx {
            TypedTransaction::Native(t) => {
                let sig = <Self as NetworkWallet<alloy_network::Ethereum>>::sign_transaction_from(
                    self, sender, t,
                )
                .await?;
                Ok(TxEnvelope::Native(sig))
            }
            TypedTransaction::Eip712(mut t) => {
                let sig = self.sign_transaction_inner(sender, &mut t).await?;
                Ok(TxEnvelope::Eip712(t.into_signed(sig)))
            }
        }
    }
}
