//! Layer for `anvil-zksync` wrapper.

use alloy::{
    providers::{Provider, ProviderLayer, RootProvider},
    transports::Transport,
};
use std::{
    marker::PhantomData,
    sync::{Arc, OnceLock},
};
use url::Url;

use crate::{
    network::Zksync,
    node_bindings::{AnvilZKsync, AnvilZKsyncInstance},
};

/// A layer that wraps an [`AnvilZKsync`] config.
///
/// The config will be used to spawn an [`AnvilZKsyncInstance`] when the layer is applied, or when the
/// user requests any information about the anvil node (e.g. via the [`AnvilZKsyncLayer::endpoint_url`]
/// method).
#[derive(Debug, Clone, Default)]
pub struct AnvilZKsyncLayer {
    anvil: AnvilZKsync,
    instance: OnceLock<Arc<AnvilZKsyncInstance>>,
}

impl AnvilZKsyncLayer {
    /// Starts the anvil instance, or gets a reference to the existing instance.
    pub fn instance(&self) -> &Arc<AnvilZKsyncInstance> {
        self.instance
            .get_or_init(|| Arc::new(self.anvil.clone().spawn()))
    }

    /// Get the instance http endpoint.
    #[doc(alias = "http_endpoint_url")]
    pub fn endpoint_url(&self) -> Url {
        self.instance().endpoint_url()
    }
}

impl From<AnvilZKsync> for AnvilZKsyncLayer {
    fn from(anvil: AnvilZKsync) -> Self {
        Self {
            anvil,
            instance: OnceLock::new(),
        }
    }
}

impl<P, T> ProviderLayer<P, T, Zksync> for AnvilZKsyncLayer
where
    P: Provider<T, Zksync>,
    T: Transport + Clone,
{
    type Provider = AnvilZKsyncProvider<P, T>;

    fn layer(&self, inner: P) -> Self::Provider {
        let anvil = self.instance();
        AnvilZKsyncProvider::new(inner, anvil.clone())
    }
}

/// A provider that wraps an [`AnvilZKsyncInstance`], preventing the instance from
/// being dropped while the provider is in use.
#[derive(Clone, Debug)]
pub struct AnvilZKsyncProvider<P, T> {
    inner: P,
    _anvil: Arc<AnvilZKsyncInstance>,
    _pd: PhantomData<fn() -> T>,
}

impl<P, T> AnvilZKsyncProvider<P, T>
where
    P: Provider<T, Zksync>,
    T: Transport + Clone,
{
    /// Creates a new `AnvilZKsyncProvider` with the given inner provider and anvil
    /// instance.
    pub fn new(inner: P, _anvil: Arc<AnvilZKsyncInstance>) -> Self {
        Self {
            inner,
            _anvil,
            _pd: PhantomData,
        }
    }
}

impl<P, T> Provider<T, Zksync> for AnvilZKsyncProvider<P, T>
where
    P: Provider<T, Zksync>,
    T: Transport + Clone,
{
    #[inline(always)]
    fn root(&self) -> &RootProvider<T, Zksync> {
        self.inner.root()
    }
}
