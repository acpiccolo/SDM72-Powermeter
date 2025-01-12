use crate::MqttConfig;
use clap::{Parser, Subcommand, ValueEnum};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use sdm72_lib::protocol as proto;
use std::{fmt, ops::Deref, time::Duration};

pub fn parse_address(s: &str) -> Result<proto::Address, String> {
    proto::Address::try_from(clap_num::maybe_hex::<u8>(s)?).map_err(|e| format!("{e}"))
}

pub fn parse_password(s: &str) -> Result<proto::Password, String> {
    proto::Password::try_from(s.parse::<u16>().map_err(|e| format!("{e}"))?)
        .map_err(|e| format!("{e}"))
}

pub fn parse_baud_rate(s: &str) -> Result<proto::BaudRate, String> {
    proto::BaudRate::try_from(s.parse::<u16>().map_err(|e| format!("{e}"))?)
        .map_err(|e| format!("{e}"))
}

pub fn parse_auto_scroll_time(s: &str) -> Result<proto::AutoScrollTime, String> {
    proto::AutoScrollTime::try_from(s.parse::<u8>().map_err(|e| format!("{e}"))?)
        .map_err(|e| format!("{e}"))
}

pub fn parse_backlight_time(s: &str) -> Result<proto::BacklightTime, String> {
    proto::BacklightTime::try_from(s.parse::<u8>().map_err(|e| format!("{e}"))?)
        .map_err(|e| format!("{e}"))
}

