use alloy_network::{
    Network, TransactionBuilder, TransactionBuilderError, UnbuiltTransactionError,
};

use super::Zksync;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum TransactionRequest {
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

impl From<crate::network::unsigned_tx::TypedTransaction> for TransactionRequest {
    fn from(value: crate::network::unsigned_tx::TypedTransaction) -> Self {
        Self::Native(From::from(value.inner))
    }
}

impl From<crate::network::tx_envelope::TxEnvelope> for TransactionRequest {
    fn from(value: crate::network::tx_envelope::TxEnvelope) -> Self {
        Self::Native(From::from(value.inner))
    }
}

impl TransactionBuilder<Zksync> for TransactionRequest {
    fn chain_id(&self) -> Option<alloy_primitives::ChainId> {
        match self {
            Self::Native(inner) => TransactionBuilder::chain_id(inner),
        }
    }

    fn set_chain_id(&mut self, chain_id: alloy_primitives::ChainId) {
        match self {
            Self::Native(inner) => TransactionBuilder::set_chain_id(inner, chain_id),
        }
    }

    fn nonce(&self) -> Option<u64> {
        match self {
            Self::Native(inner) => TransactionBuilder::nonce(inner),
        }
    }

    fn set_nonce(&mut self, nonce: u64) {
        match self {
            Self::Native(inner) => TransactionBuilder::set_nonce(inner, nonce),
        }
    }

    fn input(&self) -> Option<&alloy_primitives::Bytes> {
        match self {
            Self::Native(inner) => TransactionBuilder::input(inner),
        }
    }

    fn set_input<T: Into<alloy_primitives::Bytes>>(&mut self, input: T) {
        match self {
            Self::Native(inner) => TransactionBuilder::set_input(inner, input),
        }
    }

    fn from(&self) -> Option<alloy_primitives::Address> {
        match self {
            Self::Native(inner) => TransactionBuilder::from(inner),
        }
    }

    fn set_from(&mut self, from: alloy_primitives::Address) {
        match self {
            Self::Native(inner) => TransactionBuilder::set_from(inner, from),
        }
    }

    fn kind(&self) -> Option<alloy_primitives::TxKind> {
        match self {
            Self::Native(inner) => TransactionBuilder::kind(inner),
        }
    }

    fn clear_kind(&mut self) {
        match self {
            Self::Native(inner) => TransactionBuilder::clear_kind(inner),
        }
    }

    fn set_kind(&mut self, kind: alloy_primitives::TxKind) {
        match self {
            Self::Native(inner) => TransactionBuilder::set_kind(inner, kind),
        }
    }

    fn value(&self) -> Option<alloy_primitives::U256> {
        match self {
            Self::Native(inner) => TransactionBuilder::value(inner),
        }
    }

    fn set_value(&mut self, value: alloy_primitives::U256) {
        match self {
            Self::Native(inner) => TransactionBuilder::set_value(inner, value),
        }
    }

    fn gas_price(&self) -> Option<u128> {
        match self {
            Self::Native(inner) => TransactionBuilder::gas_price(inner),
        }
    }

    fn set_gas_price(&mut self, gas_price: u128) {
        match self {
            Self::Native(inner) => TransactionBuilder::set_gas_price(inner, gas_price),
        }
    }

    fn max_fee_per_gas(&self) -> Option<u128> {
        match self {
            Self::Native(inner) => TransactionBuilder::max_fee_per_gas(inner),
        }
    }

    fn set_max_fee_per_gas(&mut self, max_fee_per_gas: u128) {
        match self {
            Self::Native(inner) => TransactionBuilder::set_max_fee_per_gas(inner, max_fee_per_gas),
        }
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        match self {
            Self::Native(inner) => TransactionBuilder::max_priority_fee_per_gas(inner),
        }
    }

    fn set_max_priority_fee_per_gas(&mut self, max_priority_fee_per_gas: u128) {
        match self {
            Self::Native(inner) => {
                TransactionBuilder::set_max_priority_fee_per_gas(inner, max_priority_fee_per_gas)
            }
        }
    }

    fn max_fee_per_blob_gas(&self) -> Option<u128> {
        match self {
            Self::Native(inner) => TransactionBuilder::max_fee_per_blob_gas(inner),
        }
    }

    fn set_max_fee_per_blob_gas(&mut self, max_fee_per_blob_gas: u128) {
        match self {
            Self::Native(inner) => {
                TransactionBuilder::set_max_fee_per_blob_gas(inner, max_fee_per_blob_gas)
            }
        }
    }

    fn gas_limit(&self) -> Option<u128> {
        match self {
            Self::Native(inner) => TransactionBuilder::gas_limit(inner),
        }
    }

    fn set_gas_limit(&mut self, gas_limit: u128) {
        match self {
            Self::Native(inner) => TransactionBuilder::set_gas_limit(inner, gas_limit),
        }
    }

    fn access_list(&self) -> Option<&alloy_rpc_types_eth::AccessList> {
        match self {
            Self::Native(inner) => TransactionBuilder::access_list(inner),
        }
    }

    fn set_access_list(&mut self, access_list: alloy_rpc_types_eth::AccessList) {
        match self {
            Self::Native(inner) => TransactionBuilder::set_access_list(inner, access_list),
        }
    }

    fn blob_sidecar(&self) -> Option<&alloy_rpc_types_eth::BlobTransactionSidecar> {
        match self {
            Self::Native(inner) => TransactionBuilder::blob_sidecar(inner),
        }
    }

    fn set_blob_sidecar(&mut self, sidecar: alloy_rpc_types_eth::BlobTransactionSidecar) {
        match self {
            Self::Native(inner) => TransactionBuilder::set_blob_sidecar(inner, sidecar),
        }
    }

    fn complete_type(&self, ty: <Zksync as Network>::TxType) -> Result<(), Vec<&'static str>> {
        // TODO: cover era-specific types.
        let eth_ty = ty
            .as_eth_type()
            .expect("Era-specific types are not supported yet");
        match self {
            Self::Native(inner) => TransactionBuilder::complete_type(inner, eth_ty),
        }
    }

    fn can_submit(&self) -> bool {
        match self {
            Self::Native(inner) => TransactionBuilder::can_submit(inner),
        }
    }

    fn can_build(&self) -> bool {
        match self {
            Self::Native(inner) => TransactionBuilder::can_build(inner),
        }
    }

    fn output_tx_type(&self) -> <Zksync as Network>::TxType {
        match self {
            Self::Native(inner) => TransactionBuilder::output_tx_type(inner).into(),
        }
    }

    fn output_tx_type_checked(&self) -> Option<<Zksync as Network>::TxType> {
        match self {
            Self::Native(inner) => {
                TransactionBuilder::output_tx_type_checked(inner).map(Into::into)
            }
        }
    }

    fn prep_for_submission(&mut self) {
        match self {
            Self::Native(inner) => TransactionBuilder::prep_for_submission(inner),
        }
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
            Ok(tx) => Ok(crate::network::unsigned_tx::TypedTransaction { inner: tx }),
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
