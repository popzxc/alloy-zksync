//! This crate provides implementation of the ZKsync network support for the [alloy](https://github.com/alloy-rs/alloy/) ecosystem.
//!
//! > [!WARNING]  
//! > This crate is under heavy development and is not suitable for production use.
//! > For now, it's being maintained as a personal pet project and not something maintained by Matter Labs.
//! >
//! > Functionality is lacking. Tests are lacking. PRs are appreciated.

pub mod network;
pub mod node_bindings;
pub mod provider;
pub mod wallet;
