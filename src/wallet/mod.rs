//! ZKsync wallet.

use alloy::consensus::SignableTransaction;
use alloy::network::{Network, NetworkWallet, TxSigner};
use alloy::primitives::Address;

use crate::network::{Zksync, tx_envelope::TxEnvelope, unsigned_tx::TypedTransaction};

use alloy::primitives::Signature;
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
    /// [`TransactionRequest`]: alloy::rpc::types::eth::TransactionRequest
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
    /// [`TransactionRequest`]: alloy::rpc::types::eth::TransactionRequest
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
    ) -> alloy::signers::Result<Signature> {
        self.signer_by_address(sender)
            .ok_or_else(|| {
                alloy::signers::Error::other(format!("Missing signing credential for {sender}"))
            })?
            .sign_transaction(tx)
            .await
    }
}

impl<N> NetworkWallet<N> for ZksyncWallet
where
    N: Network<
            UnsignedTx = alloy::consensus::TypedTransaction,
            TxEnvelope = alloy::consensus::TxEnvelope,
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
        tx: alloy::consensus::TypedTransaction,
    ) -> alloy::signers::Result<alloy::consensus::TxEnvelope> {
        match tx {
            alloy::consensus::TypedTransaction::Legacy(mut t) => {
                let sig = self.sign_transaction_inner(sender, &mut t).await?;
                Ok(t.into_signed(sig).into())
            }
            alloy::consensus::TypedTransaction::Eip2930(mut t) => {
                let sig = self.sign_transaction_inner(sender, &mut t).await?;
                Ok(t.into_signed(sig).into())
            }
            alloy::consensus::TypedTransaction::Eip1559(mut t) => {
                let sig = self.sign_transaction_inner(sender, &mut t).await?;
                Ok(t.into_signed(sig).into())
            }
            alloy::consensus::TypedTransaction::Eip4844(mut t) => {
                let sig = self.sign_transaction_inner(sender, &mut t).await?;
                Ok(t.into_signed(sig).into())
            }
            alloy::consensus::TypedTransaction::Eip7702(mut t) => {
                let sig = self.sign_transaction_inner(sender, &mut t).await?;
                Ok(t.into_signed(sig).into())
            }
        }
    }
}

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
    ) -> alloy::signers::Result<TxEnvelope> {
        match tx {
            TypedTransaction::Native(t) => {
                let sig = <Self as NetworkWallet<alloy::network::Ethereum>>::sign_transaction_from(
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
