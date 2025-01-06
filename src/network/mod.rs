//! Definition of the ZKsync network type.

use alloy::network::Network;

pub mod header;
pub mod header_response;
pub mod receipt_envelope;
pub mod receipt_response;
pub mod transaction_request;
pub mod transaction_response;
pub mod tx_envelope;
pub mod tx_type;
pub mod unsigned_tx;

/// ZKsync Network implementation.
///
/// Defines main types used in the network:
/// - [TxType](self::tx_type::TxType)
/// - [TxEnvelope](self::tx_envelope::TxEnvelope)
/// - [UnsignedTx](self::unsigned_tx::TypedTransaction)
/// - [ReceiptEnvelope](self::receipt_envelope::ReceiptEnvelope)
/// - [Header](self::header::Header)
/// - [TransactionRequest](self::transaction_request::TransactionRequest)
/// - [TransactionResponse](self::transaction_response::TransactionResponse)
/// - [ReceiptResponse](self::receipt_response::ReceiptResponse)
/// - [HeaderResponse](self::header_response::HeaderResponse)
/// - [BlockResponse](alloy::rpc::types::Block)
#[derive(Debug, Clone, Copy)]
pub struct Zksync {
    _private: (),
}

impl Network for Zksync {
    type TxType = self::tx_type::TxType;

    type TxEnvelope = self::tx_envelope::TxEnvelope;

    type UnsignedTx = self::unsigned_tx::TypedTransaction;

    type ReceiptEnvelope = self::receipt_envelope::ReceiptEnvelope;

    type Header = self::header::Header;

    type TransactionRequest = self::transaction_request::TransactionRequest;

    type TransactionResponse = self::transaction_response::TransactionResponse;

    type ReceiptResponse = self::receipt_response::ReceiptResponse;

    type HeaderResponse = self::header_response::HeaderResponse;

    type BlockResponse = alloy::rpc::types::Block<Self::TransactionResponse, Self::HeaderResponse>;
}
