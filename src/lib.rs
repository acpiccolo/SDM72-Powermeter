#![cfg_attr(docsrs, feature(doc_cfg))]
//! A library for controlling the SDM72 series energy meters via Modbus.
//!
//! This crate provides two main ways to interact with the SDM72 energy meters:
//!
//! 1.  **High-Level, Safe Clients**: Stateful, thread-safe clients that are easy
//!     to share and use in concurrent applications. This is the recommended
//!     approach for most users. See [`tokio_sync_safe_client::SafeClient`] (blocking)
//!     and [`tokio_async_safe_client::SafeClient`] (`async`).
//!
//! 2.  **Low-Level, Stateless Functions**: A set of stateless functions that
//!     directly map to the device's Modbus commands. This API offers maximum
//!     flexibility but requires manual management of the Modbus context. See
//!     the [`tokio_sync`] and [`tokio_async`] modules.
//!
//! ## Features
//!
//! - **Protocol Implementation**: Complete implementation of the SDM72 Modbus protocol.
//! - **Stateful, Thread-Safe Clients**: For easy and safe concurrent use.
//! - **Stateless, Low-Level Functions**: For maximum flexibility and control.
//! - **Synchronous and Asynchronous APIs**: Both blocking and `async/await` APIs are available.
//! - **Strongly-Typed API**: Utilizes Rust's type system for protocol correctness.
//!
//! ## Quick Start
//!
//! This example shows how to use the recommended high-level, synchronous `SafeClient`.
//!
//! ```no_run
//! use sdm72_lib::{
//!     protocol::Address,
//!     tokio_sync_safe_client::SafeClient,
//! };
//! use tokio_modbus::client::sync::tcp;
//! use tokio_modbus::Slave;
//! use std::time::Duration;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to the device and create a stateful, safe client
//!     let socket_addr = "192.168.1.100:502".parse()?;
//!     let ctx = tcp::connect_slave(socket_addr, Slave(*Address::default()))?;
//!     let mut client = SafeClient::new(ctx);
//!
//!     // Use the client to interact with the device
//!     let values = client.read_all(&Duration::from_millis(100))?;
//!
//!     println!("Successfully read values: {:#?}", values);
//!
//!     Ok(())
//! }
//! ```
//!
//! For more details, see the documentation for the specific client you wish to use.

pub mod protocol;

#[cfg_attr(docsrs, doc(cfg(feature = "tokio-rtu-sync")))]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-tcp-sync")))]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-rtu")))]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-tcp")))]
#[cfg(any(
    feature = "tokio-rtu-sync",
    feature = "tokio-tcp-sync",
    feature = "tokio-rtu",
    feature = "tokio-tcp"
))]
pub mod tokio_common;

#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "tokio-rtu-sync", feature = "tokio-tcp-sync")))
)]
#[cfg(any(feature = "tokio-rtu-sync", feature = "tokio-tcp-sync"))]
pub mod tokio_sync;

#[cfg_attr(docsrs, doc(cfg(any(feature = "tokio-rtu", feature = "tokio-tcp"))))]
#[cfg(any(feature = "tokio-rtu", feature = "tokio-tcp"))]
pub mod tokio_async;

#[cfg_attr(
    docsrs,
    doc(cfg(all(
        feature = "safe-client-sync",
        any(feature = "tokio-rtu-sync", feature = "tokio-tcp-sync")
    )))
)]
#[cfg(all(
    feature = "safe-client-sync",
    any(feature = "tokio-rtu-sync", feature = "tokio-tcp-sync")
))]
pub mod tokio_sync_safe_client;

#[cfg_attr(
    docsrs,
    doc(cfg(all(
        feature = "safe-client-async",
        any(feature = "tokio-rtu", feature = "tokio-tcp")
    )))
)]
#[cfg(all(
    feature = "safe-client-async",
    any(feature = "tokio-rtu", feature = "tokio-tcp")
))]
pub mod tokio_async_safe_client;
