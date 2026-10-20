[![CI](https://github.com/acpiccolo/SDM72-Powermeter/actions/workflows/check.yml/badge.svg)](https://github.com/acpiccolo/SDM72-Powermeter/actions/workflows/check.yml)
[![dependency status](https://deps.rs/repo/github/acpiccolo/SDM72-Powermeter/status.svg)](https://deps.rs/repo/github/acpiccolo/SDM72-Powermeter)
[![CI](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/acpiccolo/SDM72-Powermeter/blob/main/LICENSE-MIT)
[![CI](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://github.com/acpiccolo/SDM72-Powermeter/blob/main/LICENSE-APACHE)
[![CI](https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg)](https://conventionalcommits.org)

# SDM72 Modbus Library and Tool

This repository contains a Rust library and a command-line tool for interacting with Eastron SDM72 series energy meters via the Modbus protocol.

## Table of Contents
- [Hardware Requirements](#hardware-requirements)
- [Technical Documentation](#technical-documentation)
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

## Technical Documentation
For more detailed information, please refer to the official datasheets available in the [`docs/`](./docs/) directory:
- [`eastron_sdm72dmv2.pdf`](./docs/eastron_sdm72dmv2.pdf)

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
You can also run the tool as a daemon that publishes data to an MQTT broker.

- **Log to Console:**
  ```sh
  sdm72 rtu --address 1 daemon console
  ```

- **Publish to MQTT:**
  ```sh
  sdm72 rtu --address 1 daemon mqtt --mqtt-config /path/to/mqtt.yaml
  ```
  If `--mqtt-config` is omitted, `mqtt.yaml` is loaded from the current directory.

### MQTT Configuration (mqtt.yaml & Environment Variables)

All MQTT connection parameters are configured through a YAML file. This approach centralizes settings like the broker URL, credentials, topic, QoS, and client ID.

**Path Specification:**
- Use the `--mqtt-config <PATH>` option to specify the path to your MQTT configuration file. For example:
  ```sh
  sdm72 rtu --address 1 daemon mqtt --mqtt-config /path/to/your/mqtt.yaml
  ```
- If the `--mqtt-config <PATH>` option is **not** provided, the application will attempt to load `mqtt.yaml` from the current working directory.

If the specified configuration file (either via option or the default `mqtt.yaml`) is not found, cannot be read, or is improperly formatted, the program will exit with an error.

**Mandatory Fields:**
- The `uri` field within the MQTT configuration YAML file is **mandatory**.

**Configuration Details:**

The YAML file allows you to set the following parameters:
- `uri` (String, Mandatory): URI of the MQTT broker (e.g., "tcp://localhost:1883").
- `username` (String, Optional): Username for MQTT broker authentication.
- `password` (String, Optional): Password for MQTT broker authentication.
- `topic` (String, Optional): Base MQTT topic. Defaults to "sdm72" if not set.
- `qos` (Integer, Optional): MQTT Quality of Service level (0, 1, or 2). Defaults to 0 if not set.
- `client_id` (String, Optional): Client ID for the MQTT connection. If omitted, a random ID (e.g., "sdm72-<random_suffix>") is generated.
- `oparation_timeout` (Duration, Optional): Timeout for MQTT operations (e.g., "10s").
- `keep_alive_interval` (Duration, Optional): MQTT keep-alive ping interval (e.g., "30s").
- `auto_reconnect_interval_min` (Duration, Optional): Minimum automatic reconnect backoff interval (e.g., "1s").
- `auto_reconnect_interval_max` (Duration, Optional): Maximum automatic reconnect backoff interval (e.g., "30s").

**Example `mqtt.yaml`:**

```yaml
# Sample MQTT Configuration for SDM72 Powermeter CLI

# URI of the MQTT broker (Mandatory).
# Example: "tcp://localhost:1883" or "mqtts://secure-broker.com:8883"
uri: "tcp://localhost:1883"

# Username for MQTT broker authentication (optional).
# username: "your_username"

# Password for MQTT broker authentication (optional).
# password: "your_password"

# Base MQTT topic to publish measurements to.
# Defaults to "sdm72".
topic: "sdm72/house/powermeter"

# MQTT Quality of Service (QoS) level for publishing messages.
# 0: At most once, 1: At least once, 2: Exactly once.
# Defaults to 0.
qos: 1

# Client ID for the MQTT connection (optional).
# If not provided, a random ID will be generated by the client library.
# client_id: "sdm72-powermeter-main"
```

#### Environment Variables Override

You can also override or specify configuration values via environment variables (useful for containerized environments or avoiding plain-text credentials in configuration files):

* `MQTT_URI`: Overrides `uri`
* `MQTT_USERNAME`: Overrides `username`
* `MQTT_PASSWORD`: Overrides `password`
* `MQTT_TOPIC`: Overrides `topic`
* `MQTT_CLIENT_ID`: Overrides `client_id`

## Library Usage
The `sdm72_lib` crate provides two main ways to interact with the SDM72 energy meters:

1.  **High-Level, Safe Clients**: Stateful, thread-safe clients that are easy to share and use in concurrent applications. This is the recommended approach for most users. See `tokio_sync_safe_client::SafeClient` (blocking) and `tokio_async_safe_client::SafeClient` (`async`).

2.  **Low-Level, Stateless Functions**: A set of stateless functions that directly map to the device's Modbus commands. This API offers maximum flexibility but requires manual management of the Modbus context. See the `tokio_sync` and `tokio_async` modules.

### Quick Start: Synchronous Client

Here's a quick example of how to use the synchronous `SafeClient` to read all values over a TCP connection.

#### Dependencies

First, add the required dependencies to your project:
```sh
cargo add SDM72@0.2 --no-default-features --features "tokio-tcp-sync,safe-client-sync,serde"
cargo add tokio-modbus@0.16
cargo add tokio@1 --features full
```

#### Example Usage

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

This crate uses a feature-based system to minimize dependencies. When using it as a library, you should disable default features and select only the components you need.

- **`default`**: Enables `bin-dependencies`, intended for compiling the `sdm72` command-line tool.

### Client Features
- **`tokio-rtu-sync`**: Synchronous (blocking) RTU client.
- **`tokio-tcp-sync`**: Synchronous (blocking) TCP client.
- **`tokio-rtu`**: Asynchronous (non-blocking) RTU client.
- **`tokio-tcp`**: Asynchronous (non-blocking) TCP client.

### High-Level Wrappers
- **`safe-client-sync`**: A thread-safe, stateful wrapper for synchronous clients.
- **`safe-client-async`**: A thread-safe, stateful wrapper for asynchronous clients.

### Utility Features
- **`serde`**: Implements `serde::Serialize` and `serde::Deserialize` for protocol structs.
- **`bin-dependencies`**: All features required to build the `sdm72` binary.

## License
Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
