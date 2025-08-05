[![CI](https://github.com/acpiccolo/SDM72-Powermeter/actions/workflows/check.yml/badge.svg)](https://github.com/acpiccolo/SDM72-Powermeter/actions/workflows/check.yml)
[![dependency status](https://deps.rs/repo/github/acpiccolo/SDM72-Powermeter/status.svg)](https://deps.rs/repo/github/acpiccolo/SDM72-Powermeter)
[![CI](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/acpiccolo/SDM72-Powermeter/blob/main/LICENSE-MIT)
[![CI](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://github.com/acpiccolo/SDM72-Powermeter/blob/main/LICENSE-APACHE)
[![CI](https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg)](https://conventionalcommits.org)

# SDM72 three phase four wire energy meter
This RUST project can read and write a SDM72 energy meter from the command line.

## Hardware
The following hardware is required for this project:
* One or more SDM72 energy meter.
* One USB-RS485 converter.

### Data sheet SDM72 energy meter
* maximum 100 Ampere
* Two wire types: 3 phase with 4 wire or 1 phase with 2 wire
* RS485 Modbus RTU output
* Pulse Output
* Bi-directional measurement (import & export)


## Compilation
1. Install Rust e.g. using [these instructions](https://www.rust-lang.org/learn/get-started).
2. Ensure that you have a C compiler and linker.
3. Clone `git clone https://github.com/acpiccolo/SDM72-Powermeter.git`
4. Run `cargo install --path .` to install the binary. Alternatively,
   check out the repository and run `cargo build --release`. This will compile
   the binary to `target/release/sdm72`.

## Getting started
To see all available commands:
```
sdm72 --help
```
For RTU Modbus connected sdm72 energy meter:
```
sdm72 rtu --address 1 --baudrate 9600 read-all
```
For TCP Modbus connected sdm72 energy meter:
```
sdm72 tcp 192.168.0.222:502 read-all
```
You can even use this tool as a daemon for a MQTT broker, the connection is configured via the `mqtt.yaml` file:
```
sdm72 rtu --address 1 --baudrate 9600 daemon mqtt
```

### Cargo Features
| Feature | Purpose | Default |
| :--- | :------ | :-----: |
| `tokio-rtu-sync` | Enable the implementation for the tokio modbus synchronous RTU client | ✅ |
| `tokio-rtu` | Enable the implementation for the tokio modbus asynchronous RTU client | ✅ |
| `tokio-tcp-sync` | Enable the implementation for the tokio modbus synchronous TCP client | - |
| `tokio-tcp` | Enable the implementation for the tokio modbus asynchronous TCP client | - |
| `bin-dependencies` | Enable all features required by the binary | ✅ |


## License
Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
