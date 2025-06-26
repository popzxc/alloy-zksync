// Adapted from Anvil node bindings in the alloy project:
// https://github.com/alloy-rs/alloy/blob/2d26b057c64cbcc77654f4691141c308d63b286f/crates/node-bindings/src/anvil.rs

//! Utilities for launching an `anvil-zksync` instance.

use alloy::primitives::{hex, Address, ChainId};
use k256::{ecdsa::SigningKey, SecretKey as K256SecretKey};
use rand::Rng;
use std::{
    io::{BufRead, BufReader},
    net::SocketAddr,
    path::PathBuf,
    process::{Child, Command},
    str::FromStr,
    time::{Duration, Instant},
};
use thiserror::Error;
use url::Url;

/// How long we will wait for anvil-zksync to indicate that it is ready.
const ANVIL_ZKSYNC_STARTUP_TIMEOUT_MILLIS: u64 = 10_000;

/// An anvil-zksync CLI instance. Will close the instance when dropped.
///
/// Construct this using [`AnvilZKsync`].
#[derive(Debug)]
pub struct AnvilZKsyncInstance {
    child: Child,
    private_keys: Vec<K256SecretKey>,
    addresses: Vec<Address>,
    port: u16,
    chain_id: Option<ChainId>,
}

impl AnvilZKsyncInstance {
    /// Returns a reference to the child process.
    pub const fn child(&self) -> &Child {
        &self.child
    }

    /// Returns a mutable reference to the child process.
    pub fn child_mut(&mut self) -> &mut Child {
        &mut self.child
    }

    /// Returns the private keys used to instantiate this instance
    pub fn keys(&self) -> &[K256SecretKey] {
        &self.private_keys
    }

    /// Returns the addresses used to instantiate this instance
    pub fn addresses(&self) -> &[Address] {
        &self.addresses
    }

    /// Returns the port of this instance
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Returns the chain of the anvil-zksync instance
    pub fn chain_id(&self) -> ChainId {
        const ANVIL_ZKSYNC_CHAIN_ID: ChainId = 260;
        self.chain_id.unwrap_or(ANVIL_ZKSYNC_CHAIN_ID)
    }

    /// Returns the HTTP endpoint of this instance
    #[doc(alias = "http_endpoint")]
    pub fn endpoint(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    /// Returns the HTTP endpoint url of this instance
    #[doc(alias = "http_endpoint_url")]
    pub fn endpoint_url(&self) -> Url {
        Url::parse(&self.endpoint()).unwrap()
    }
}

impl Drop for AnvilZKsyncInstance {
    fn drop(&mut self) {
        self.child.kill().expect("could not kill anvil-zksync");
    }
}

/// Errors that can occur when working with the [`AnvilZKsync`].
#[derive(Debug, Error)]
pub enum AnvilZKsyncError {
    /// Spawning the anvil-zksync process failed.
    #[error("could not start anvil-zksync: {0}")]
    SpawnError(std::io::Error),

    /// Timed out waiting for a message from anvil-zksync's stderr.
    #[error("timed out waiting for anvil-zksync to spawn; anvil-zksync installed?")]
    Timeout,

    /// A line could not be read from the anvil-zksync stderr.
    #[error("could not read line from anvil-zksync stderr: {0}")]
    ReadLineError(std::io::Error),

    /// The child anvil-zksync process's stderr was not captured.
    #[error("could not get stderr for anvil-zksync child process")]
    NoStderr,

    /// The private key could not be parsed.
    #[error("could not parse private key")]
    ParsePrivateKeyError,

    /// An error occurred while deserializing a private key.
    #[error("could not deserialize private key from bytes")]
    DeserializePrivateKeyError,

    /// The port could not be parsed.
    #[error("could not parse the port")]
    ParsePortError,

