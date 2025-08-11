//! This module defines the core data structures and traits for the SDM72 Modbus protocol.
//!
//! It includes definitions for all supported Modbus registers, data types for meter
//! settings and measurements, and helper traits for encoding and decoding Modbus data.
//!
//! The documentation for this module is based on the "Eastron SDM72D-M-v2 Modbus Protocol"
//! document.

use crate::Error;

/// 16-bit value stored in Modbus register.
pub type Word = u16;

/// A trait for defining Modbus parameters.
///
/// This trait provides a common interface for defining the properties of a Modbus
/// register, such as its address, the number of words it occupies, and the data
/// type it represents.
pub trait ModbusParam: Sized {
    /// The Modbus holding register address.
    const ADDRESS: u16;
    /// The quantity of Modbus words (16-bit). The length in bytes is `QUANTITY * 2`.
    const QUANTITY: u16;
    /// The data type that the Modbus words represent (e.g., `f32`, `u16`).
    type ProtocolType;
}

/// A macro to convert a slice of `u16` words into a protocol value (e.g., `f32`).
macro_rules! words_to_protocol_value {
    ($words:expr) => {{
        let bytes = $words
            .iter()
            .copied()
            .flat_map(u16::to_be_bytes)
            .collect::<Vec<u8>>();
        let array = bytes.try_into().or(Err(Error::WordsCountError))?;
        Ok(<Self as ModbusParam>::ProtocolType::from_be_bytes(array))
    }};
}

/// A macro to convert a protocol value (e.g., `f32`) into a `Vec<u16>` of Modbus words.
macro_rules! protocol_value_to_words {
    ($val:expr) => {
        $val.to_be_bytes()
            .chunks(2)
            .map(|chunk| {
                let array = chunk.try_into().expect("unexpected encoding error");
                u16::from_be_bytes(array)
            })
            .collect()
    };
}

/// The system (wiring) type.
///
/// Note: To set the value you need ['KPPA'](enum@KPPA).
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SystemType {
    /// 1 phase with 2 wire
    Type1P2W,

    #[default]
    /// 3 phase with 4 wire
    Type3P4W,
}
impl ModbusParam for SystemType {
    type ProtocolType = f32;
    const ADDRESS: u16 = 0x000A;
    const QUANTITY: u16 = 2;
}
impl SystemType {
    pub fn decode_from_holding_registers(words: &[Word]) -> Result<Self, Error> {
        let val = words_to_protocol_value!(words)?;
        match val {
            1.0 => Ok(SystemType::Type1P2W),
            3.0 => Ok(SystemType::Type3P4W),
            _ => Err(Error::InvalidValue),
        }
    }

    pub fn encode_for_write_registers(&self) -> Vec<Word> {
        let val = match self {
            SystemType::Type1P2W => 1,
            SystemType::Type3P4W => 3,
        } as <Self as ModbusParam>::ProtocolType;
        protocol_value_to_words!(val)
    }
}
impl std::fmt::Display for SystemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemType::Type1P2W => write!(f, "1 phase 2 wire"),
            SystemType::Type3P4W => write!(f, "3 phase 4 wire"),
        }
    }
}

/// Pulse width for the pulse output in milliseconds.
///
/// Note: If pulse constant is 1000 imp/kWh, then the pulse width is fixed to 35ms and cannot be adjusted!
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PulseWidth(u16);
impl ModbusParam for PulseWidth {
    type ProtocolType = f32;
    const ADDRESS: u16 = 0x000C;
    const QUANTITY: u16 = 2;
}
impl std::ops::Deref for PulseWidth {
    type Target = u16;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Default for PulseWidth {
    fn default() -> Self {
        Self(100)
    }
}
impl PulseWidth {
    pub fn decode_from_holding_registers(words: &[Word]) -> Result<Self, Error> {
        let val = words_to_protocol_value!(words)?;
        Ok(Self(val as u16))
    }

