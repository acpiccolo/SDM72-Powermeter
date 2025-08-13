//! # SDM72 Modbus Library
//!
//! This library provides a comprehensive Rust interface for interacting with
//! Eastron SDM72 series energy meters via the Modbus protocol. It is designed
//! to be flexible and robust, supporting both RTU (serial) and TCP connections
//! with both synchronous and asynchronous clients.
//!
//! ## Key Features
//!
//! - **Multiple Client Flavors**: Choose the client that best fits your application's needs:
//!   - Synchronous RTU (`tokio-rtu-sync`)
//!   - Asynchronous RTU (`tokio-rtu`)
//!   - Synchronous TCP (`tokio-tcp-sync`)
//!   - Asynchronous TCP (`tokio-tcp`)
//! - **Type-Safe**: All Modbus registers are represented by type-safe data structures.
//! - **Efficient Batch Operations**: Read all meter settings or all measurement
//!   values in a single, efficient call.
//! - **Asynchronous Support**: Built on `tokio` for modern, non-blocking I/O.
//!
//! ## Getting Started
//!
//! To use this library, you must enable one of the client feature flags in your
//! `Cargo.toml`. The client you choose determines how you will connect to the
//! meter.
//!
//! ### Choosing a Client
//!
//! - **RTU vs. TCP**: Choose `rtu` if your meter is connected via a serial port
//!   (e.g., RS485 to USB adapter). Choose `tcp` if your meter is connected to an
//!   Ethernet-to-Serial gateway.
//! - **Sync vs. Async**: Choose `sync` for simpler, blocking operations, which
//!   are suitable for applications that do not require concurrency. Choose `async`
//!   if you are using the `tokio` runtime and need non-blocking I/O.
//!
//! The available feature flags are:
//! - `tokio-rtu-sync`: Enables the synchronous RTU client in [`tokio_sync_client`].
//! - `tokio-rtu`: Enables the asynchronous RTU client in [`tokio_async_client`].
//! - `tokio-tcp-sync`: Enables the synchronous TCP client in [`tokio_sync_client`].
//! - `tokio-tcp`: Enables the asynchronous TCP client in [`tokio_async_client`].
//!
//! For example, to use the synchronous RTU client:
//!
//! ```toml
//! [dependencies]
//! sdm72 = { version = "0.1.0", features = ["tokio-rtu-sync"] }
//! ```
//!
//! ## Examples
//!
//! Here are some examples of how to use the different clients.
//!
//! ### Example: Synchronous RTU Client (`tokio-rtu-sync`)
//!
//! ```no_run
//! use sdm72_lib::{protocol::BaudRate, tokio_sync_client::SDM72};
//! use std::time::Duration;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let builder = sdm72_lib::tokio_common::serial_port_builder(
//!         "/dev/ttyUSB0", // Or "COM3" on Windows, etc.
//!         &BaudRate::B9600,
//!         &Default::default(),
//!     );
//!     let slave = tokio_modbus::Slave(1);
//!     let ctx = tokio_modbus::client::sync::rtu::connect_slave(&builder, slave)?;
//!     let mut client = SDM72::new(ctx);
//!
//!     // Set a timeout for the Modbus context
//!     client.set_timeout(Duration::from_secs(1));
//!
//!     // Read all measurement values
//!     let all_values = client.read_all(&Duration::from_millis(100))?;
//!     println!("{:#?}", all_values);
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Example: Asynchronous RTU Client (`tokio-rtu`)
//!
//! ```no_run
//! use sdm72_lib::{protocol::BaudRate, tokio_async_client::SDM72};
//! use std::time::Duration;
//! use tokio_modbus::prelude::*;
//! use tokio_serial::SerialStream;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let builder = sdm72_lib::tokio_common::serial_port_builder(
//!         "/dev/ttyUSB0", // Or "COM3" on Windows, etc.
//!         &BaudRate::B9600,
//!         &Default::default(),
//!     );
//!     let slave = tokio_modbus::Slave(1);
//!     let port = SerialStream::open(&builder)?;
//!     let mut ctx = rtu::attach_slave(port, slave);
//!     let mut client = SDM72::new(ctx);
//!
//!     // Read all measurement values
//!     let all_values = client.read_all(&Duration::from_millis(100)).await?;
//!     println!("{:#?}", all_values);
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Example: Synchronous TCP Client (`tokio-tcp-sync`)
//!
//! ```no_run
//! use sdm72_lib::tokio_sync_client::SDM72;
//! use std::time::Duration;
//! use tokio_modbus::prelude::*;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let socket_addr = "127.0.0.1:502".parse()?;
//!     let slave = tokio_modbus::Slave(1);
//!     let ctx = sync::tcp::connect_slave(socket_addr, slave)?;
//!     let mut client = SDM72::new(ctx);
//!
//!     // Set a timeout for the Modbus context
//!     client.set_timeout(Duration::from_secs(1));
//!
//!     // Read all measurement values
//!     let all_values = client.read_all(&Duration::from_millis(100))?;
//!     println!("{:#?}", all_values);
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Example: Asynchronous TCP Client (`tokio-tcp`)
//!
//! ```no_run
//! use sdm72_lib::tokio_async_client::SDM72;
//! use std::time::Duration;
//! use tokio_modbus::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let socket_addr = "127.0.0.1:502".parse()?;
//!     let slave = tokio_modbus::Slave(1);
//!     let mut ctx = tcp::connect_slave(socket_addr, slave).await?;
//!     let mut client = SDM72::new(ctx);
//!
//!     // Read all measurement values
//!     let all_values = client.read_all(&Duration::from_millis(100)).await?;
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
