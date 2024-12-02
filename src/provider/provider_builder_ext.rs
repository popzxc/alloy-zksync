use alloy::{
    providers::{
        fillers::{JoinFill, TxFiller, WalletFiller},
        ProviderBuilder, ProviderLayer, RootProvider,
    },
    signers::local::LocalSigner,
};

use crate::{
    network::Zksync,
    node_bindings::{AnvilZKsync, AnvilZKsyncError},
    provider::layers::anvil_zksync::{AnvilZKsyncLayer, AnvilZKsyncProvider},
    wallet::ZksyncWallet,
};

type AnvilZKsyncProviderResult<T> = Result<T, AnvilZKsyncError>;
type JoinedZksyncWalletFiller<F> = JoinFill<F, WalletFiller<ZksyncWallet>>;
type ZksyncHttpTransport = alloy::transports::http::Http<reqwest::Client>;

pub trait ProviderBuilderExt<L, F>: Sized
where
    F: TxFiller<Zksync> + ProviderLayer<L::Provider, ZksyncHttpTransport, Zksync>,
    L: ProviderLayer<
        AnvilZKsyncProvider<RootProvider<ZksyncHttpTransport, Zksync>, ZksyncHttpTransport>,
        ZksyncHttpTransport,
        Zksync,
    >,
{
    fn on_anvil_zksync(self) -> F::Provider;

    fn on_anvil_zksync_with_wallet(
        self,
    ) -> <JoinedZksyncWalletFiller<F> as ProviderLayer<
        L::Provider,
        ZksyncHttpTransport,
        Zksync,
    >>::Provider;

    fn on_anvil_zksync_with_config(self, f: impl FnOnce(AnvilZKsync) -> AnvilZKsync)
        -> F::Provider;

    fn on_anvil_zksync_with_wallet_and_config(
        self,
        f: impl FnOnce(AnvilZKsync) -> AnvilZKsync,
    ) -> <JoinedZksyncWalletFiller<F> as ProviderLayer<
        L::Provider,
        ZksyncHttpTransport,
        Zksync,
    >>::Provider;

    fn try_on_anvil_zksync_with_wallet_and_config(
        self,
        f: impl FnOnce(AnvilZKsync) -> AnvilZKsync,
    ) -> AnvilZKsyncProviderResult<
        <JoinedZksyncWalletFiller<F> as ProviderLayer<
            L::Provider,
            ZksyncHttpTransport,
            Zksync,
        >>::Provider,
    >;
}

impl<L, F> ProviderBuilderExt<L, F> for ProviderBuilder<L, F, Zksync>
where
    F: TxFiller<Zksync> + ProviderLayer<L::Provider, ZksyncHttpTransport, Zksync>,
    L: ProviderLayer<
        AnvilZKsyncProvider<RootProvider<ZksyncHttpTransport, Zksync>, ZksyncHttpTransport>,
        ZksyncHttpTransport,
        Zksync,
    >,
{
    fn on_anvil_zksync(self) -> F::Provider {
        self.on_anvil_zksync_with_config(std::convert::identity)
    }

    fn on_anvil_zksync_with_wallet(
        self,
    ) -> <JoinedZksyncWalletFiller<F> as ProviderLayer<
        L::Provider,
        ZksyncHttpTransport,
        Zksync,
    >>::Provider{
        self.on_anvil_zksync_with_wallet_and_config(std::convert::identity)
    }

    fn on_anvil_zksync_with_config(
        self,
        f: impl FnOnce(AnvilZKsync) -> AnvilZKsync,
    ) -> F::Provider {
        let anvil_zksync_layer = AnvilZKsyncLayer::from(f(Default::default()));
        let url = anvil_zksync_layer.endpoint_url();

        self.layer(anvil_zksync_layer).on_http(url)
    }

    fn on_anvil_zksync_with_wallet_and_config(
        self,
        f: impl FnOnce(AnvilZKsync) -> AnvilZKsync,
    ) -> <JoinedZksyncWalletFiller<F> as ProviderLayer<
        L::Provider,
        ZksyncHttpTransport,
        Zksync,
    >>::Provider{
        self.try_on_anvil_zksync_with_wallet_and_config(f).unwrap()
    }

    /// Build this provider with anvil-zksync, using an Reqwest HTTP transport. The
    /// given function is used to configure the anvil-zksync instance. This
    /// function configures a wallet backed by anvil-zksync keys, and is intended for
    /// use in tests.
    fn try_on_anvil_zksync_with_wallet_and_config(
        self,
        f: impl FnOnce(AnvilZKsync) -> AnvilZKsync,
    ) -> AnvilZKsyncProviderResult<
        <JoinedZksyncWalletFiller<F> as ProviderLayer<
            L::Provider,
            ZksyncHttpTransport,
            Zksync,
        >>::Provider,
    >{
        use alloy::signers::Signer;

        let anvil_zksync_layer = AnvilZKsyncLayer::from(f(Default::default()));
        let url = anvil_zksync_layer.endpoint_url();

        let default_keys = anvil_zksync_layer.instance().keys().to_vec();
        let (default_key, remaining_keys) = default_keys
            .split_first()
            .ok_or(crate::node_bindings::AnvilZKsyncError::NoKeysAvailable)?;

        let default_signer = LocalSigner::from(default_key.clone())
            .with_chain_id(Some(anvil_zksync_layer.instance().chain_id()));
        let mut wallet = ZksyncWallet::from(default_signer);

        for key in remaining_keys {
            let signer = LocalSigner::from(key.clone());
            wallet.register_signer(signer)
        }

        Ok(self.wallet(wallet).layer(anvil_zksync_layer).on_http(url))
    }
}