    pub fn encode_for_write_registers(&self) -> Vec<Word> {
        let val = self.0 as <Self as ModbusParam>::ProtocolType;
        protocol_value_to_words!(val)
    }
}
impl TryFrom<u16> for PulseWidth {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}
impl std::fmt::Display for PulseWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// KPPA (Key Parameter Programming Authorization) write the correct password to get KPPA.
/// This will be required to change the settings.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum KPPA {
    NotAuthorized,
    Authorized,
}
impl ModbusParam for KPPA {
    type ProtocolType = f32;
    const ADDRESS: u16 = 0x000E;
    const QUANTITY: u16 = 2;
}
impl KPPA {
    pub fn decode_from_holding_registers(words: &[Word]) -> Result<Self, Error> {
        let val = words_to_protocol_value!(words)?;
        match val {
            0.0 => Ok(Self::NotAuthorized),
            1.0 => Ok(Self::Authorized),
            _ => Err(Error::InvalidValue),
        }
    }

    pub fn encode_for_write_registers(password: Password) -> Vec<Word> {
        password.encode_for_write_registers()
    }
}
impl std::fmt::Display for KPPA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KPPA::NotAuthorized => write!(f, "not authorized"),
            KPPA::Authorized => write!(f, "authorized"),
        }
    }
}

/// Parity and stop bits of the Modbus RTU protocol for the RS485 serial port.
///
/// Note: To set the value you need ['KPPA'](enum@KPPA).
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParityAndStopBit {
    #[default]
    /// no parity, one stop bit
    NoParityOneStopBit,

    /// even parity, one stop bit
    EvenParityOneStopBit,

    /// odd parity, one stop bit
    OddParityOneStopBit,

    /// no parity, two stop bits
    NoParityTwoStopBits,
}
impl ModbusParam for ParityAndStopBit {
    type ProtocolType = f32;
    const ADDRESS: u16 = 0x0012;
    const QUANTITY: u16 = 2;
}
impl ParityAndStopBit {
    pub fn decode_from_holding_registers(words: &[Word]) -> Result<Self, Error> {
        let val = words_to_protocol_value!(words)?;
        match val {
            0.0 => Ok(Self::NoParityOneStopBit),
            1.0 => Ok(Self::EvenParityOneStopBit),
            2.0 => Ok(Self::OddParityOneStopBit),
            3.0 => Ok(Self::NoParityTwoStopBits),
            _ => Err(Error::InvalidValue),
        }
    }

    pub fn encode_for_write_registers(&self) -> Vec<Word> {
        let val = match self {
            Self::NoParityOneStopBit => 0,
            Self::EvenParityOneStopBit => 1,
            Self::OddParityOneStopBit => 2,
            Self::NoParityTwoStopBits => 3,
        } as <Self as ModbusParam>::ProtocolType;
        protocol_value_to_words!(val)
    }
}
impl std::fmt::Display for ParityAndStopBit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoParityOneStopBit => write!(f, "no parity one stop bit"),
            Self::EvenParityOneStopBit => write!(f, "even parity one stop bit"),
            Self::OddParityOneStopBit => write!(f, "odd parity one stop bit"),
            Self::NoParityTwoStopBits => write!(f, "no parity two stop bit"),
        }
    }
}

/// Address of the Modbus RTU protocol for the RS485 serial port.
/// The address must be in the range from 1 to 247.
///
/// Note: To set the value you need ['KPPA'](enum@KPPA).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Address(u8);
impl ModbusParam for Address {
    type ProtocolType = f32;
    const ADDRESS: u16 = 0x0014;
    const QUANTITY: u16 = 2;
}
impl std::ops::Deref for Address {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Default for Address {
    fn default() -> Self {
        Self(0x01)
    }
}
impl Address {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 247;

    pub fn decode_from_holding_registers(words: &[Word]) -> Result<Self, Error> {
        let val = words_to_protocol_value!(words)?;
        Ok(Self(val as u8))
    }

