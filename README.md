[![CI](https://github.com/acpiccolo/SDM72-Powermeter/actions/workflows/check.yml/badge.svg)](https://github.com/acpiccolo/SDM72-Powermeter/actions/workflows/check.yml)
[![dependency status](https://deps.rs/repo/github/acpiccolo/SDM72-Powermeter/status.svg)](https://deps.rs/repo/github/acpiccolo/SDM72-Powermeter)
[![CI](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/acpiccolo/SDM72-Powermeter/blob/main/LICENSE-MIT)
[![CI](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://github.com/acpiccolo/SDM72-Powermeter/blob/main/LICENSE-APACHE)
[![CI](https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg)](https://conventionalcommits.org)

# SDM72 Modbus Library and Tool

This repository contains a Rust library and a command-line tool for interacting with Eastron SDM72 series energy meters via the Modbus protocol.

## Table of Contents
- [Hardware Requirements](#hardware-requirements)
- [Technical Specifications](#technical-specifications)
- [Installation & Compilation](#installation--compilation)
- [Command-Line Usage](#command-line-usage)
- [Library Usage](#library-usage)
- [Cargo Features](#cargo-features)
- [License](#license)

## Hardware Requirements
To use this tool, you need:
- An **Eastron SDM72 series energy meter**.
- A **USB-to-RS485 converter** (for RTU mode) or a **Modbus TCP gateway**.

## Technical Specifications
| Feature | Details |
|---|---|
| **Nominal Voltage** | 230/400V AC (3~) |
| **Operational Voltage** | 80%~120% of nominal voltage |
| **Current Measurement** | Up to 100A direct connection |
| **Communication Protocol** | Modbus RTU/TCP |
| **Baud Rates** | 2400, 4800, 9600, 19200, 38400 |
| **Data Format** | N, 8, 1 (No parity, 8 data bits, 1 stop bit) |

## Installation & Compilation

### Prerequisites
Ensure you have the following dependencies installed before proceeding:
- **Rust and Cargo**: Install via [rustup](https://rustup.rs/)
- **Git**: To clone the repository
- A C compiler and linker

### Building from Source
1. **Clone the repository**:
   ```sh
   git clone https://github.com/acpiccolo/SDM72-Powermeter.git
   cd SDM72-Powermeter
   ```
2. **Compile the project**:
   ```sh
   cargo build --release
   ```
   The compiled binary will be available at:
   ```sh
   target/release/sdm72
   ```
3. **(Optional) Install the binary system-wide**:
   ```sh
   cargo install --path .
   ```
   This installs `sdm72` to `$HOME/.cargo/bin`, making it accessible from anywhere.

## Command-Line Usage
### View Available Commands
To list all available commands and their options, run:
```sh
sdm72 --help
```
### Read All Values
For **RTU Modbus (RS485) connected** devices:
```sh
sdm72 rtu --address 1 --baudrate 9600 read-all
```
For **TCP Modbus connected** devices:
```sh
sdm72 tcp 192.168.0.222:502 read-all
```
### Daemon Mode with MQTT
You can also run the tool as a daemon that publishes data to an MQTT broker. The connection is configured via an `mqtt.yaml` file.
```sh
sdm72 rtu --address 1 --baudrate 9600 daemon mqtt
```

## Library Usage
The `sdm72_lib` crate provides two main ways to interact with the SDM72 energy meters:

1.  **High-Level, Safe Clients**: Stateful, thread-safe clients that are easy to share and use in concurrent applications. This is the recommended approach for most users. See `tokio_sync_safe_client::SafeClient` (blocking) and `tokio_async_safe_client::SafeClient` (`async`).

2.  **Low-Level, Stateless Functions**: A set of stateless functions that directly map to the device's Modbus commands. This API offers maximum flexibility but requires manual management of the Modbus context. See the `tokio_sync` and `tokio_async` modules.

### Features
- **Protocol Implementation**: Complete implementation of the SDM72 Modbus protocol.
- **Stateful, Thread-Safe Clients**: For easy and safe concurrent use.
- **Stateless, Low-Level Functions**: For maximum flexibility and control.
- **Synchronous and Asynchronous APIs**: Both blocking and `async/await` APIs are available.
- **Strongly-Typed API**: Utilizes Rust's type system for protocol correctness.

### Quick Start
This example shows how to use the recommended high-level, synchronous `SafeClient`.

```rust
use sdm72_lib::{
    protocol::Address,
    tokio_sync_safe_client::SafeClient,
};
use tokio_modbus::client::sync::tcp;
use tokio_modbus::Slave;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the device and create a stateful, safe client
    let socket_addr = "192.168.1.100:502".parse()?;
    let ctx = tcp::connect_slave(socket_addr, Slave(*Address::default()))?;
    let mut client = SafeClient::new(ctx);

    // Use the client to interact with the device
    let values = client.read_all(&Duration::from_millis(100))?;

    println!("Successfully read values: {:#?}", values);

    Ok(())
}
```

## Cargo Features
| Feature | Purpose | Default |
| :--- | :------ | :-----: |
| `tokio-rtu-sync` | Enable the implementation for the tokio modbus synchronous RTU client | - |
| `tokio-rtu` | Enable the implementation for the tokio modbus asynchronous RTU client | - |
| `tokio-tcp-sync` | Enable the implementation for the tokio modbus synchronous TCP client | - |
| `tokio-tcp` | Enable the implementation for the tokio modbus asynchronous TCP client | - |
| `bin-dependencies` | Enable all features required by the binary | ✅ |

## License
Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
