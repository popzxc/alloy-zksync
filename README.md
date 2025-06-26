# alloy-zksync

[Latest release docs](https://docs.rs/alloy-zksync) | [Main branch docs](https://matter-labs.github.io/alloy-zksync/alloy_zksync/)

This crate provides implementation of the ZKsync network support for the [alloy](https://github.com/alloy-rs/alloy/)
ecosystem.

> [!WARNING]  
> Work in progress.
>
> This crate can already be used for L2 communication, and is already integrated to some projects,
> including [foundry-zksync](https://github.com/matter-labs/foundry-zksync).
>
> Bug reports and PRs are appreciated.

## Overview

The main part of this crate is `Zksync` [network](https://docs.rs/alloy/latest/alloy/network/trait.Network.html)
implementation, which makes it possible to use all of the `alloy` functionality in L1-compatible way.

Where possible, all the technicalities (including, for example, signing EIP712 transactions) are hidden under the hood,
so that interfaces remain unchanged. In some cases, it means that some magic happens under the hood (one example is
that user doesn't have to manually specify the contract deployer address in deploy transactions).

Where existing interfaces cannot be used (for example, adding factory dependencies to deployment, or specifying
paymaster parameters),
the functionality is added in a way that is aligned with overall `alloy` design, either as methods on the types
specific to the `Zksync` network, or via extension traits (in case of `Provider`, for example).

Similarly to `anvil` bindings, `alloy-zksync` provides bindings for
[anvil-zksync](https://github.com/matter-labs/anvil-zksync).

For usage, the best reference currently is [examples](./examples/):
- [basic usage](./examples/showcase.rs),
- [contract deployment and interaction](./examples/deploy.rs).

## Progress

- [x] It compiles
- [x] Bindings for test node
- [x] Zksync network definition
- [x] Example of sending tx to era test node via era binding
- [x] Eip712 tx (works partially)
- [x] API extensions (`zks` namespace)
- [x] L1->L2 txs (deposits)
- [ ] L2->L1 txs (withdrawals)
- [x] Fillers
- [ ] Mirrored crate structure
- [ ] Mirrored features from upstream crates (e.g. serde)
- [ ] Comprehensive documentation

## Acknowledgements

Parts of the codebase for this repository were based on the [zksync-web3-rs](https://github.com/lambdaclass/zksync-web3-rs/)
crate by [LambdaClass](https://lambdaclass.com/), as well as [alloy](https://github.com/alloy-rs/alloy/) codebase.

## License

alloy-zksync is distributed under the terms of either

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/blog/license/mit/>)

at your option.