    pub fn encode_for_write_registers(&self) -> Vec<Word> {
        let val = self.0 as <Self as ModbusParam>::ProtocolType;
        protocol_value_to_words!(val)
    }
}
impl TryFrom<u8> for Address {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if (Self::MIN..=Self::MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(Error::AddressOutOfRange(value))
        }
    }
}
impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#04x}", self.0)
    }
}

/// Pulse constant for the pulse output in impulses per kilo watt hour.
///
/// Note: If pulse constant is 1000 imp/kWh, then the pulse width is fixed to 35ms and cannot be adjusted!
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PulseConstant {
    #[default]
    /// 1000 imp/kWh
    PC1000,

    /// 100 imp/kWh
    PC100,

    /// 10 imp/kWh
    PC10,

    /// 1 imp/kWh
    PC1,
}
impl ModbusParam for PulseConstant {
    type ProtocolType = f32;
    const ADDRESS: u16 = 0x0016;
    const QUANTITY: u16 = 2;
}
impl PulseConstant {
    pub fn decode_from_holding_registers(words: &[Word]) -> Result<Self, Error> {
        let val = words_to_protocol_value!(words)?;
        match val {
            0.0 => Ok(Self::PC1000),
            1.0 => Ok(Self::PC100),
            2.0 => Ok(Self::PC10),
            3.0 => Ok(Self::PC1),
            _ => Err(Error::InvalidValue),
        }
    }

    pub fn encode_for_write_registers(&self) -> Vec<Word> {
        let val = match self {
            Self::PC1000 => 0,
            Self::PC100 => 1,
            Self::PC10 => 2,
            Self::PC1 => 3,
        } as <Self as ModbusParam>::ProtocolType;
        protocol_value_to_words!(val)
    }
}
impl std::fmt::Display for PulseConstant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PC1000 => write!(f, "1000 imp/kWh"),
            Self::PC100 => write!(f, "100 imp/kWh"),
            Self::PC10 => write!(f, "10 imp/kWh"),
            Self::PC1 => write!(f, "1 imp/kWh"),
        }
    }
}

/// Password must be in the range from 0 to 9999.
///
/// Note: To set the value you need ['KPPA'](enum@KPPA).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Password(u16);
impl ModbusParam for Password {
    type ProtocolType = f32;
    const ADDRESS: u16 = 0x0018;
    const QUANTITY: u16 = 2;
}
impl std::ops::Deref for Password {
    type Target = u16;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Default for Password {
    fn default() -> Self {
        Self(1000)
    }
}
impl Password {
    pub const MIN: u16 = 0;
    pub const MAX: u16 = 9999;

    pub fn decode_from_holding_registers(words: &[Word]) -> Result<Self, Error> {
        let val = words_to_protocol_value!(words)?;
        Ok(Self(val as u16))
    }

    pub fn encode_for_write_registers(&self) -> Vec<Word> {
        let val = self.0 as <Self as ModbusParam>::ProtocolType;
        protocol_value_to_words!(val)
    }
}
impl TryFrom<u16> for Password {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if (Self::MIN..=Self::MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(Error::PasswordOutOfRange(value))
        }
    }
}
impl std::fmt::Display for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}", self.0)
    }
}

/// Baud rate of the Modbus RTU protocol for the RS485 serial port.
/// Supported rates are: 1200, 2400, 4800, 9600, 19200
///
/// Note: To set the value you need ['KPPA'](enum@KPPA).
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BaudRate {
    B1200,
    B2400,
    B4800,
    #[default]
    B9600,
    B19200,
}
impl ModbusParam for BaudRate {
    type ProtocolType = f32;
    const ADDRESS: u16 = 0x001C;
    const QUANTITY: u16 = 2;
}
impl BaudRate {
    pub fn decode_from_holding_registers(words: &[Word]) -> Result<Self, Error> {
        let val = words_to_protocol_value!(words)?;
        match val {
            5.0 => Ok(Self::B1200),
            0.0 => Ok(Self::B2400),
            1.0 => Ok(Self::B4800),
            2.0 => Ok(Self::B9600),
            3.0 => Ok(Self::B19200),
            _ => Err(Error::InvalidValue),
        }
    }

