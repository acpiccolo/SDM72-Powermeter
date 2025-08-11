//! # SDM72 Modbus Library
//!
//! This library provides a Rust interface for interacting with SDM72 series energy meters
//! via the Modbus protocol. It supports both RTU (serial) and TCP connections, and
//! offers both synchronous and asynchronous clients.
//!
//! ## Features
//!
//! - Read and write meter settings.
//! - Read all measurement values in a single call.
//! - Type-safe data structures for all Modbus registers.
//! - Supports `tokio` for asynchronous operations.
//! - Provides both sync and async clients.
//!
//! ## Usage
//!
//! To use this library, you need to enable the desired client feature flags in your `Cargo.toml`.
//! For example, to use the synchronous RTU client:
//!
//! ```toml
//! [dependencies]
//! sdm72 = { version = "0.1.0", features = ["tokio-rtu-sync"] }
//! ```
//!
//! ### Example: Reading all values from an RTU meter
//!
//! ```no_run
//! use sdm72_lib::{tokio_sync_client::SDM72, protocol::BaudRate};
//! use std::time::Duration;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let builder = sdm72_lib::tokio_serial::serial_port_builder("/dev/ttyUSB0", &BaudRate::B9600, &Default::default());
//!     let slave = tokio_modbus::Slave(1);
//!     let ctx = tokio_modbus::client::sync::rtu::connect_slave(&builder, slave)?;
//!     let mut client = SDM72::new(ctx);
//!
//!     let all_values = client.read_all(&Duration::from_millis(100))?;
//!     println!("{:#?}", all_values);
//!
//!     Ok(())
//! }
//! ```

mod error;

pub use error::Error;
pub mod protocol;

#[cfg(any(
    feature = "tokio-rtu-sync",
    feature = "tokio-tcp-sync",
    feature = "tokio-rtu",
    feature = "tokio-tcp"
))]
pub mod tokio_common;

#[cfg(any(feature = "tokio-rtu-sync", feature = "tokio-tcp-sync"))]
pub mod tokio_sync_client;

#[cfg(any(feature = "tokio-rtu", feature = "tokio-tcp"))]
pub mod tokio_async_client;

#[cfg(any(feature = "tokio-rtu", feature = "tokio-rtu-sync"))]
pub mod tokio_serial;