    /// An error occurred while parsing a hex string.
    #[error(transparent)]
    FromHexError(#[from] hex::FromHexError),

    /// No private keys were found.
    #[error("no private keys found")]
    NoKeysAvailable,
}

/// Builder for launching `anvil-zksync`.
///
/// # Panics
///
/// If `spawn` is called without `anvil-zksync` being available in the user's $PATH
///
/// # Example
///
/// ```no_run
/// use alloy_zksync::node_bindings::AnvilZKsync;
///
/// let port = 8545u16;
/// let url = format!("http://localhost:{}", port).to_string();
///
/// let anvil_zksync = AnvilZKsync::new()
///     .port(port)
///     .spawn();
///
/// drop(anvil_zksync); // this will kill the instance
/// ```
#[derive(Clone, Debug, Default)]
#[must_use = "This Builder struct does nothing unless it is `spawn`ed"]
pub struct AnvilZKsync {
    program: Option<PathBuf>,
    port: Option<u16>,
    // If the block_time is an integer, f64::to_string() will output without a decimal point
    // which allows this to be backwards compatible.
    block_time: Option<f64>,
    no_mine: bool,
    chain_id: Option<ChainId>,
    mnemonic: Option<String>,
    fork: Option<String>,
    fork_block_number: Option<u64>,
    args: Vec<String>,
    timeout: Option<u64>,
}

impl AnvilZKsync {
    /// Creates an empty AnvilZKsync builder.
    /// The default port is 8545. The mnemonic is chosen randomly.
    ///
    /// # Example
    ///
    /// ```
    /// # use alloy_zksync::node_bindings::AnvilZKsync;
    /// fn a() {
    ///  let anvil_zksync = AnvilZKsync::default().spawn();
    ///
    ///  println!("AnvilZKsync running at `{}`", anvil_zksync.endpoint());
    /// # }
    /// ```
    pub fn new() -> Self {
        let mut self_ = Self::default();
        // Assign a random port so that we can run multiple instances.
        let port = rand::thread_rng().gen_range(8000..16000);
        self_.port = Some(port);
        self_
    }

    /// Creates an AnvilZKsync builder which will execute `anvil-zksync` at the given path.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use alloy_zksync::node_bindings::AnvilZKsync;
    /// fn a() {
    ///  let anvil_zksync = AnvilZKsync::at("~/some/location/anvil-zksync").spawn();
    ///
    ///  println!("AnvilZKsync running at `{}`", anvil_zksync.endpoint());
    /// # }
    /// ```
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self::new().path(path)
    }

    /// Sets the `path` to the `anvil-zksync` cli
    ///
    /// By default, it's expected that `anvil-zksync` is in `$PATH`, see also
    /// [`std::process::Command::new()`]
    pub fn path<T: Into<PathBuf>>(mut self, path: T) -> Self {
        self.program = Some(path.into());
        self
    }

    /// Sets the port which will be used when the `anvil-zksync` instance is launched.
    pub fn port<T: Into<u16>>(mut self, port: T) -> Self {
        self.port = Some(port.into());
        self
    }

    /// Sets the chain_id the `anvil-zksync` instance will use.
    pub const fn chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    /// Sets the no-mine status which will be used when the `era_test_node` instance is launched.
    pub const fn no_mine(mut self) -> Self {
        self.no_mine = true;
        self
    }

    /// Sets the mnemonic which will be used when the `anvil-zksync` instance is launched.
    pub fn mnemonic<T: Into<String>>(mut self, mnemonic: T) -> Self {
        self.mnemonic = Some(mnemonic.into());
        self
    }

    /// Sets the block-time in seconds which will be used when the `anvil-zksync` instance is launched.
    pub const fn block_time(mut self, block_time: u64) -> Self {
        self.block_time = Some(block_time as f64);
        self
    }

    // TODO
    // /// Sets the block-time in sub-seconds which will be used when the `anvil-zksync` instance is launched.
    // /// Older versions of `anvil-zksync` do not support sub-second block times.
    // pub const fn block_time_f64(mut self, block_time: f64) -> Self {
    //     self.block_time = Some(block_time);
    //     self
    // }

    /// Sets the `fork-block-number` which will be used in addition to [`Self::fork`].
    ///
    /// **Note:** if set, then this requires `fork` to be set as well
    pub const fn fork_block_number(mut self, fork_block_number: u64) -> Self {
        self.fork_block_number = Some(fork_block_number);
        self
    }

    /// Sets the `fork` argument to fork from another currently running Ethereum client
    /// at a given block. Input should be the HTTP location and port of the other client,
    /// e.g. `http://localhost:8545`. You can optionally specify the block to fork from
    /// using an @ sign: `http://localhost:8545@1599200`
    pub fn fork<T: Into<String>>(mut self, fork: T) -> Self {
        self.fork = Some(fork.into());
        self
    }

    /// Adds an argument to pass to the `anvil-zksync`.
    pub fn arg<T: Into<String>>(mut self, arg: T) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Adds multiple arguments to pass to the `anvil-zksync`.
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for arg in args {
            self = self.arg(arg);
        }
        self
    }

