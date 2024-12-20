use super::l1_communication_error::L1CommunicationError;
use crate::{contracts::l1::bridge_hub::Bridgehub::NewPriorityRequest, network::Zksync};
use alloy::{
    providers::{PendingTransactionBuilder, RootProvider},
    rpc::types::eth::TransactionReceipt,
    transports::Transport,
};

/// A wrapper struct to hold L1 transaction receipt and L2 provider
/// which is used by the associated functions.
pub struct L1TransactionReceipt<T> {
    /// Ethereum transaction receipt.
    inner: TransactionReceipt,
    /// A reference to the L2 provider.
    l2_provider: RootProvider<T, Zksync>,
}

impl<T> L1TransactionReceipt<T>
where
    T: Transport + Clone,
{
    pub fn new(tx_receipt: TransactionReceipt, l2_provider: RootProvider<T, Zksync>) -> Self {
        Self {
            inner: tx_receipt,
            l2_provider,
        }
    }

    pub fn get_receipt(&self) -> &TransactionReceipt {
        &self.inner
    }

    pub fn get_l2_tx(&self) -> Result<PendingTransactionBuilder<T, Zksync>, L1CommunicationError> {
        let l1_to_l2_tx_log = self
            .inner
            .inner
            .logs()
            .iter()
            .filter_map(|log| log.log_decode::<NewPriorityRequest>().ok())
            .next()
            .ok_or(L1CommunicationError::NewPriorityRequestLogNotFound)?;

        let l2_tx_hash = l1_to_l2_tx_log.inner.txHash;

        Ok(PendingTransactionBuilder::new(
            self.l2_provider.clone(),
            l2_tx_hash,
        ))
    }
}
