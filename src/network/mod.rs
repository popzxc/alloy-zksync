use alloy_network::Ethereum;
use alloy_network::Network;

mod header;
mod header_response;
mod receipt_envelope;
mod receipt_response;
mod transaction_request;
mod transaction_response;
mod tx_envelope;
mod tx_type;
mod unsigned_tx;

#[derive(Debug, Clone, Copy)]
pub struct EraNetwork {
    _private: (),
}

impl Network for EraNetwork {
    type TxType = <Ethereum as Network>::TxType;

    type TxEnvelope = self::tx_envelope::TxEnvelope;

    type UnsignedTx = self::unsigned_tx::TypedTransaction;

    type ReceiptEnvelope = <Ethereum as Network>::ReceiptEnvelope;

    type Header = <Ethereum as Network>::Header;

    type TransactionRequest = self::transaction_request::TransactionRequest;

    type TransactionResponse = <Ethereum as Network>::TransactionResponse;

    type ReceiptResponse = <Ethereum as Network>::ReceiptResponse;

    type HeaderResponse = <Ethereum as Network>::HeaderResponse;
}
