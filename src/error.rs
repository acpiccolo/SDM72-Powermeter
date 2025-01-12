use crate::protocol::{self};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "The address value {0} is outside the permissible range of {min} to {max}",
        min = protocol::Address::MIN,
        max = protocol::Address::MAX
    )]
    AddressOutOfRange(u8),
    #[error(
        "The password value {0} is outside the permissible range of {min} to {max}",
        min = protocol::Password::MIN,
        max = protocol::Password::MAX
    )]
    PasswordOutOfRange(u16),
    #[error(
        "The auto scroll time value {0} is outside the permissible range of {min} to {max}",
        min = protocol::AutoScrollTime::MIN,
        max = protocol::AutoScrollTime::MAX
    )]
    AutoScrollTimeOutOfRange(u8),
    #[error(
        "The backlit time value {0} is outside the permissible range of {min} to {max}",
        min = protocol::BacklightTime::MIN,
        max = protocol::BacklightTime::MAX
    )]
    BacklitTimeOutOfRange(u8),
    #[error("Baud rate must by any value of 1200, 2400, 4800, 9600, 19200.")]
    InvalidBaudRate,
    #[error("Value is outside the permissible range")]
    OutOfRange,
    #[error("Unexpected value")]
    InvalidValue,
    #[error("Words count error")]
    WordsCountError,
}