    /// Sets the timeout which will be used when the `anvil-zksync` instance is launched.
    pub const fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Consumes the builder and spawns `anvil-zksync`.
    ///
    /// # Panics
    ///
    /// If spawning the instance fails at any point.
    #[track_caller]
    pub fn spawn(self) -> AnvilZKsyncInstance {
        self.try_spawn().unwrap()
    }

    /// Consumes the builder and spawns `anvil-zksync`. If spawning fails, returns an error.
    pub fn try_spawn(self) -> Result<AnvilZKsyncInstance, AnvilZKsyncError> {
        let mut cmd = self
            .program
            .as_ref()
            .map_or_else(|| Command::new("anvil-zksync"), Command::new);
        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit());
        // let mut port = self.port.unwrap_or_default();
        // cmd.arg("-p").arg(port.to_string());
        if let Some(port) = self.port {
            cmd.arg("--port").arg(port.to_string());
        }

        if let Some(mnemonic) = self.mnemonic {
            cmd.arg("-m").arg(mnemonic);
        }

        if let Some(chain_id) = self.chain_id {
            cmd.arg("--chain-id").arg(chain_id.to_string());
        }

        if let Some(block_time) = self.block_time {
            cmd.arg("-b").arg(block_time.to_string());
        }

        if self.no_mine {
            cmd.arg("--no-mine");
        }

        cmd.args(self.args);

        if let Some(fork) = self.fork {
            cmd.arg("fork").arg("--network").arg(fork);
            if let Some(fork_block_number) = self.fork_block_number {
                println!("fork_block_number ln 312: {}", fork_block_number);
                cmd.arg("--fork-block-number")
                    .arg(fork_block_number.to_string());
            }
        } else {
            cmd.arg("run");
        }

        let mut child = cmd.spawn().map_err(AnvilZKsyncError::SpawnError)?;

        let stdout = child.stdout.as_mut().ok_or(AnvilZKsyncError::NoStderr)?;

        let start = Instant::now();
        let mut reader = BufReader::new(stdout);

        let mut private_keys = Vec::new();
        let mut addresses = Vec::new();
        let mut chain_id = None;
        let port;
        loop {
            if start
                + Duration::from_millis(self.timeout.unwrap_or(ANVIL_ZKSYNC_STARTUP_TIMEOUT_MILLIS))
                <= Instant::now()
            {
                return Err(AnvilZKsyncError::Timeout);
            }

            let mut line = String::new();
            reader
                .read_line(&mut line)
                .map_err(AnvilZKsyncError::ReadLineError)?;
            tracing::trace!(target: "anvil-zksync", line);
            if let Some(addr) = line.trim().split("Listening on").nth(1) {
                // <Node is ready at 127.0.0.1:8011>
                // parse the actual port
                port = SocketAddr::from_str(addr.trim())
                    .map_err(|_| AnvilZKsyncError::ParsePortError)?
                    .port();
                break;
            }

            // Questionable but OK.
            // Start the internal loop to go over the private keys
            if line.contains("Private Keys") {
                loop {
                    let mut pk_line = String::new();
                    reader
                        .read_line(&mut pk_line)
                        .map_err(AnvilZKsyncError::ReadLineError)?;
                    tracing::trace!(target: "anvil-zksync", pk_line);
                    match pk_line.trim() {
                        "" => break,
                        pk_line => {
                            if pk_line.contains("0x") {
                                let key_str = pk_line.split("0x").nth(1).unwrap();
                                let key_hex =
                                    hex::decode(key_str).map_err(AnvilZKsyncError::FromHexError)?;
                                let key = K256SecretKey::from_bytes((&key_hex[..]).into())
                                    .map_err(|_| AnvilZKsyncError::DeserializePrivateKeyError)?;
                                addresses.push(Address::from_public_key(
                                    SigningKey::from(&key).verifying_key(),
                                ));
                                private_keys.push(key);
                            }
                        }
                    }
                }
            } else if line.contains("Chain ID:") {
                // Chain ID: 260
                if let Ok(chain) = line
                    .split("Chain ID:")
                    .nth(1)
                    .unwrap()
                    .trim()
                    .parse::<u64>()
                {
                    chain_id = Some(chain);
                };
            }
        }

