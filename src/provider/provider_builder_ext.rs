use alloy::{
    providers::{
        fillers::{JoinFill, TxFiller, WalletFiller},
        ProviderBuilder, ProviderLayer,
    },
    signers::local::LocalSigner,
};

use crate::{
    network::Zksync,
    node_bindings::{EraTestNode, EraTestNodeError},
    provider::layers::era_test_node::EraTestNodeLayer,
    wallet::ZksyncWallet,
};

type EraTestNodeProviderResult<T> = Result<T, EraTestNodeError>;
type JoinedZksyncWalletFiller<F> = JoinFill<F, WalletFiller<ZksyncWallet>>;

pub trait ProviderBuilderExt<L, F>: Sized
where
    F: TxFiller<Zksync>
        + ProviderLayer<L::Provider, alloy::transports::http::Http<reqwest::Client>, Zksync>,
    L: alloy::providers::ProviderLayer<
        crate::provider::layers::era_test_node::EraTestNodeProvider<
            alloy::providers::RootProvider<alloy::transports::http::Http<reqwest::Client>, Zksync>,
            alloy::transports::http::Http<reqwest::Client>,
        >,
        alloy::transports::http::Http<reqwest::Client>,
        Zksync,
    >,
{
    fn on_era_test_node(self) -> F::Provider;

    fn on_era_test_node_with_wallet(
        self,
    ) -> <JoinedZksyncWalletFiller<F> as ProviderLayer<
        L::Provider,
        alloy::transports::http::Http<reqwest::Client>,
        Zksync,
    >>::Provider;

    fn on_era_test_node_with_config(
        self,
        f: impl FnOnce(EraTestNode) -> EraTestNode,
    ) -> F::Provider;

    fn on_era_test_node_with_wallet_and_config(
        self,
        f: impl FnOnce(EraTestNode) -> EraTestNode,
    ) -> <JoinedZksyncWalletFiller<F> as ProviderLayer<
        L::Provider,
        alloy::transports::http::Http<reqwest::Client>,
        Zksync,
    >>::Provider;

    #[allow(clippy::type_complexity)] // TODO: Should indeed be simplified, but rn it's ported from upstream code.
    fn try_on_era_test_node_with_wallet_and_config(
        self,
        f: impl FnOnce(EraTestNode) -> EraTestNode,
    ) -> EraTestNodeProviderResult<
        <JoinedZksyncWalletFiller<F> as ProviderLayer<
            L::Provider,
            alloy::transports::http::Http<reqwest::Client>,
            Zksync,
        >>::Provider,
    >;
}

impl<L, F> ProviderBuilderExt<L, F> for ProviderBuilder<L, F, Zksync>
where
    F: TxFiller<Zksync>
        + ProviderLayer<L::Provider, alloy::transports::http::Http<reqwest::Client>, Zksync>,
    L: alloy::providers::ProviderLayer<
        crate::provider::layers::era_test_node::EraTestNodeProvider<
            alloy::providers::RootProvider<alloy::transports::http::Http<reqwest::Client>, Zksync>,
            alloy::transports::http::Http<reqwest::Client>,
        >,
        alloy::transports::http::Http<reqwest::Client>,
        Zksync,
    >,
{
    fn on_era_test_node(self) -> F::Provider {
        self.on_era_test_node_with_config(std::convert::identity)
    }

    fn on_era_test_node_with_wallet(
        self,
    ) -> <JoinedZksyncWalletFiller<F> as ProviderLayer<
        L::Provider,
        alloy::transports::http::Http<reqwest::Client>,
        Zksync,
    >>::Provider {
        self.on_era_test_node_with_wallet_and_config(std::convert::identity)
    }

    fn on_era_test_node_with_config(
        self,
        f: impl FnOnce(EraTestNode) -> EraTestNode,
    ) -> F::Provider {
        let era_test_node_layer = EraTestNodeLayer::from(f(Default::default()));
        let url = era_test_node_layer.endpoint_url();

        self.layer(era_test_node_layer).on_http(url)
    }

    fn on_era_test_node_with_wallet_and_config(
        self,
        f: impl FnOnce(EraTestNode) -> EraTestNode,
    ) -> <JoinedZksyncWalletFiller<F> as ProviderLayer<
        L::Provider,
        alloy::transports::http::Http<reqwest::Client>,
        Zksync,
    >>::Provider {
        self.try_on_era_test_node_with_wallet_and_config(f).unwrap()
    }

    /// Build this provider with era_test_node, using an Reqwest HTTP transport. The
    /// given function is used to configure the era_test_node instance. This
    /// function configures a wallet backed by era_test_node keys, and is intended for
    /// use in tests.
    fn try_on_era_test_node_with_wallet_and_config(
        self,
        f: impl FnOnce(EraTestNode) -> EraTestNode,
    ) -> EraTestNodeProviderResult<
        <JoinedZksyncWalletFiller<F> as ProviderLayer<
            L::Provider,
            alloy::transports::http::Http<reqwest::Client>,
            Zksync,
        >>::Provider,
    > {
        use alloy::signers::Signer;

        let era_test_node_layer = EraTestNodeLayer::from(f(Default::default()));
        let url = era_test_node_layer.endpoint_url();

        let default_keys = era_test_node_layer.instance().keys().to_vec();
        let (default_key, remaining_keys) = default_keys
            .split_first()
            .ok_or(crate::node_bindings::EraTestNodeError::NoKeysAvailable)?;

        let default_signer = LocalSigner::from(default_key.clone())
            .with_chain_id(Some(era_test_node_layer.instance().chain_id()));
        let mut wallet = ZksyncWallet::from(default_signer);

        for key in remaining_keys {
            let signer = LocalSigner::from(key.clone());
            wallet.register_signer(signer)
        }

        Ok(self.wallet(wallet).layer(era_test_node_layer).on_http(url))
    }
}
