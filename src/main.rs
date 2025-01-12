use anyhow::{Context, Result};
use clap::Parser;
use flexi_logger::{Logger, LoggerHandle};
use log::*;
use paho_mqtt as mqtt;
use sdm72_lib::{protocol as proto, tokio_sync_client::SDM72};
use serde::{Deserialize, Serialize};
use std::{ops::Deref, panic, time::Duration};

mod commandline;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct MqttConfig {
    url: String,
    username: Option<String>,
    password: Option<String>,
    #[serde(default = "MqttConfig::default_repeats")]
    repeats: u32,
    #[serde(default = "MqttConfig::default_timeout", with = "humantime_serde")]
    timeout: Duration,
    #[serde(
        default = "MqttConfig::default_keep_alive_interval",
        with = "humantime_serde"
    )]
    keep_alive_interval: Duration,
    #[serde(default = "MqttConfig::default_topic")]
    topic: String,
    #[serde(default = "MqttConfig::default_qos")]
    qos: u8,
}

impl MqttConfig {
    fn default_repeats() -> u32 {
        1
    }
    fn default_timeout() -> Duration {
        Duration::from_secs(5)
    }
    fn default_keep_alive_interval() -> Duration {
        Duration::from_secs(20)
    }
    fn default_topic() -> String {
        String::from("sdm72")
    }
    fn default_qos() -> u8 {
        0
    }
    const DEFAULT_CONFIG_FILE: &str = "mqtt_config.yml";

