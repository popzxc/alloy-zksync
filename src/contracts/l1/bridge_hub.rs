alloy::sol! {
    /// Represents a direct L2 transaction request.
    #[allow(missing_docs)]
    struct L2TransactionRequestDirect {
        uint256 chainId;
        uint256 mintValue;
        address l2Contract;
        uint256 l2Value;
        bytes l2Calldata;
        uint256 l2GasLimit;
        uint256 l2GasPerPubdataByteLimit;
        bytes[] factoryDeps;
        address refundRecipient;
    }

    /// Represents an L2 transaction request involving two bridges.
    #[allow(missing_docs)]
    struct L2TransactionRequestTwoBridges {
        uint256 chainId;
        uint256 mintValue;
        uint256 l2Value;
        uint256 l2GasLimit;
        uint256 l2GasPerPubdataByteLimit;
        address refundRecipient;
        address secondBridgeAddress;
        uint256 secondBridgeValue;
        bytes secondBridgeCalldata;
    }

    /// Represents a canonical L2 transaction.
    #[allow(missing_docs)]
    struct L2CanonicalTransaction {
        uint256 txType;
        uint256 from;
        uint256 to;
        uint256 gasLimit;
        uint256 gasPerPubdataByteLimit;
        uint256 maxFeePerGas;
        uint256 maxPriorityFeePerGas;
        uint256 paymaster;
        uint256 nonce;
        uint256 value;
        uint256[4] reserved;
        bytes data;
        bytes signature;
        uint256[] factoryDeps;
        bytes paymasterInput;
        bytes reservedDynamic;
    }

    /// Bridgehub contract for handling L2 transaction requests and related operations.
    #[allow(missing_docs)]
    #[sol(rpc)]
    contract Bridgehub {
        /// Requests a direct L2 transaction.
        ///
        /// # Arguments
        ///
        /// * `request` - The L2 transaction request.
        ///
        /// # Returns
        ///
        /// The canonical transaction hash.
        function requestL2TransactionDirect(
            L2TransactionRequestDirect memory request
        ) external payable returns (bytes32 canonicalTxHash);

        /// Requests an L2 transaction involving two bridges.
        ///
        /// # Arguments
        ///
        /// * `_request` - The L2 transaction request.
        ///
        /// # Returns
        ///
        /// The canonical transaction hash.
        function requestL2TransactionTwoBridges(
            L2TransactionRequestTwoBridges calldata _request
        ) external payable returns (bytes32 canonicalTxHash);

        /// Calculates the base cost of an L2 transaction.
        ///
        /// # Arguments
        ///
        /// * `_chainId` - The chain ID.
        /// * `_gasPrice` - The gas price.
        /// * `_l2GasLimit` - The L2 gas limit.
        /// * `_l2GasPerPubdataByteLimit` - The L2 gas per pubdata byte limit.
        ///
        /// # Returns
        ///
        /// The base cost of the L2 transaction.
        function l2TransactionBaseCost(
            uint256 _chainId,
            uint256 _gasPrice,
            uint256 _l2GasLimit,
            uint256 _l2GasPerPubdataByteLimit
        ) external view returns (uint256);

        /// Emitted when a new priority request is made.
        ///
        /// # Arguments
        ///
        /// * `txId` - The transaction ID.
        /// * `txHash` - The transaction hash.
        /// * `expirationTimestamp` - The expiration timestamp.
        /// * `transaction` - The canonical transaction.
        /// * `factoryDeps` - The factory dependencies.
        event NewPriorityRequest(
            uint256 txId,
            bytes32 txHash,
            uint64 expirationTimestamp,
            L2CanonicalTransaction transaction,
            bytes[] factoryDeps
        );
    }
}
