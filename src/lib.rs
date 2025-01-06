//! ## alloy-zksync
//!
//! Implementation of the ZKsync network support for the [alloy][alloy] ecosystem.
//!
//! ## Overview
//!
//! This crate is designed to be a plug-in for [alloy][alloy]. If you're not familiar with it, the great entrypoint
//! would be the [alloy book](https://alloy.rs/) and [examples](https://github.com/alloy-rs/examples).
//!
//! `alloy-zksync` project has two main goals:
//! - Everything that works in `alloy` should work in `alloy-zksync` as well, even if the underlying implementation
//!   of functionality is different.
//! - ZKsync-specific features are supported.
//!
//! Main entrypoints are:
//! - [`ZksyncProvider`](crate::provider::ZksyncProvider) and [`zksync_provider`](crate::provider::zksync_provider). The
//!   provider is used whenever you need to access the node API.
//! - [`ZksyncWallet`](crate::wallet::ZksyncWallet). The wallet represents a provider with an account attached, and thus
//!   can be used to sign transactions.
//! - [`Zksync` network](crate::network::Zksync): a [network][alloy_network] definition. Most likely you won't need to
//!   interact with it directly, but the Network trait implementation is useful to look at if you want to see main data
//!   types.
//!
//! ## Examples
//!
//! Check out the [examples](https://github.com/popzxc/alloy-zksync/tree/main/examples) directory for more examples.
//!
//! [alloy]: https://github.com/alloy-rs/alloy/
//! [alloy_network]: https://docs.rs/alloy/latest/alloy/network/trait.Network.html

pub mod contracts;
pub mod network;
pub mod node_bindings;
pub mod provider;
pub mod types;
pub mod utils;
pub mod wallet;