    pub fn encode_for_write_registers(&self) -> Vec<Word> {
        let val = match self {
            Self::B1200 => 5,
            Self::B2400 => 0,
            Self::B4800 => 1,
            Self::B9600 => 2,
            Self::B19200 => 3,
        } as <Self as ModbusParam>::ProtocolType;
        protocol_value_to_words!(val)
    }

    pub fn decode(words: &[Word]) -> Result<u16, Error> {
        let val = words_to_protocol_value!(words)?;
        Ok(val as u16)
    }
}
impl TryFrom<u16> for BaudRate {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1200 => Ok(BaudRate::B1200),
            2400 => Ok(BaudRate::B2400),
            4800 => Ok(BaudRate::B4800),
            9600 => Ok(BaudRate::B9600),
            19200 => Ok(BaudRate::B19200),
            _ => Err(Error::InvalidBaudRate),
        }
    }
}
impl From<&BaudRate> for u16 {
    fn from(baud_rate: &BaudRate) -> u16 {
        match baud_rate {
            BaudRate::B1200 => 1200,
            BaudRate::B2400 => 2400,
            BaudRate::B4800 => 4800,
            BaudRate::B9600 => 9600,
            BaudRate::B19200 => 19200,
        }
    }
}
impl std::fmt::Display for BaudRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", u16::from(self))
    }
}

/// Automatic display scroll time in seconds.
/// The time must be in the range from 0 to 60.
///
/// Note: To set the value you need ['KPPA'](enum@KPPA).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AutoScrollTime(u8);
impl ModbusParam for AutoScrollTime {
    type ProtocolType = f32;
    const ADDRESS: u16 = 0x003A;
    const QUANTITY: u16 = 2;
}
impl Default for AutoScrollTime {
    fn default() -> Self {
        Self(5)
    }
}
impl std::ops::Deref for AutoScrollTime {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AutoScrollTime {
    pub const MIN: u8 = 0;
    pub const MAX: u8 = 60;

    pub fn decode_from_holding_registers(words: &[Word]) -> Result<Self, Error> {
        let val = words_to_protocol_value!(words)?;
        Ok(Self(val as u8))
    }

    pub fn encode_for_write_registers(&self) -> Vec<Word> {
        let val = self.0 as <Self as ModbusParam>::ProtocolType;
        protocol_value_to_words!(val)
    }

    pub fn decode(words: &[Word]) -> Result<u8, Error> {
        let val = words_to_protocol_value!(words)?;
        Ok(val as u8)
    }
}
impl TryFrom<u8> for AutoScrollTime {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if (Self::MIN..=Self::MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(Error::AutoScrollTimeOutOfRange(value))
        }
    }
}
impl std::fmt::Display for AutoScrollTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} sec", self.0)
    }
}

/// Back light time of the display in minutes.
/// The time must be in the range from 1 to 120.
///
/// Note: To set the value you need ['KPPA'](enum@KPPA).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BacklightTime {
    AlwaysOn,
    AlwaysOff,
    Delayed(u8),
}
impl ModbusParam for BacklightTime {
    type ProtocolType = f32;
    const ADDRESS: u16 = 0x003C;
    const QUANTITY: u16 = 2;
}
impl Default for BacklightTime {
    fn default() -> Self {
        Self::Delayed(60)
    }
}
impl BacklightTime {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 120;

