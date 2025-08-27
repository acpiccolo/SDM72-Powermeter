//! This module provides a synchronous client for the SDM72 energy meter.
//!
//! The [`SDM72`] struct is the main entry point for interacting with the meter. It
//! wraps a synchronous `tokio-modbus` context and provides high-level methods
//! for reading and writing meter data.
//!
//! This client is suitable for applications that do not require asynchronous
//! operations and can be used in environments without the `tokio` runtime. For
//! applications that use `tokio`, the asynchronous client in the
//! [`crate::tokio_async`] module may be more suitable.
//!
//! # Example
//!
//! ```no_run
//! use sdm72_lib::{
//!     protocol::Address,
//!     tokio_sync::SDM72,
//! };
//! use tokio_modbus::client::sync::tcp;
//! use tokio_modbus::Slave;
//! use std::time::Duration;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let socket_addr = "192.168.1.100:502".parse()?;
//!     let mut ctx = tcp::connect_slave(socket_addr, Slave(*Address::default()))?;
//!
//!     let values = SDM72::read_all(&mut ctx, &Duration::from_millis(100))?;
//!
//!     println!("Successfully read values: {:#?}", values);
//!
//!     Ok(())
//! }
//! ```

use crate::{
    protocol::{self as proto, ModbusParam},
    tokio_common::{AllSettings, AllValues, Result},
};
use tokio_modbus::prelude::{SyncReader, SyncWriter};

/// A synchronous client for the SDM72 energy meter.
///
/// This struct provides a high-level interface for interacting with the SDM72
/// energy meter. It uses a synchronous `tokio-modbus` context for communication.
/// An instance of this client can be created using the [`new`](#method.new) method.
pub struct SDM72;

/// A macro to generate a function for reading a holding register.
macro_rules! read_holding {
    ($func_name:expr, $ty:ident) => {
        paste::item! {
            #[doc = "Reads the [`proto::" $ty "`] value from the Modbus holding register."]
            pub fn $func_name(ctx: &mut tokio_modbus::client::sync::Context) -> Result<proto::$ty> {
                let rsp = ctx
                    .read_holding_registers(<proto::$ty>::ADDRESS, <proto::$ty>::QUANTITY)??;
                Ok(<proto::$ty>::decode_from_holding_registers(&rsp)?)
            }
        }
    };
}

/// A macro to generate a function for writing a holding register.
macro_rules! write_holding {
    ($func_name:expr, $ty:ident) => {
        paste::item! {
            #[doc = "Writes the [`proto::" $ty "`] value to the Modbus holding register."]
            pub fn [< set_ $func_name >](ctx: &mut tokio_modbus::client::sync::Context, value: proto::$ty) -> Result<()> {
                Ok(ctx.write_multiple_registers(
                    <proto::$ty>::ADDRESS,
                    &value.encode_for_write_registers(),
                )??)
            }
        }
    };
}

impl SDM72 {
    read_holding!(system_type, SystemType);
    write_holding!(system_type, SystemType);
    read_holding!(pulse_width, PulseWidth);
    write_holding!(pulse_width, PulseWidth);
    read_holding!(kppa, KPPA);
    /// Sets the Key Parameter Programming Authorization (KPPA).
    ///
    /// This is required to change settings on the meter.
    pub fn set_kppa(
        ctx: &mut tokio_modbus::client::sync::Context,
        password: proto::Password,
    ) -> Result<()> {
        Ok(ctx.write_multiple_registers(
            proto::KPPA::ADDRESS,
            &proto::KPPA::encode_for_write_registers(password),
        )??)
    }
    read_holding!(parity_and_stop_bit, ParityAndStopBit);
    write_holding!(parity_and_stop_bit, ParityAndStopBit);
    read_holding!(address, Address);
    write_holding!(address, Address);
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
    pub fn reset_historical_data(ctx: &mut tokio_modbus::client::sync::Context) -> Result<()> {
        Ok(ctx.write_multiple_registers(
            proto::ResetHistoricalData::ADDRESS,
            &proto::ResetHistoricalData::encode_for_write_registers(),
        )??)
    }
    read_holding!(serial_number, SerialNumber);
    read_holding!(meter_code, MeterCode);
    read_holding!(software_version, SoftwareVersion);