        Ok(AnvilZKsyncInstance {
            child,
            private_keys,
            addresses,
            port,
            chain_id: self.chain_id.or(chain_id),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::providers::{Provider, ProviderBuilder};

    #[test]
    fn can_launch_anvil_zksync() {
        let _ = AnvilZKsync::new().spawn();
    }

    #[test]
    fn can_launch_anvil_zksync_with_custom_port() {
        const PORT: u16 = 7555;
        let anvil_zksync = AnvilZKsync::new().port(PORT).spawn();
        assert_eq!(anvil_zksync.port(), PORT);
    }

    #[test]
    fn assert_block_time_is_natural_number() {
        // This test is to ensure that older versions of era_test_node are supported
        // even though the block time is a f64, it should be passed as a whole number
        let era_test_node = AnvilZKsync::new().block_time(12);
        assert_eq!(era_test_node.block_time.unwrap().to_string(), "12");
        let _ = era_test_node.spawn();
    }

    // #[test]
    // fn can_launch_anvil_zksync_with_sub_seconds_block_time() {
    //     let _ = AnvilZKsync::new().block_time_f64(0.5).spawn();
    // }

    #[tokio::test(flavor = "multi_thread")]
    async fn fork_initializes_correct_chain_id() {
        let chain_id = 92;
        let anvil_zksync = AnvilZKsync::new().chain_id(chain_id).spawn();
        let rpc_url = anvil_zksync.endpoint_url();
        let provider = ProviderBuilder::new().on_http(rpc_url);

        let returned_chain_id = provider.get_chain_id().await.unwrap();

        assert_eq!(returned_chain_id, chain_id);

        drop(anvil_zksync);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore]
    async fn fork_initializes_correct_chain() {
        let anvil_zksync = AnvilZKsync::new().fork("mainnet").spawn();
        let rpc_url = anvil_zksync.endpoint_url();
        let provider = ProviderBuilder::new().on_http(rpc_url);

        let chain_id = provider.get_chain_id().await.unwrap();

        assert_eq!(chain_id, 324);

        drop(anvil_zksync);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn fork_initializes_at_specified_block() {
        let fork_block_number = 62174000;

        let anvil_zksync = AnvilZKsync::new()
            .fork("mainnet")
            .fork_block_number(fork_block_number)
            .spawn();

        let rpc_url = anvil_zksync.endpoint_url();
        let provider = ProviderBuilder::new().on_http(rpc_url);

        // Query the latest block number to verify the fork block number.
        let block_number = provider.get_block_number().await.unwrap();

        assert_eq!(
            block_number, fork_block_number,
            "The node did not fork at the expected block number"
        );

        drop(anvil_zksync);
    }

    #[test]
    fn assert_chain_id_without_rpc() {
        let anvil_zksync = AnvilZKsync::new().spawn();
        assert_eq!(anvil_zksync.chain_id(), 260);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_mnemonic_usage() {
        let test_mnemonic =
            "nasty genius bright property zero practice critic draft turkey cigar option south";

        let anvil_zksync = AnvilZKsync::new().mnemonic(test_mnemonic).spawn();

        let expected_addresses = vec![
            "0xe99f84afb6fcad9ebe0e1970fc7632ec00b3a5dd",
            "0x2d0472332f336d00d71a9055a04315684466b7ab",
            "0x10293d5d0127eaa1838779a54833f2c76a3893db",
            "0x574e479338bb22b856feb3df7296c65247c99a5a",
            "0x68256d3e5eae3ee2bc1cf4172c4fdc1f76d51b4d",
            "0x00c0b6d136ab72156734f08c704704f8130f5062",
            "0xa6004bae3cd480660e17542a83fe164b8e128362",
            "0xa845c4de08761a3d93e0aea1006bfc05de02f6ef",
            "0x76454a9658bec53daee3c5fc1d369ea757ebd5cb",
            "0xcda2c6614a1014d27f6dfd9b8323d688931f69b9",
        ];

        let derived_addresses: Vec<_> = anvil_zksync
            .addresses()
            .iter()
            .map(|address| format!("{:#x}", address))
            .collect();

        assert_eq!(
            derived_addresses, expected_addresses,
            "The derived addresses do not match the expected addresses"
        );

        drop(anvil_zksync);
    }

    #[test]
    fn can_launch_era_test_node_with_no_mine() {
        let anvil_zksync = AnvilZKsync::new().no_mine().spawn();

        drop(anvil_zksync);
    }
}
