//! This module defines the custom error types used throughout the `sdm72` library.
//!
//! The `Error` enum represents all possible errors that can occur during the
//! processing of Modbus data, excluding communication errors, which are handled
//! by the `tokio_common::Error` enum.

use crate::protocol::{self};

/// Represents errors that can occur within the SDM72 protocol logic.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The provided Modbus address is outside the valid range.
    #[error(
        "The address value {0} is outside the permissible range of {min} to {max}",
        min = protocol::Address::MIN,
        max = protocol::Address::MAX
    )]
    AddressOutOfRange(u8),

    /// The provided password is outside the valid range.
    #[error(
        "The password value {0} is outside the permissible range of {min} to {max}",
        min = protocol::Password::MIN,
        max = protocol::Password::MAX
    )]
    PasswordOutOfRange(u16),

    /// The provided auto scroll time is outside the valid range.
    #[error(
        "The auto scroll time value {0} is outside the permissible range of {min} to {max}",
        min = protocol::AutoScrollTime::MIN,
        max = protocol::AutoScrollTime::MAX
    )]
    AutoScrollTimeOutOfRange(u8),

    /// The provided backlight time is outside the valid range.
    #[error(
        "The backlit time value {0} is outside the permissible range of {min} to {max}",
        min = protocol::BacklightTime::MIN,
        max = protocol::BacklightTime::MAX
    )]
    BacklitTimeOutOfRange(u8),

    /// The provided baud rate is not supported by the device.
    #[error("Baud rate must by any value of 1200, 2400, 4800, 9600, 19200.")]
    InvalidBaudRate,

    /// A generic error for a value that is outside its permissible range.
    #[error("Value is outside the permissible range")]
    OutOfRange,

    /// An unexpected or invalid value was received from the device.
    #[error("Unexpected value")]
    InvalidValue,

    /// The number of words received from the device is incorrect for the requested operation.
    #[error("Words count error")]
    WordsCountError,
}
