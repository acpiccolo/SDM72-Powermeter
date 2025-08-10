use anyhow::{Context, Result};
use clap::Parser;
use flexi_logger::{Logger, LoggerHandle};
use log::*;
use sdm72_lib::{protocol as proto, tokio_sync_client::SDM72};
use std::{ops::Deref, panic, time::Duration};

mod commandline;
mod mqtt;

fn logging_init(loglevel: LevelFilter) -> LoggerHandle {
    let log_handle = Logger::try_with_env_or_str(loglevel.as_str())
        .expect("Cannot init logging")
        .start()
        .expect("Cannot start logging");

    panic::set_hook(Box::new(|panic_info| {
        let (filename, line, column) = panic_info
            .location()
            .map(|loc| (loc.file(), loc.line(), loc.column()))
            .unwrap_or(("<unknown>", 0, 0));
        let cause = panic_info
            .payload()
            .downcast_ref::<String>()
            .map(String::deref);
        let cause = cause.unwrap_or_else(|| {
            panic_info
                .payload()
                .downcast_ref::<&str>()
                .copied()
                .unwrap_or("<cause unknown>")
        });

        error!(
            "Thread '{}' panicked at {}:{}:{}: {}",
            std::thread::current().name().unwrap_or("<unknown>"),
            filename,
            line,
            column,
            cause
        );
    }));
    log_handle
}

fn minimum_rtu_delay(baud_rate: &proto::BaudRate) -> Duration {
    // https://minimalmodbus.readthedocs.io/en/stable/serialcommunication.html#timing-of-the-serial-communications
    let rate = u16::from(baud_rate) as f64;
    let bit_time = Duration::from_secs_f64(1.0 / rate);
    let char_time = bit_time * 11;
    let result = Duration::from_millis((char_time.as_secs_f64() * 3.5 * 1_000.0) as u64);
    let min_duration = Duration::from_micros(1_750);
    if result < min_duration {
        min_duration
    } else {
        result
    }
}

fn check_rtu_delay(delay: Duration, baud_rate: &proto::BaudRate) -> Duration {
    let min_rtu_delay = minimum_rtu_delay(baud_rate);
    if delay < min_rtu_delay {
        warn!(
            "Your RTU delay of {delay:?} is below the minimum delay of {min_rtu_delay:?}, fallback to minimum"
        );
        return min_rtu_delay;
    }
    delay
}

