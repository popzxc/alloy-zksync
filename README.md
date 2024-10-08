# alloy-zksync

This crate provides implementation of the ZKsync network support for the [alloy](https://github.com/alloy-rs/alloy/) ecosystem.

> [!WARNING]  
> This crate is under heavy development and is not suitable for production use.
> For now, it's being maintained as a personal pet project and not something maintained by Matter Labs.
>
> Functionality is lacking. Tests are lacking. PRs are appreciated.

## Progress

- [x] It compiles
- [x] Bindings for test node
- [x] Zksync network definition
- [x] Example of sending tx to era test node via era binding
- [x] Eip712 tx
- [in progress] API extensions (`zks` namespace)
- [ ] L1->L2 txs (deposits)
- [ ] L2->L1 txs (withdrawals)
- [ ] Mirrored crate structure
- [ ] Mirrored features from upstream crates (e.g. serde)


## Acknowledgements

Parts of the codebase for this repository were based on the [zksync-web3-rs](https://github.com/lambdaclass/zksync-web3-rs/)
crate by [LambdaClass](https://lambdaclass.com/), as well as [alloy](https://github.com/alloy-rs/alloy/) codebase.

## License

alloy-zksync is distributed under the terms of either

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/blog/license/mit/>)

at your option.
