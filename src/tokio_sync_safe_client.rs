//! This module provides a thread-safe synchronous client for the SDM72 energy meter.
//!
//! The [`SafeClient`] struct wraps the core synchronous API and manages the Modbus
//! context within an `Arc<Mutex>`, allowing it to be shared across threads safely.
//!
//! # Example
//!
//! ```no_run
//! use sdm72_lib::{
//!     protocol::Address,
//!     tokio_sync_safe_client::SafeClient,
//! };
//! use tokio_modbus::client::sync::tcp;
//! use tokio_modbus::Slave;
//! use std::time::Duration;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let socket_addr = "192.168.1.100:502".parse()?;
//!     let ctx = tcp::connect_slave(socket_addr, Slave(*Address::default()))?;
//!     let mut client = SafeClient::new(ctx);
//!
//!     let values = client.read_all(&Duration::from_millis(100))?;
//!
//!     println!("Successfully read values: {:#?}", values);
//!
//!     Ok(())
//! }
//! ```

use crate::{
    protocol as proto,
    tokio_common::{AllSettings, AllValues, Result},
    tokio_sync::SDM72,
};
use std::sync::{Arc, Mutex};
use tokio_modbus::{client::sync::Context, prelude::SlaveContext};

/// A thread-safe synchronous client for the SDM72 energy meter.
#[derive(Clone)]
pub struct SafeClient {
    ctx: Arc<Mutex<Context>>,
}

macro_rules! read_holding {
    ($func_name:ident, $ty:ident) => {
        paste::item! {
            #[doc = "Reads the [`proto::" $ty "`] value from the Modbus holding register."]
            pub fn $func_name(&mut self) -> Result<proto::$ty> {
                let mut ctx = self.ctx.lock().unwrap();
                SDM72::$func_name(&mut ctx)
            }
        }
    };
}

macro_rules! write_holding {
    ($func_name:ident, $ty:ident) => {
        paste::item! {
            #[doc = "Writes the [`proto::" $ty "`] value to the Modbus holding register."]
            pub fn [< set_ $func_name >](&mut self, value: proto::$ty) -> Result<()> {
                let mut ctx = self.ctx.lock().unwrap();
                SDM72::[< set_ $func_name >](&mut ctx, value)
            }
        }
    };
}

impl SafeClient {
    /// Constructs a new `SafeClient` with the given `tokio-modbus` context.
    pub fn new(ctx: Context) -> Self {
        Self {
            ctx: Arc::new(Mutex::new(ctx)),
        }
    }

    /// Constructs a new `SafeClient` from a shared context.
    pub fn from_shared_context(ctx: Arc<Mutex<Context>>) -> Self {
        Self { ctx }
    }

    /// Returns a clone of the shared context.
    pub fn shared_context(&self) -> Arc<Mutex<Context>> {
        self.ctx.clone()
    }

    read_holding!(system_type, SystemType);
    write_holding!(system_type, SystemType);
    read_holding!(pulse_width, PulseWidth);
    write_holding!(pulse_width, PulseWidth);
    read_holding!(kppa, KPPA);

    /// Sets the Key Parameter Programming Authorization (KPPA).
    ///
    /// This is required to change settings on the meter.
    pub fn set_kppa(&mut self, password: proto::Password) -> Result<()> {
        let mut ctx = self.ctx.lock().unwrap();
        SDM72::set_kppa(&mut ctx, password)
    }

    read_holding!(parity_and_stop_bit, ParityAndStopBit);
    write_holding!(parity_and_stop_bit, ParityAndStopBit);
    read_holding!(address, Address);

    pub fn set_address(&mut self, value: proto::Address) -> Result<()> {
        let mut ctx = self.ctx.lock().unwrap();
        SDM72::set_address(&mut ctx, value)?;
        ctx.set_slave(tokio_modbus::Slave(*value));
        Ok(())
    }

    read_holding!(pulse_constant, PulseConstant);
    write_holding!(pulse_constant, PulseConstant);
    read_holding!(password, Password);
    write_holding!(password, Password);
    read_holding!(baud_rate, BaudRate);
    write_holding!(baud_rate, BaudRate);
    read_holding!(auto_scroll_time, AutoScrollTime);
    write_holding!(auto_scroll_time, AutoScrollTime);
    read_holding!(backlight_time, BacklightTime);
    write_holding!(backlight_time, BacklightTime);
    read_holding!(pulse_energy_type, PulseEnergyType);
    write_holding!(pulse_energy_type, PulseEnergyType);

    /// Resets the historical data on the meter.
    ///
    /// This requires KPPA authorization.
    pub fn reset_historical_data(&mut self) -> Result<()> {
        let mut ctx = self.ctx.lock().unwrap();
        SDM72::reset_historical_data(&mut ctx)
    }

    read_holding!(serial_number, SerialNumber);
    read_holding!(meter_code, MeterCode);
    read_holding!(software_version, SoftwareVersion);

    /// Reads all settings from the meter in a single batch operation.
    pub fn read_all_settings(&mut self, delay: &std::time::Duration) -> Result<AllSettings> {
        let mut ctx = self.ctx.lock().unwrap();
        SDM72::read_all_settings(&mut ctx, delay)
    }

    /// Reads all measurement values from the meter in a single batch operation.
    pub fn read_all(&mut self, delay: &std::time::Duration) -> Result<AllValues> {
        let mut ctx = self.ctx.lock().unwrap();
        SDM72::read_all(&mut ctx, delay)
    }
}
