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
    /// Creates a new `L1TransactionReceipt` object.
    pub fn new(tx_receipt: TransactionReceipt, l2_provider: RootProvider<T, Zksync>) -> Self {
        Self {
            inner: tx_receipt,
            l2_provider,
        }
    }

    /// Returns a receipt for the L1 operation.
    pub fn get_receipt(&self) -> &TransactionReceipt {
        &self.inner
    }

    /// Returns a [`PendingTransactionBuilder`](https://docs.rs/alloy/latest/alloy/providers/struct.PendingTransactionBuilder.html)
    /// for the L2 transaction, which can be used to await the transaction on L2.
    ///
    /// Will return an error if the transaction request used to create the object does not contain
    /// priority operation information (e.g. it doesn't correspond to an L1->L2 transaction).
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