    /// Reads all settings from the meter in a single batch operation.
    ///
    /// This method is more efficient than reading each setting individually because
    /// it minimizes the number of Modbus requests by batching them. The SDM72
    /// meter has a limit of 30 parameters per request, so this function splits
    /// the reads into multiple batches.
    ///
    /// # Arguments
    ///
    /// * `delay` - The delay to be inserted between Modbus requests. This is
    ///   necessary for some Modbus devices, which may need a short pause to
    ///   process a request before they are ready to accept the next one. A
    ///   typical value is 100 milliseconds, but this may vary depending on the
    ///   device and network conditions.
    pub fn read_all_settings(
        ctx: &mut tokio_modbus::client::sync::Context,
        delay: &std::time::Duration,
    ) -> Result<AllSettings> {
        let offset1 = proto::SystemType::ADDRESS;
        let quantity =
            { proto::PulseEnergyType::ADDRESS - offset1 + proto::PulseEnergyType::QUANTITY };
        let rsp1 = ctx.read_holding_registers(offset1, quantity)??;

        std::thread::sleep(*delay);
        let serial_number = Self::serial_number(ctx)?;
        std::thread::sleep(*delay);
        let meter_code = Self::meter_code(ctx)?;
        std::thread::sleep(*delay);
        let software_version = Self::software_version(ctx)?;

        Ok(AllSettings {
            system_type: crate::decode_subset_item_from_holding_register!(
                offset1,
                proto::SystemType,
                &rsp1
            )?,
            pulse_width: crate::decode_subset_item_from_holding_register!(
                offset1,
                proto::PulseWidth,
                &rsp1
            )?,
            kppa: crate::decode_subset_item_from_holding_register!(offset1, proto::KPPA, &rsp1)?,
            parity_and_stop_bit: crate::decode_subset_item_from_holding_register!(
                offset1,
                proto::ParityAndStopBit,
                &rsp1
            )?,
            address: crate::decode_subset_item_from_holding_register!(
                offset1,
                proto::Address,
                &rsp1
            )?,
            pulse_constant: crate::decode_subset_item_from_holding_register!(
                offset1,
                proto::PulseConstant,
                &rsp1
            )?,
            password: crate::decode_subset_item_from_holding_register!(
                offset1,
                proto::Password,
                &rsp1
            )?,
            baud_rate: crate::decode_subset_item_from_holding_register!(
                offset1,
                proto::BaudRate,
                &rsp1
            )?,
            auto_scroll_time: crate::decode_subset_item_from_holding_register!(
                offset1,
                proto::AutoScrollTime,
                &rsp1
            )?,
            backlight_time: crate::decode_subset_item_from_holding_register!(
                offset1,
                proto::BacklightTime,
                &rsp1
            )?,
            pulse_energy_type: crate::decode_subset_item_from_holding_register!(
                offset1,
                proto::PulseEnergyType,
                &rsp1
            )?,
            serial_number,
            meter_code,
            software_version,
        })
    }

