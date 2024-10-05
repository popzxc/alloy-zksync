use alloy_network::{Ethereum, NetworkWallet};
use alloy_primitives::Address;

use crate::network::Zksync;

#[derive(Debug, Clone)]
pub struct EraWallet<T: NetworkWallet<Ethereum> + Clone> {
    inner: T,
}

impl<T: NetworkWallet<Ethereum> + Clone> From<T> for EraWallet<T> {
    fn from(inner: T) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl<T: NetworkWallet<Ethereum> + Clone> NetworkWallet<Zksync> for EraWallet<T> {
    fn default_signer_address(&self) -> Address {
        self.inner.default_signer_address()
    }

    fn has_signer_for(&self, address: &Address) -> bool {
        self.inner.has_signer_for(address)
    }

    /// Return an iterator of all signer addresses.
    fn signer_addresses(&self) -> impl Iterator<Item = Address> {
        self.inner.signer_addresses()
    }

    /// Asynchronously sign an unsigned transaction, with a specified
    /// credential.
    #[doc(alias = "sign_tx_from")]
    async fn sign_transaction_from(
        &self,
        sender: Address,
        tx: crate::network::unsigned_tx::TypedTransaction,
    ) -> alloy_signer::Result<crate::network::tx_envelope::TxEnvelope> {
        match tx {
            crate::network::unsigned_tx::TypedTransaction::Native(inner) => {
                let signed = self.inner.sign_transaction_from(sender, inner).await?;
                Ok(crate::network::tx_envelope::TxEnvelope::Native(signed))
            }
            crate::network::unsigned_tx::TypedTransaction::Eip712(inner) => {
                todo!("Eip712 signing is not yet implemented");
                // let signed = self
                //     .inner
                //     .sign_transaction_from(sender, inner.into())
                //     .await?;
                // Ok(crate::network::tx_envelope::TxEnvelope::Eip712(signed))
            }
        }
    }
}