fn default_device_name() -> String {
    if cfg!(target_os = "windows") {
        String::from("COM1")
    } else {
        String::from("/dev/ttyUSB0")
    }
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Connection {
    /// Use Modbus/TCP connection
    Tcp {
        // TCP address (e.g. 192.168.0.222:502)
        address: String,

        #[command(subcommand)]
        command: Commands,
    },
    /// Use Modbus/RTU connection
    Rtu {
        /// Device
        #[arg(short, long, default_value_t = default_device_name())]
        device: String,

        /// Baud rate any of 1200, 2400, 4800, 9600, 19200
        #[arg(long, default_value_t = proto::BaudRate::default(), value_parser = parse_baud_rate)]
        baud_rate: proto::BaudRate,

        /// RS485 address from 1 to 247
        #[arg(long, default_value_t = proto::Address::default(), value_parser = parse_address)]
        address: proto::Address,

        /// Parity and stop bits of the Modbus RTU protocol for the RS485 serial port.
        #[arg(long, default_value_t = ParityAndStopBit(proto::ParityAndStopBit::default()))]
        parity_and_stop_bit: ParityAndStopBit,

        #[command(subcommand)]
        command: Commands,
    },
}

#[derive(Subcommand, Debug, Clone, PartialEq, Default)]
pub enum DaemonMode {
    #[default]
    /// Print values to stdout [default]
    Stdout,
    /// Send values to a MQTT Broker
    Mqtt {
        /// The configuration file for the MQTT broker
        #[arg(long, default_value_t = MqttConfig::DEFAULT_CONFIG_FILE.to_string())]
        config_file: String,
        // /// URL to the MQTT broker like: mqtt://localhost:1883
        // url: String,

        // /// The user name for authentication with the broker
        // #[arg(short, long)]
        // username: Option<String>,

        // /// The password for authentication with the broker
        // #[arg(short, long)]
        // password: Option<String>,

        // /// MQTT topic
        // #[arg(long, default_value_t = MqttConfig::default_topic())]
        // topic: String,

        // /// Quality of service to use
        // #[arg(long, default_value_t = MqttConfig::default_qos())]
        // qos: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WiringType(proto::SystemType);
impl clap::ValueEnum for WiringType {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            WiringType(proto::SystemType::Type1P2W),
            WiringType(proto::SystemType::Type3P4W),
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self.0 {
            proto::SystemType::Type1P2W => {
                Some(clap::builder::PossibleValue::new("1p2w").help("1 phase with 2 wire"))
            }
            proto::SystemType::Type3P4W => {
                Some(clap::builder::PossibleValue::new("3p4w").help("3 phase with 4 wire"))
            }
        }
    }
}
impl Deref for WiringType {
    type Target = proto::SystemType;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParityAndStopBit(proto::ParityAndStopBit);
impl clap::ValueEnum for ParityAndStopBit {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            ParityAndStopBit(proto::ParityAndStopBit::NoParityOneStopBit),
            ParityAndStopBit(proto::ParityAndStopBit::EvenParityOneStopBit),
            ParityAndStopBit(proto::ParityAndStopBit::OddParityOneStopBit),
            ParityAndStopBit(proto::ParityAndStopBit::NoParityTwoStopBits),
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self.0 {
            proto::ParityAndStopBit::NoParityOneStopBit => {
                Some(clap::builder::PossibleValue::new("np1b").help("no parity, one stop bit"))
            }
            proto::ParityAndStopBit::EvenParityOneStopBit => {
                Some(clap::builder::PossibleValue::new("ep1b").help("even parity, one stop bit"))
            }
            proto::ParityAndStopBit::OddParityOneStopBit => {
                Some(clap::builder::PossibleValue::new("op1b").help("odd parity, one stop bit"))
            }
            proto::ParityAndStopBit::NoParityTwoStopBits => {
                Some(clap::builder::PossibleValue::new("np2b").help("no parity, two stop bits"))
            }
        }
    }
}
impl Deref for ParityAndStopBit {
    type Target = proto::ParityAndStopBit;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl fmt::Display for ParityAndStopBit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.to_possible_value()
                .map(|val| val.get_name().to_string())
                .unwrap_or_default()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PulseConstant(proto::PulseConstant);
impl clap::ValueEnum for PulseConstant {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            PulseConstant(proto::PulseConstant::PC1000),
            PulseConstant(proto::PulseConstant::PC100),
            PulseConstant(proto::PulseConstant::PC10),
            PulseConstant(proto::PulseConstant::PC1),
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self.0 {
            proto::PulseConstant::PC1000 => {
                Some(clap::builder::PossibleValue::new("1000").help("1000 imp/kWh"))
            }
            proto::PulseConstant::PC100 => {
                Some(clap::builder::PossibleValue::new("100").help("100 imp/kWh"))
            }
            proto::PulseConstant::PC10 => {
                Some(clap::builder::PossibleValue::new("10").help("10 imp/kWh"))
            }
            proto::PulseConstant::PC1 => {
                Some(clap::builder::PossibleValue::new("1").help("1 imp/kWh"))
            }
        }
    }
}
impl Deref for PulseConstant {
    type Target = proto::PulseConstant;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PulseEnergyType(proto::PulseEnergyType);
impl clap::ValueEnum for PulseEnergyType {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            PulseEnergyType(proto::PulseEnergyType::ImportActiveEnergy),
            PulseEnergyType(proto::PulseEnergyType::TotalActiveEnergy),
            PulseEnergyType(proto::PulseEnergyType::ExportActiveEnergy),
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self.0 {
            proto::PulseEnergyType::ImportActiveEnergy => {
                Some(clap::builder::PossibleValue::new("import"))
            }
            proto::PulseEnergyType::TotalActiveEnergy => {
                Some(clap::builder::PossibleValue::new("total"))
            }
            proto::PulseEnergyType::ExportActiveEnergy => {
                Some(clap::builder::PossibleValue::new("export"))
            }
        }
    }
}
impl Deref for PulseEnergyType {
    type Target = proto::PulseEnergyType;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Commands {
    /// Daemon mode to read all values of the measured and calculated electrical quantities
    Daemon {
        /// Interval for repeated polling of the values
        #[arg(value_parser = humantime::parse_duration, short, long, default_value = "2sec")]
        poll_iterval: Duration,

        #[command(subcommand)]
        mode: DaemonMode,
    },

    /// Read all values of the measured and calculated electrical quantities
    ReadAll,

    /// Read all settings
    ReadAllSettings,

    /// Password to obtain authorization to change the settings
    Password {
        #[arg(value_parser = parse_password)]
        password: proto::Password,
    },

    /// Set the parity and stop bit
    SetParityAndStopBit {
        parity_and_stop_bit: ParityAndStopBit,
    },

    /// Set the baud rate
    SetBaudRate {
        /// The new baud rate any value of 1200, 2400, 4800, 9600, 19200
        #[arg(value_parser = parse_baud_rate)]
        baud_rate: proto::BaudRate,
    },

    /// Set the RS485 address
    SetAddress {
        /// The RS485 address can be from 1 to 247
        #[arg(value_parser = parse_address)]
        address: proto::Address,
    },

    /// Set the wiring type
    SetWiringType { wiring_type: WiringType },

    /// Pulse constant for the pulse output
    SetPulseConstant {
        /// The pulse is specified in impulses per kilo watt hour
        pulse_constant_in_kwh: PulseConstant,
    },

    /// Set password to change the settings
    SetPassword {
        /// The password must be in the range from 0 to 9999
        #[arg(value_parser = parse_password)]
        password: proto::Password,
    },

    /// Automatic display scroll time
    SetAutoScrollTime {
        /// The time is specified in seconds and must be in the range from 0 to 60
        #[arg(value_parser = parse_auto_scroll_time)]
        auto_scroll_time_in_seconds: proto::AutoScrollTime,
    },

    /// Back light time of the display
    SetBacklightTime {
        /// The time is specified in minutes and must be in the range from 0 to 121, 0 means always on and 121 means always off
        #[arg(value_parser = parse_backlight_time)]
        backlight_time_in_minutes: proto::BacklightTime,
    },

    /// Pulse energy type for the pulse output
    SetPulseEnergyType {
        /// This is the value that the pulse output returns
        pulse_energy_type: PulseEnergyType,
    },

    /// Reset the historical saved data
    ResetHistoricalData,
}

const fn about_text() -> &'static str {
    "SDM72 powermeter for the command line tool"
}

#[derive(Parser, Debug)]
#[command(version, about=about_text(), long_about = None)]
pub struct Args {
    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,

    /// Output to stdout not in JSON format
    #[arg(long, default_value = "false")]
    pub no_json: bool,

    // Connection type
    #[command(subcommand)]
    pub connection: Connection,

    /// Modbus Input/Output operations timeout
    #[arg(value_parser = humantime::parse_duration, long, default_value = "200ms")]
    pub timeout: Duration,

    // According to Modbus specification:
    // Wait at least 3.5 char between frames
    // However, some USB - RS485 dongles requires at least 10ms to switch between TX and RX, so use a save delay between frames
    /// Delay between multiple modbus commands
    #[arg(value_parser = humantime::parse_duration, long, default_value = "50ms")]
    pub delay: Duration,
}