    /// Reads all measurement values from the meter in a single batch operation.
    ///
    /// This method is more efficient than reading each value individually because
    /// it minimizes the number of Modbus requests by batching them. The SDM72
    /// meter has a limit of 30 parameters per request, so this function splits
    /// the reads into multiple batches.
    ///
    /// # Arguments
    ///
    /// * `delay` - The delay to be inserted between Modbus requests. This is
    ///   necessary for some Modbus devices, which may need a short pause to
    ///   process a request before they are ready to accept the next one. A
    ///   typical value is 100 milliseconds, but this may vary depending on the
    ///   device and network conditions.
    pub fn read_all(
        ctx: &mut tokio_modbus::client::sync::Context,
        delay: &std::time::Duration,
    ) -> Result<AllValues> {
        let offset1 = proto::L1Voltage::ADDRESS;
        let quantity =
            { proto::ExportEnergyActive::ADDRESS - offset1 + proto::ExportEnergyActive::QUANTITY };
        let rsp1 = ctx.read_input_registers(offset1, quantity)??;

        std::thread::sleep(*delay);

        let offset2 = proto::L1ToL2Voltage::ADDRESS;
        let quantity =
            { proto::NeutralCurrent::ADDRESS - offset2 + proto::NeutralCurrent::QUANTITY };
        let rsp2 = ctx.read_input_registers(offset2, quantity)??;

        std::thread::sleep(*delay);

        let offset3 = proto::TotalEnergyActive::ADDRESS;
        let quantity = { proto::NetKwh::ADDRESS - offset3 + proto::NetKwh::QUANTITY };
        let rsp3 = ctx.read_input_registers(offset3, quantity)??;

        std::thread::sleep(*delay);

        let offset4 = proto::ImportTotalPowerActive::ADDRESS;
        let quantity = {
            proto::ExportTotalPowerActive::ADDRESS - offset4
                + proto::ExportTotalPowerActive::QUANTITY
        };
        let rsp4 = ctx.read_input_registers(offset4, quantity)??;

        Ok(AllValues {
            l1_voltage: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::L1Voltage,
                &rsp1
            )?,
            l2_voltage: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::L2Voltage,
                &rsp1
            )?,
            l3_voltage: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::L3Voltage,
                &rsp1
            )?,
            l1_current: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::L1Current,
                &rsp1
            )?,
            l2_current: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::L2Current,
                &rsp1
            )?,
            l3_current: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::L3Current,
                &rsp1
            )?,
            l1_power_active: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::L1PowerActive,
                &rsp1
            )?,
            l2_power_active: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::L2PowerActive,
                &rsp1
            )?,
            l3_power_active: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::L3PowerActive,
                &rsp1
            )?,
            l1_power_apparent: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::L1PowerApparent,
                &rsp1
            )?,
            l2_power_apparent: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::L2PowerApparent,
                &rsp1
            )?,
            l3_power_apparent: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::L3PowerApparent,
                &rsp1
            )?,
            l1_power_reactive: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::L1PowerReactive,
                &rsp1
            )?,
            l2_power_reactive: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::L2PowerReactive,
                &rsp1
            )?,
            l3_power_reactive: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::L3PowerReactive,
                &rsp1
            )?,
            l1_power_factor: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::L1PowerFactor,
                &rsp1
            )?,
            l2_power_factor: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::L2PowerFactor,
                &rsp1
            )?,
            l3_power_factor: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::L3PowerFactor,
                &rsp1
            )?,
            ln_average_voltage: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::LtoNAverageVoltage,
                &rsp1
            )?,
            ln_average_current: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::LtoNAverageCurrent,
                &rsp1
            )?,
            total_line_current: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::TotalLineCurrent,
                &rsp1
            )?,
            total_power: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::TotalPower,
                &rsp1
            )?,
            total_power_apparent: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::TotalPowerApparent,
                &rsp1
            )?,
            total_power_reactive: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::TotalPowerReactive,
                &rsp1
            )?,
            total_power_factor: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::TotalPowerFactor,
                &rsp1
            )?,
            frequency: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::Frequency,
                &rsp1
            )?,
            import_energy_active: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::ImportEnergyActive,
                &rsp1
            )?,
            export_energy_active: crate::decode_subset_item_from_input_register!(
                offset1,
                proto::ExportEnergyActive,
                &rsp1
            )?,

            l1l2_voltage: crate::decode_subset_item_from_input_register!(
                offset2,
                proto::L1ToL2Voltage,
                &rsp2
            )?,
            l2l3_voltage: crate::decode_subset_item_from_input_register!(
                offset2,
                proto::L2ToL3Voltage,
                &rsp2
            )?,
            l3l1_voltage: crate::decode_subset_item_from_input_register!(
                offset2,
                proto::L3ToL1Voltage,
                &rsp2
            )?,
            ll_average_voltage: crate::decode_subset_item_from_input_register!(
                offset2,
                proto::LtoLAverageVoltage,
                &rsp2
            )?,
            neutral_current: crate::decode_subset_item_from_input_register!(
                offset2,
                proto::NeutralCurrent,
                &rsp2
            )?,

            total_energy_active: crate::decode_subset_item_from_input_register!(
                offset3,
                proto::TotalEnergyActive,
                &rsp3
            )?,
            total_energy_reactive: crate::decode_subset_item_from_input_register!(
                offset3,
                proto::TotalEnergyReactive,
                &rsp3
            )?,
            resettable_total_energy_active: crate::decode_subset_item_from_input_register!(
                offset3,
                proto::ResettableTotalEnergyActive,
                &rsp3
            )?,
            resettable_total_energy_reactive: crate::decode_subset_item_from_input_register!(
                offset3,
                proto::ResettableTotalEnergyReactive,
                &rsp3
            )?,
            resettable_import_energy_active: crate::decode_subset_item_from_input_register!(
                offset3,
                proto::ResettableImportEnergyActive,
                &rsp3
            )?,
            resettable_export_energy_active: crate::decode_subset_item_from_input_register!(
                offset3,
                proto::ResettableExportEnergyActive,
                &rsp3
            )?,
            net_kwh: crate::decode_subset_item_from_input_register!(offset3, proto::NetKwh, &rsp3)?,

            import_total_energy_active: crate::decode_subset_item_from_input_register!(
                offset4,
                proto::ImportTotalPowerActive,
                &rsp4
            )?,
            export_total_energy_active: crate::decode_subset_item_from_input_register!(
                offset4,
                proto::ExportTotalPowerActive,
                &rsp4
            )?,
        })
    }
}