    fn load(file_name: &str) -> Result<Self> {
        use std::{fs::File, path::Path};

        let config_file_path = Path::new(file_name);
        if !config_file_path.exists() {
            anyhow::bail!(
                "Cannot open config file '{}'",
                config_file_path.to_string_lossy()
            );
        }
        log::debug!("Loading config file from {:?}", &config_file_path);
        let config_file = File::open(config_file_path)
            .with_context(|| format!("Cannot open config file {:?}", config_file_path))?;
        let config: Self = serde_yaml::from_reader(&config_file)
            .with_context(|| format!("Cannot read config file {:?}", config_file_path))?;
        drop(config_file);
        Ok(config)
    }
}

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
            "Your RTU delay of {:?} is below the minimum delay of {:?}, fallback to minimum",
            delay, min_rtu_delay
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
                .with_context(|| format!("Cannot parse address {}", address))?;
            trace!("Open TCP address {}", socket_addr);
            (
                SDM72::new(
                    tokio_modbus::client::sync::tcp::connect(socket_addr)
                        .with_context(|| format!("Cannot open {:?}", socket_addr))?,
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
                "Open RTU {} address {} baud rate {} parity and stop bits {}",
                device,
                address,
                baud_rate,
                parity_and_stop_bits
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
                        format!("Cannot open device {} baud rate {}", device, baud_rate)
                    })?,
                ),
                command,
            )
        }
    };
    d.set_timeout(args.timeout);

    match command {
        commandline::Commands::Daemon { poll_iterval, mode } => match mode {
            commandline::DaemonMode::Stdout => loop {
                let values = d
                    .read_all(&delay)
                    .with_context(|| "Cannot read all values")?;
                if args.no_json {
                    println!("{}", values);
                } else {
                    println!("{}", serde_json::to_string_pretty(&values)?);
                }
                std::thread::sleep(delay.max(*poll_iterval));
            },
            commandline::DaemonMode::Mqtt { config_file } => {
                let mqtt_config = MqttConfig::load(config_file)?;

                let mut cli = mqtt::Client::new(mqtt_config.url.clone())
                    .with_context(|| "Error creating MQTT client")?;

                // Use timeouts for sync calls.
                cli.set_timeout(mqtt_config.timeout);

                let mut conn_builder = mqtt::ConnectOptionsBuilder::new();
                let mut conn_builder = conn_builder
                    .keep_alive_interval(mqtt_config.keep_alive_interval)
                    .clean_session(true);

                if let Some(user_name) = mqtt_config.username {
                    conn_builder = conn_builder.user_name(user_name)
                }
                if let Some(password) = mqtt_config.password {
                    conn_builder = conn_builder.password(password)
                }
                let conn_ops = conn_builder.finalize();

                // Connect and wait for it to complete or fail.
                // The default connection uses MQTT v3.x
                cli.connect(conn_ops)
                    .with_context(|| "MQTT client unable to connect")?;

                loop {
                    let values = d
                        .read_all(&delay)
                        .with_context(|| "Cannot read all values")?;

                    macro_rules! pub_msg {
                        ($label:expr, $val:expr) => {
                            cli.publish(mqtt::Message::new(
                                format!("{}/{}", mqtt_config.topic, $label),
                                $val.to_string(),
                                mqtt_config.qos as i32,
                            ))
                            .with_context(|| "Cannot publish MQTT message")?;
                        };
                    }

                    pub_msg!("L1_Voltage", values.l1_voltage);
                    pub_msg!("L2_Voltage", values.l2_voltage);
                    pub_msg!("L3_Voltage", values.l3_voltage);
                    pub_msg!("L1_Current", values.l1_current);
                    pub_msg!("L2_Current", values.l2_current);
                    pub_msg!("L3_Current", values.l3_current);
                    pub_msg!("L1_Power_Active", values.l1_power_active);
                    pub_msg!("L2_Power_Active", values.l2_power_active);
                    pub_msg!("L3_Power_Active", values.l3_power_active);
                    pub_msg!("L1_Power_Apparent", values.l1_power_apparent);
                    pub_msg!("L2_Power_Apparent", values.l2_power_apparent);
                    pub_msg!("L3_Power_Apparent", values.l3_power_apparent);
                    pub_msg!("L1_Power_Reactive", values.l1_power_reactive);
                    pub_msg!("L2_Power_Reactive", values.l2_power_reactive);
                    pub_msg!("L3_Power_Reactive", values.l3_power_reactive);
                    pub_msg!("L1_Power_Factor", values.l1_power_factor);
                    pub_msg!("L2_Power_Factor", values.l2_power_factor);
                    pub_msg!("L3_Power_Factor", values.l3_power_factor);
                    pub_msg!("L-N_average_Voltage", values.ln_average_voltage);
                    pub_msg!("L-N_average_Current", values.ln_average_current);
                    pub_msg!("Total_Line_Current", values.total_line_current);
                    pub_msg!("Total_Power", values.total_power);
                    pub_msg!("Total_Power_Apparent", values.total_power_apparent);
                    pub_msg!("Total_Power_Reactive", values.total_power_reactive);
                    pub_msg!("Total_Power_Factor", values.total_power_factor);
                    pub_msg!("Frequency", values.frequency);
                    pub_msg!("Import_Energy_Active", values.import_energy_active);
                    pub_msg!("Export_Energy_Active", values.export_energy_active);

                    pub_msg!("L1-L2_Voltage", values.l1l2_voltage);
                    pub_msg!("L2-L3_Voltage", values.l2l3_voltage);
                    pub_msg!("L3-L1_Voltage", values.l3l1_voltage);
                    pub_msg!("L-L_average_Voltage", values.ll_average_voltage);
                    pub_msg!("Neutral_Current", values.neutral_current);

                    pub_msg!("Total_Energy_Active", values.total_energy_active);
                    pub_msg!("Total_Energy_Reactive", values.total_energy_reactive);
                    pub_msg!(
                        "Resettable_Total_Energy_Active",
                        values.resettable_total_energy_active
                    );
                    pub_msg!(
                        "Resettable_Total_Energy_Reactive",
                        values.resettable_total_energy_reactive
                    );
                    pub_msg!(
                        "Resettable_Import_Energy_Active",
                        values.resettable_import_energy_active
                    );
                    pub_msg!(
                        "Resettable_Export_Energy_Active",
                        values.resettable_export_energy_active
                    );
                    pub_msg!("Net_kWh_Import_-_Export", values.net_kwh);

                    pub_msg!(
                        "Import_Total_Energy_Active",
                        values.import_total_energy_active
                    );
                    pub_msg!(
                        "Export_Total_Energy_Active",
                        values.export_total_energy_active
                    );

                    if !args.no_json {
                        let payload = serde_json::to_string(&values)?;
                        let msg = mqtt::Message::new(
                            format!("{}/JSON", mqtt_config.topic),
                            payload,
                            mqtt_config.qos as i32,
                        );
                        cli.publish(msg)
                            .with_context(|| "Cannot publish MQTT message")?;
                    }
                    std::thread::sleep(delay.max(*poll_iterval));
                }
            }
        },
        commandline::Commands::ReadAll => {
            let values = d
                .read_all(&delay)
                .with_context(|| "Cannot read all values")?;
            if args.no_json {
                println!("{}", values);
            } else {
                println!("{}", serde_json::to_string_pretty(&values)?);
            }
        }
        commandline::Commands::ReadAllSettings => {
            let settings = d
                .read_all_settings(&delay)
                .with_context(|| "Cannot read all settings")?;
            if args.no_json {
                println!("{}", settings);
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
            println!("Baud rate successfully changed to: {}", baud_rate);
        }
        commandline::Commands::SetAddress { address } => {
            ensure_authorization(&mut d)?;
            d.set_address(*address)
                .with_context(|| "Cannot set RS485 address")?;
            println!("Address successfully changed to: {}", address);
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
            println!("Password successfully changed to: {}", password);
        }
        commandline::Commands::SetAutoScrollTime {
            auto_scroll_time_in_seconds,
        } => {
            ensure_authorization(&mut d)?;
            d.set_auto_scroll_time(*auto_scroll_time_in_seconds)
                .with_context(|| "Cannot set auto scroll time")?;
            println!(
                "Auto scroll time successfully changed to: {}",
                auto_scroll_time_in_seconds
            );
        }
        commandline::Commands::SetBacklightTime {
            backlight_time_in_minutes,
        } => {
            ensure_authorization(&mut d)?;
            d.set_backlight_time(*backlight_time_in_minutes)
                .with_context(|| "Cannot set backlinght time")?;
            println!(
                "Backlight time successfully changed to: {}",
                backlight_time_in_minutes
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
            println!("Historical data successfully resetted",);
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