    pub fn decode_from_holding_registers(words: &[Word]) -> Result<Self, Error> {
        let val = words_to_protocol_value!(words)?;
        match val {
            0.0 => Ok(Self::AlwaysOn),
            121.0 => Ok(Self::AlwaysOff),
            _ => Ok(Self::Delayed(val as u8)),
        }
    }

    pub fn encode_for_write_registers(&self) -> Vec<Word> {
        let val = match self {
            Self::AlwaysOn => 0,
            Self::AlwaysOff => 121,
            Self::Delayed(val) => *val,
        } as <Self as ModbusParam>::ProtocolType;
        protocol_value_to_words!(val)
    }
}
impl TryFrom<u8> for BacklightTime {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if (Self::MIN..=Self::MAX).contains(&value) {
            Ok(Self::Delayed(value))
        } else if value == 0 {
            Ok(Self::AlwaysOn)
        } else if value == 121 {
            Ok(Self::AlwaysOff)
        } else {
            Err(Error::BacklitTimeOutOfRange(value))
        }
    }
}
impl std::fmt::Display for BacklightTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlwaysOn => write!(f, "always on"),
            Self::AlwaysOff => write!(f, "always off"),
            Self::Delayed(val) => write!(f, "{val} min"),
        }
    }
}

/// Pulse energy type for the pulse output. This is the value that the pulse output returns.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PulseEnergyType {
    ImportActiveEnergy,

    #[default]
    TotalActiveEnergy,

    ExportActiveEnergy,
}
impl ModbusParam for PulseEnergyType {
    type ProtocolType = f32;
    const ADDRESS: u16 = 0x0056;
    const QUANTITY: u16 = 2;
}
impl PulseEnergyType {
    pub fn decode_from_holding_registers(words: &[Word]) -> Result<Self, Error> {
        let val = words_to_protocol_value!(words)?;
        match val {
            1.0 => Ok(Self::ImportActiveEnergy),
            2.0 => Ok(Self::TotalActiveEnergy),
            4.0 => Ok(Self::ExportActiveEnergy),
            _ => Err(Error::InvalidValue),
        }
    }

    pub fn encode_for_write_registers(&self) -> Vec<Word> {
        let val = match self {
            Self::ImportActiveEnergy => 1,
            Self::TotalActiveEnergy => 2,
            Self::ExportActiveEnergy => 4,
        } as <Self as ModbusParam>::ProtocolType;
        protocol_value_to_words!(val)
    }
}
impl std::fmt::Display for PulseEnergyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ImportActiveEnergy => write!(f, "import active energy"),
            Self::TotalActiveEnergy => write!(f, "total active energy"),
            Self::ExportActiveEnergy => write!(f, "export active energy"),
        }
    }
}

/// Reset the historical saved data.
///
/// Note: To reset the data you need ['KPPA'](enum@KPPA).
pub struct ResetHistoricalData;
impl ModbusParam for ResetHistoricalData {
    type ProtocolType = u16;
    const ADDRESS: u16 = 0xF010;
    const QUANTITY: u16 = 1;
}
impl ResetHistoricalData {
    pub fn encode_for_write_registers() -> Vec<Word> {
        let val = 0x0003 as <Self as ModbusParam>::ProtocolType;
        protocol_value_to_words!(val)
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SerialNumber(u32);
impl ModbusParam for SerialNumber {
    type ProtocolType = u32;
    const ADDRESS: u16 = 0xFC00;
    const QUANTITY: u16 = 2;
}
impl SerialNumber {
    pub fn decode_from_holding_registers(words: &[Word]) -> Result<Self, Error> {
        let val = words_to_protocol_value!(words)?;
        Ok(Self(val))
    }
}
impl std::ops::Deref for SerialNumber {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::fmt::Display for SerialNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Meter code SDM72D-M-2 = 0089
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MeterCode(u16);
impl ModbusParam for MeterCode {
    type ProtocolType = u16;
    const ADDRESS: u16 = 0xFC02;
    const QUANTITY: u16 = 1;
}
impl MeterCode {
    pub fn decode_from_holding_registers(words: &[Word]) -> Result<Self, Error> {
        let val = words_to_protocol_value!(words)?;
        Ok(Self(val))
    }
}
impl std::ops::Deref for MeterCode {
    type Target = u16;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::fmt::Display for MeterCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0>4x}", self.0)
    }
}

/// The software version showed on display
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SoftwareVersion(u16);
impl ModbusParam for SoftwareVersion {
    type ProtocolType = u16;
    const ADDRESS: u16 = 0xFC84;
    const QUANTITY: u16 = 1;
}
impl SoftwareVersion {
    pub fn decode_from_holding_registers(words: &[Word]) -> Result<Self, Error> {
        let val = words_to_protocol_value!(words)?;
        Ok(Self(val))
    }
}
impl std::ops::Deref for SoftwareVersion {
    type Target = u16;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::fmt::Display for SoftwareVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0>2x}.{:0>2x}", (self.0 >> 8) as u8, self.0 as u8)
    }
}