fn ensure_authorization(d: &mut SDM72) -> Result<()> {
    if proto::KPPA::Authorized != d.kppa().with_context(|| "Cannot get authorization")? {
        let passwd = dialoguer::Input::new()
            .with_prompt("Authorization is required, please enter password")
            .validate_with(|input: &String| -> Result<(), String> {
                commandline::parse_password(input)?;
                Ok(())
            })
            .default(proto::Password::default().to_string())
            .interact_text()
            .unwrap();
        d.set_kppa(commandline::parse_password(&passwd).unwrap())
            .with_context(|| "Authorization failed")?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = commandline::Args::parse();

    let mut delay = args.delay;

    let _log_handle = logging_init(args.verbose.log_level_filter());

    let (mut d, command) = match &args.connection {
        commandline::Connection::Tcp { address, command } => {
            let socket_addr = address
                .parse()
                .with_context(|| format!("Cannot parse address {address}"))?;
            trace!("Open TCP address {socket_addr}");
            (
                SDM72::new(
                    tokio_modbus::client::sync::tcp::connect(socket_addr)
                        .with_context(|| format!("Cannot open {socket_addr:?}"))?,
                ),
                command,
            )
        }
        commandline::Connection::Rtu {
            device,
            baud_rate,
            address,
            parity_and_stop_bit: parity_and_stop_bits,
            command,
        } => {
            trace!(
                "Open RTU {device} address {address} baud rate {baud_rate} parity and stop bits {parity_and_stop_bits}"
            );
            delay = check_rtu_delay(delay, baud_rate);
            (
                SDM72::new(
                    tokio_modbus::client::sync::rtu::connect_slave(
                        &sdm72_lib::tokio_serial::serial_port_builder(
                            device,
                            baud_rate,
                            parity_and_stop_bits,
                        ),
                        tokio_modbus::Slave(**address),
                    )
                    .with_context(|| {
                        format!("Cannot open device {device} baud rate {baud_rate}")
                    })?,
                ),
                command,
            )
        }
    };
    d.set_timeout(args.timeout);

    match command {
        commandline::Commands::Daemon { poll_iterval, mode } => match mode {
            commandline::DaemonOutput::Console => loop {
                let values = d
                    .read_all(&delay)
                    .with_context(|| "Cannot read all values")?;
                if args.no_json {
                    println!("{values}");
                } else {
                    println!("{}", serde_json::to_string_pretty(&values)?);
                }
                std::thread::sleep(delay.max(*poll_iterval));
            },
            commandline::DaemonOutput::Mqtt { config_file } => {
                mqtt::run_mqtt_daemon(&mut d, &delay, poll_iterval, config_file, args.no_json)?;
            }
        },
        commandline::Commands::ReadAll => {
            let values = d
                .read_all(&delay)
                .with_context(|| "Cannot read all values")?;
            if args.no_json {
                println!("{values}");
            } else {
                println!("{}", serde_json::to_string_pretty(&values)?);
            }
        }
        commandline::Commands::ReadAllSettings => {
            let settings = d
                .read_all_settings(&delay)
                .with_context(|| "Cannot read all settings")?;
            if args.no_json {
                println!("{settings}");
            } else {
                println!("{}", serde_json::to_string_pretty(&settings)?);
            }
        }

        commandline::Commands::Password { password } => {
            d.set_kppa(*password)
                .with_context(|| "Cannot set authorization")?;
        }
        commandline::Commands::SetWiringType { wiring_type } => {
            ensure_authorization(&mut d)?;
            d.set_system_type(**wiring_type)
                .with_context(|| "Cannot set wiring type")?;
            println!("Wiring type successfully changed to: {}", **wiring_type);
        }
        commandline::Commands::SetParityAndStopBit {
            parity_and_stop_bit,
        } => {
            ensure_authorization(&mut d)?;
            d.set_parity_and_stop_bit(**parity_and_stop_bit)
                .with_context(|| "Cannot set parity and stop bit")?;
            println!(
                "Parity and stop bit successfully changed to: {}",
                **parity_and_stop_bit
            );
        }
        commandline::Commands::SetBaudRate { baud_rate } => {
            ensure_authorization(&mut d)?;
            d.set_baud_rate(*baud_rate)
                .with_context(|| "Cannot set baud rate")?;
            println!("Baud rate successfully changed to: {baud_rate}");
        }
        commandline::Commands::SetAddress { address } => {
            ensure_authorization(&mut d)?;
            d.set_address(*address)
                .with_context(|| "Cannot set RS485 address")?;
            println!("Address successfully changed to: {address}");
        }
        commandline::Commands::SetPulseConstant {
            pulse_constant_in_kwh,
        } => {
            ensure_authorization(&mut d)?;
            d.set_pulse_constant(**pulse_constant_in_kwh)
                .with_context(|| "Cannot set pulse constant")?;
            println!(
                "Pulse constant successfully changed to: {}",
                **pulse_constant_in_kwh
            );
        }
        commandline::Commands::SetPassword { password } => {
            ensure_authorization(&mut d)?;
            d.set_password(*password)
                .with_context(|| "Cannot set password")?;
            println!("Password successfully changed to: {password}");
        }
        commandline::Commands::SetAutoScrollTime {
            auto_scroll_time_in_seconds,
        } => {
            ensure_authorization(&mut d)?;
            d.set_auto_scroll_time(*auto_scroll_time_in_seconds)
                .with_context(|| "Cannot set auto scroll time")?;
            println!(
                "Auto scroll time successfully changed to: {auto_scroll_time_in_seconds}"
            );
        }
        commandline::Commands::SetBacklightTime {
            backlight_time_in_minutes,
        } => {
            ensure_authorization(&mut d)?;
            d.set_backlight_time(*backlight_time_in_minutes)
                .with_context(|| "Cannot set backlinght time")?;
            println!(
                "Backlight time successfully changed to: {backlight_time_in_minutes}"
            );
        }
        commandline::Commands::SetPulseEnergyType { pulse_energy_type } => {
            ensure_authorization(&mut d)?;
            d.set_pulse_energy_type(**pulse_energy_type)
                .with_context(|| "Cannot set pulse energy type")?;
            println!(
                "Pulse energy type successfully changed to: {}",
                **pulse_energy_type
            );
        }
        commandline::Commands::ResetHistoricalData => {
            ensure_authorization(&mut d)?;
            d.reset_historical_data()
                .with_context(|| "Cannot reset historical data")?;
            println!("Historical data successfully reset",);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtu_delay() {
        assert_eq!(minimum_rtu_delay(&proto::BaudRate::B1200).as_millis(), 32);
        assert_eq!(minimum_rtu_delay(&proto::BaudRate::B2400).as_millis(), 16);
        assert_eq!(minimum_rtu_delay(&proto::BaudRate::B4800).as_millis(), 8);
        assert_eq!(minimum_rtu_delay(&proto::BaudRate::B9600).as_millis(), 4);
        assert_eq!(minimum_rtu_delay(&proto::BaudRate::B19200).as_millis(), 2);
    }
}
