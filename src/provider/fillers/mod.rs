use alloy::{
    network::TransactionBuilder,
    primitives::U256,
    providers::{
        fillers::{FillerControlFlow, TxFiller},
        Provider, SendableTx,
    },
    transports::{Transport, TransportResult},
};

use crate::network::{
    transaction_request::{TransactionRequest, ZksyncTransactionBuilder},
    Zksync,
};

use super::{Eip712Fee, ZksyncProvider};

#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct Eip712FeeFiller {}

impl TxFiller<Zksync> for Eip712FeeFiller {
    type Fillable = Eip712Fee;

    fn status(&self, tx: &TransactionRequest) -> FillerControlFlow {
        if ZksyncTransactionBuilder::gas_per_pubdata(tx).unwrap_or_default() > U256::ZERO  // TODO: Should be `is_none()` once `gas_per_pubdata` in TransactionRequest is `Option`
            && tx.gas_limit().is_some()
            && tx.max_fee_per_gas().is_some()
            && tx.max_priority_fee_per_gas().is_some()
        {
            return FillerControlFlow::Finished;
        }
        if tx.from().is_none() {
            return FillerControlFlow::missing("Eip712FeeFiller", vec!["from"]);
        }
        FillerControlFlow::Ready
    }

    fn fill_sync(&self, _tx: &mut SendableTx<Zksync>) {}

    async fn prepare<P, T>(
        &self,
        provider: &P,
        tx: &TransactionRequest,
    ) -> TransportResult<Self::Fillable>
    where
        P: Provider<T, Zksync>,
        T: Transport + Clone,
    {
        let fee = provider.estimate_fee(tx.clone()).await?;
        Ok(fee)
    }

    async fn fill(
        &self,
        fee: Self::Fillable,
        mut tx: SendableTx<Zksync>,
    ) -> TransportResult<SendableTx<Zksync>> {
        if let Some(builder) = tx.as_mut_builder() {
            // Only set fields that are missing to prevent accidental overrides.
            if builder.gas_limit().is_none() {
                builder.set_gas_limit(fee.gas_limit);
            }
            if builder.max_fee_per_gas().is_none() {
                builder.set_max_fee_per_gas(fee.max_fee_per_gas);
            }
            if builder.max_priority_fee_per_gas().is_none() {
                builder.set_max_priority_fee_per_gas(fee.max_priority_fee_per_gas);
            }
            // TODO: Should be `is_none()` once `gas_per_pubdata` in TransactionRequest is `Option`
            if ZksyncTransactionBuilder::gas_per_pubdata(builder).unwrap_or_default() == U256::ZERO
            {
                ZksyncTransactionBuilder::set_gas_per_pubdata(builder, fee.gas_per_pubdata_limit);
            }
        }
        Ok(tx)
    }
}