/// A trait for Modbus input registers.
///
/// Input registers are used to indicate the present values of the measured and
/// calculated electrical quantities. Modbus Protocol function code 04 is used to
/// access all parameters.
///
/// Note: Each request for data must be restricted to 30 parameters or less.
/// Exceeding the 30 parameter limit will cause a Modbus Protocol exception code
/// to be returned.
pub trait ModbusInputRegister: ModbusParam {
    /// Decodes a value from a slice of Modbus input register words.
    fn decode_from_input_register(words: &[Word]) -> Result<Self, Error>;
}

fn f32round(val: f32) -> f32 {
    ((val as f64 * 100.).round() / 100.) as f32
}

#[cfg(feature = "serde")]
fn f32ser2<S>(fv: &f32, se: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    se.serialize_f32(f32round(*fv))
}

/// A macro to define a newtype struct for a Modbus input register.
///
/// This macro generates a newtype struct that wraps a protocol type (e.g., `f32`)
/// and implements the `ModbusParam` and `ModbusInputRegister` traits for it.
/// It also implements `Display` and `Deref`.
macro_rules! modbus_input_register {
    ($vis:vis $ty:ident, $address:expr, $quantity:expr, $protocol_type:ty) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        $vis struct $ty(
            #[cfg_attr(feature = "serde", serde(serialize_with = "f32ser2"))]
            $protocol_type,
        );
        impl std::fmt::Display for $ty {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(fmt, "{}", f32round(self.0))
            }
        }

        impl ModbusParam for $ty {
            type ProtocolType = $protocol_type;
            const ADDRESS: u16 = $address;
            const QUANTITY: u16 = $quantity;
        }

        impl std::ops::Deref for $ty {
            type Target = $protocol_type;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl $ty {
            pub fn decode_from_input_register(words: &[Word]) -> Result<Self, Error> {
                let val = words_to_protocol_value!(words)?;
                Ok(Self(val as $protocol_type))
            }
        }
    };
}
/// A macro to get the range of a register within a larger response slice.
///
/// This is used when reading a batch of registers at once.
#[macro_export]
macro_rules! get_subset_register_range {
    ($offset:expr, $register_name:ty) => {{
        (<$register_name>::ADDRESS - $offset) as usize
            ..(<$register_name>::ADDRESS - $offset + <$register_name>::QUANTITY) as usize
    }};
}

/// A macro to decode a holding register value from a response slice.
///
/// This is used when reading a batch of registers at once.
#[macro_export]
macro_rules! decode_subset_item_from_holding_register {
    ($offset:expr, $register_name:ty, $rsp:expr) => {{
        <$register_name>::decode_from_holding_registers(
            &$rsp[$crate::get_subset_register_range!($offset, $register_name)],
        )
    }};
}

