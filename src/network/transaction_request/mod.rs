use alloy_network::{
    Network, TransactionBuilder, TransactionBuilderError, UnbuiltTransactionError,
};

use super::Zksync;

pub mod eip712;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum TransactionRequest {
    // TODO: it's a builder, should not be a enum. Instead store `eip712meta` as an option.
    Native(alloy_rpc_types_eth::transaction::TransactionRequest),
    // eip_712_meta: Eip712Meta,
}

impl Default for TransactionRequest {
    fn default() -> Self {
        Self::Native(Default::default())
    }
}

#[derive(Debug, Clone, Default)]
struct Eip712Meta {
    _todo: (),
}

/// Macro that delegates a method call to the inner variant implementation.
macro_rules! delegate {
    ($_self:ident.$method:ident($($args:expr),*)) => {
        match $_self {
            Self::Native(inner) => TransactionBuilder::$method(inner, $($args),*),
        }
    };
}

impl From<crate::network::unsigned_tx::TypedTransaction> for TransactionRequest {
    fn from(value: crate::network::unsigned_tx::TypedTransaction) -> Self {
        match value {
            crate::network::unsigned_tx::TypedTransaction::Native(inner) => {
                Self::Native(inner.into())
            }
        }
    }
}

impl From<crate::network::tx_envelope::TxEnvelope> for TransactionRequest {
    fn from(value: crate::network::tx_envelope::TxEnvelope) -> Self {
        match value {
            crate::network::tx_envelope::TxEnvelope::Native(inner) => Self::Native(inner.into()),
        }
    }
}

impl TransactionBuilder<Zksync> for TransactionRequest {
    fn chain_id(&self) -> Option<alloy_primitives::ChainId> {
        delegate!(self.chain_id())
    }

    fn set_chain_id(&mut self, chain_id: alloy_primitives::ChainId) {
        delegate!(self.set_chain_id(chain_id))
    }

    fn nonce(&self) -> Option<u64> {
        delegate!(self.nonce())
    }

    fn set_nonce(&mut self, nonce: u64) {
        delegate!(self.set_nonce(nonce))
    }

    fn input(&self) -> Option<&alloy_primitives::Bytes> {
        delegate!(self.input())
    }

    fn set_input<T: Into<alloy_primitives::Bytes>>(&mut self, input: T) {
        delegate!(self.set_input(input.into()))
    }

    fn from(&self) -> Option<alloy_primitives::Address> {
        delegate!(self.from())
    }

    fn set_from(&mut self, from: alloy_primitives::Address) {
        delegate!(self.set_from(from))
    }

    fn kind(&self) -> Option<alloy_primitives::TxKind> {
        delegate!(self.kind())
    }

    fn clear_kind(&mut self) {
        delegate!(self.clear_kind())
    }

    fn set_kind(&mut self, kind: alloy_primitives::TxKind) {
        delegate!(self.set_kind(kind))
    }

    fn value(&self) -> Option<alloy_primitives::U256> {
        delegate!(self.value())
    }

    fn set_value(&mut self, value: alloy_primitives::U256) {
        delegate!(self.set_value(value))
    }

    fn gas_price(&self) -> Option<u128> {
        delegate!(self.gas_price())
    }

    fn set_gas_price(&mut self, gas_price: u128) {
        delegate!(self.set_gas_price(gas_price))
    }

    fn max_fee_per_gas(&self) -> Option<u128> {
        delegate!(self.max_fee_per_gas())
    }

    fn set_max_fee_per_gas(&mut self, max_fee_per_gas: u128) {
        delegate!(self.set_max_fee_per_gas(max_fee_per_gas))
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        delegate!(self.max_priority_fee_per_gas())
    }

    fn set_max_priority_fee_per_gas(&mut self, max_priority_fee_per_gas: u128) {
        delegate!(self.set_max_priority_fee_per_gas(max_priority_fee_per_gas))
    }

    fn max_fee_per_blob_gas(&self) -> Option<u128> {
        delegate!(self.max_fee_per_blob_gas())
    }

    fn set_max_fee_per_blob_gas(&mut self, max_fee_per_blob_gas: u128) {
        delegate!(self.set_max_fee_per_blob_gas(max_fee_per_blob_gas))
    }

    fn gas_limit(&self) -> Option<u128> {
        delegate!(self.gas_limit())
    }

    fn set_gas_limit(&mut self, gas_limit: u128) {
        delegate!(self.set_gas_limit(gas_limit))
    }

    fn access_list(&self) -> Option<&alloy_rpc_types_eth::AccessList> {
        delegate!(self.access_list())
    }

    fn set_access_list(&mut self, access_list: alloy_rpc_types_eth::AccessList) {
        delegate!(self.set_access_list(access_list))
    }

    fn blob_sidecar(&self) -> Option<&alloy_rpc_types_eth::BlobTransactionSidecar> {
        delegate!(self.blob_sidecar())
    }

    fn set_blob_sidecar(&mut self, sidecar: alloy_rpc_types_eth::BlobTransactionSidecar) {
        delegate!(self.set_blob_sidecar(sidecar))
    }

    fn complete_type(&self, ty: <Zksync as Network>::TxType) -> Result<(), Vec<&'static str>> {
        // TODO: cover era-specific types.
        let eth_ty = ty
            .as_eth_type()
            .expect("Era-specific types are not supported yet");
        delegate!(self.complete_type(eth_ty))
    }

    fn can_submit(&self) -> bool {
        delegate!(self.can_submit())
    }

    fn can_build(&self) -> bool {
        delegate!(self.can_build())
    }

    fn output_tx_type(&self) -> <Zksync as Network>::TxType {
        delegate!(self.output_tx_type()).into()
    }

    fn output_tx_type_checked(&self) -> Option<<Zksync as Network>::TxType> {
        delegate!(self.output_tx_type_checked()).map(Into::into)
    }

    fn prep_for_submission(&mut self) {
        delegate!(self.prep_for_submission())
    }

    fn build_unsigned(
        self,
    ) -> alloy_network::BuildResult<crate::network::unsigned_tx::TypedTransaction, Zksync> {
        use TransactionBuilderError::*;

        let inner = match self {
            Self::Native(inner) => inner,
        };

        let result = TransactionBuilder::build_unsigned(inner);
        match result {
            Ok(tx) => Ok(crate::network::unsigned_tx::TypedTransaction::Native(tx)),
            Err(err) => {
                let UnbuiltTransactionError { request, error } = err;
                let wrapped_request = Self::Native(request);
                let error = match error {
                    InvalidTransactionRequest(tx, fields) => {
                        InvalidTransactionRequest(tx.into(), fields)
                    }
                    UnsupportedSignatureType => UnsupportedSignatureType,
                    Signer(s) => Signer(s),
                    Custom(c) => Custom(c),
                };

                Err(UnbuiltTransactionError {
                    request: wrapped_request,
                    error,
                })
            }
        }
    }

    async fn build<W: alloy_network::NetworkWallet<Zksync>>(
        self,
        wallet: &W,
    ) -> Result<<Zksync as Network>::TxEnvelope, TransactionBuilderError<Zksync>> {
        Ok(wallet.sign_request(self).await?)
    }
}