/// A macro to decode an input register value from a response slice.
///
/// This is used when reading a batch of registers at once.
#[macro_export]
macro_rules! decode_subset_item_from_input_register {
    ($offset:expr, $register_name:ty, $rsp:expr) => {{
        <$register_name>::decode_from_input_register(
            &$rsp[$crate::get_subset_register_range!($offset, $register_name)],
        )
    }};
}

// 1 Batch
modbus_input_register!(pub L1Voltage, 0x0000, 2, f32);
modbus_input_register!(pub L2Voltage, 0x0002, 2, f32);
modbus_input_register!(pub L3Voltage, 0x0004, 2, f32);
modbus_input_register!(pub L1Current, 0x0006, 2, f32);
modbus_input_register!(pub L2Current, 0x0008, 2, f32);
modbus_input_register!(pub L3Current, 0x000A, 2, f32);
modbus_input_register!(pub L1PowerActive, 0x000C, 2, f32);
modbus_input_register!(pub L2PowerActive, 0x000E, 2, f32);
modbus_input_register!(pub L3PowerActive, 0x0010, 2, f32);
modbus_input_register!(pub L1PowerApparent, 0x0012, 2, f32);
modbus_input_register!(pub L2PowerApparent, 0x0014, 2, f32);
modbus_input_register!(pub L3PowerApparent, 0x0016, 2, f32);
modbus_input_register!(pub L1PowerReactive, 0x0018, 2, f32);
modbus_input_register!(pub L2PowerReactive, 0x001A, 2, f32);
modbus_input_register!(pub L3PowerReactive, 0x001C, 2, f32);
modbus_input_register!(pub L1PowerFactor, 0x0001E, 2, f32);
modbus_input_register!(pub L2PowerFactor, 0x0020, 2, f32);
modbus_input_register!(pub L3PowerFactor, 0x0022, 2, f32);
modbus_input_register!(pub LtoNAverageVoltage, 0x002A, 2, f32);
modbus_input_register!(pub LtoNAverageCurrent, 0x002E, 2, f32);
modbus_input_register!(pub TotalLineCurrent, 0x0030, 2, f32);
modbus_input_register!(pub TotalPower, 0x0034, 2, f32);
modbus_input_register!(pub TotalPowerApparent, 0x0038, 2, f32);
modbus_input_register!(pub TotalPowerReactive, 0x003C, 2, f32);
modbus_input_register!(pub TotalPowerFactor, 0x003E, 2, f32);
modbus_input_register!(pub Frequency, 0x0046, 2, f32);
modbus_input_register!(pub ImportEnergyActive, 0x0048, 2, f32);
modbus_input_register!(pub ExportEnergyActive, 0x004A, 2, f32);
// 2 Batch
modbus_input_register!(pub L1ToL2Voltage, 0x00C8, 2, f32);
modbus_input_register!(pub L2ToL3Voltage, 0x00CA, 2, f32);
modbus_input_register!(pub L3ToL1Voltage, 0x00CC, 2, f32);
modbus_input_register!(pub LtoLAverageVoltage, 0x00CE, 2, f32);
modbus_input_register!(pub NeutralCurrent, 0x00E0, 2, f32);
// 3 Batch
modbus_input_register!(pub TotalEnergyActive, 0x0156, 2, f32);
modbus_input_register!(pub TotalEnergyReactive, 0x0158, 2, f32);
modbus_input_register!(pub ResettableTotalEnergyActive, 0x0180, 2, f32);
modbus_input_register!(pub ResettableTotalEnergyReactive, 0x0182, 2, f32);
modbus_input_register!(pub ResettableImportEnergyActive, 0x0184, 2, f32);
modbus_input_register!(pub ResettableExportEnergyActive, 0x0186, 2, f32);
modbus_input_register!(pub NetKwh, 0x018C, 2, f32);
modbus_input_register!(pub ImportTotalPowerActive, 0x0500, 2, f32);
modbus_input_register!(pub ExportTotalPowerActive, 0x0502, 2, f32);
