use crate::protocol as proto;

/// Represent possible errors encountered during Modbus communication.
///
/// This enum encapsulates all potential errors that might occur while
/// interacting with the SDM72 device.
///
/// # Variants
///
/// * 'SDM72Error' - Failure from the SDM72 protocol.
/// * 'ModbusError' - Protocol or transport errors.
/// * 'ModbusException' - A server (slave) exception.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("SDM72 error: {0}")]
    SDM72Error(#[from] crate::Error),
    #[error("Modbus error: {0}")]
    ModbusError(#[from] tokio_modbus::Error),
    #[error("Modbus exception: {0}")]
    ModbusException(#[from] tokio_modbus::ExceptionCode),
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AllSettings {
    pub system_type: proto::SystemType,
    pub pulse_width: proto::PulseWidth,
    pub kppa: proto::KPPA,
    pub parity_and_stop_bit: proto::ParityAndStopBit,
    pub address: proto::Address,
    pub pulse_constant: proto::PulseConstant,
    pub password: proto::Password,
    pub baud_rate: proto::BaudRate,
    pub auto_scroll_time: proto::AutoScrollTime,
    pub backlight_time: proto::BacklightTime,
    pub pulse_energy_type: proto::PulseEnergyType,
    pub serial_number: proto::SerialNumber,
    pub meter_code: proto::MeterCode,
    pub software_version: proto::SoftwareVersion,
}
impl std::fmt::Display for AllSettings {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(fmt, "System type: {}", self.system_type)?;
        writeln!(fmt, "Pulse width: {}", self.pulse_width)?;
        writeln!(fmt, "KPPA: {}", self.kppa)?;
        writeln!(fmt, "Parity and stop bit: {}", self.parity_and_stop_bit)?;
        writeln!(fmt, "Address: {}", self.address)?;
        writeln!(fmt, "Pulse constant: {}", self.pulse_constant)?;
        writeln!(fmt, "Password: {}", self.password)?;
        writeln!(fmt, "Baud rate: {}", self.baud_rate)?;
        writeln!(fmt, "Auto scroll time: {}", self.auto_scroll_time)?;
        writeln!(fmt, "Backlight time: {}", self.backlight_time)?;
        writeln!(fmt, "Pulse energy type: {}", self.pulse_energy_type)?;
        writeln!(fmt, "Serial number: {}", self.serial_number)?;
        writeln!(fmt, "Meter code: {}", self.meter_code)?;
        write!(fmt, "Software version: {}", self.software_version)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AllValues {
    pub l1_voltage: proto::L1Voltage,
    pub l2_voltage: proto::L2Voltage,
    pub l3_voltage: proto::L3Voltage,
    pub l1_current: proto::L1Current,
    pub l2_current: proto::L2Current,
    pub l3_current: proto::L3Current,
    pub l1_power_active: proto::L1PowerActive,
    pub l2_power_active: proto::L2PowerActive,
    pub l3_power_active: proto::L3PowerActive,
    pub l1_power_apparent: proto::L1PowerApparent,
    pub l2_power_apparent: proto::L2PowerApparent,
    pub l3_power_apparent: proto::L3PowerApparent,
    pub l1_power_reactive: proto::L1PowerReactive,
    pub l2_power_reactive: proto::L2PowerReactive,
    pub l3_power_reactive: proto::L3PowerReactive,
    pub l1_power_factor: proto::L1PowerFactor,
    pub l2_power_factor: proto::L2PowerFactor,
    pub l3_power_factor: proto::L3PowerFactor,
    #[cfg_attr(feature = "serde", serde(rename = "l-n_average_voltage"))]
    pub ln_average_voltage: proto::LtoNAverageVoltage,
    #[cfg_attr(feature = "serde", serde(rename = "l-n_average_current"))]
    pub ln_average_current: proto::LtoNAverageCurrent,
    pub total_line_current: proto::TotalLineCurrent,
    pub total_power: proto::TotalPower,
    pub total_power_apparent: proto::TotalPowerApparent,
    pub total_power_reactive: proto::TotalPowerReactive,
    pub total_power_factor: proto::TotalPowerFactor,
    pub frequency: proto::Frequency,
    pub import_energy_active: proto::ImportEnergyActive,
    pub export_energy_active: proto::ExportEnergyActive,

    #[cfg_attr(feature = "serde", serde(rename = "l1-l2_voltage"))]
    pub l1l2_voltage: proto::L1ToL2Voltage,
    #[cfg_attr(feature = "serde", serde(rename = "l2-l3_voltage"))]
    pub l2l3_voltage: proto::L2ToL3Voltage,
    #[cfg_attr(feature = "serde", serde(rename = "l3-l1_voltage"))]
    pub l3l1_voltage: proto::L3ToL1Voltage,
    #[cfg_attr(feature = "serde", serde(rename = "l-l_average_voltage"))]
    pub ll_average_voltage: proto::LtoLAverageVoltage,
    pub neutral_current: proto::NeutralCurrent,

    pub total_energy_active: proto::TotalEnergyActive,
    pub total_energy_reactive: proto::TotalEnergyReactive,
    pub resettable_total_energy_active: proto::ResettableTotalEnergyActive,
    pub resettable_total_energy_reactive: proto::ResettableTotalEnergyReactive,
    pub resettable_import_energy_active: proto::ResettableImportEnergyActive,
    pub resettable_export_energy_active: proto::ResettableExportEnergyActive,
    #[cfg_attr(feature = "serde", serde(rename = "net_kwh_import_-_export"))]
    pub net_kwh: proto::NetKwh,

    pub import_total_energy_active: proto::ImportTotalPowerActive,
    pub export_total_energy_active: proto::ExportTotalPowerActive,
}
impl std::fmt::Display for AllValues {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(fmt, "L1 Voltage: {}", self.l1_voltage)?;
        writeln!(fmt, "L2 Voltage: {}", self.l2_voltage)?;
        writeln!(fmt, "L3 Voltage: {}", self.l3_voltage)?;
        writeln!(fmt, "L1 Current: {}", self.l1_current)?;
        writeln!(fmt, "L2 Current: {}", self.l2_current)?;
        writeln!(fmt, "L3 Current: {}", self.l3_current)?;
        writeln!(fmt, "L1 Power Active: {}", self.l1_power_active)?;
        writeln!(fmt, "L2 Power Active: {}", self.l2_power_active)?;
        writeln!(fmt, "L3 Power Active: {}", self.l3_power_active)?;
        writeln!(fmt, "L1 Power Apparent: {}", self.l1_power_apparent)?;
        writeln!(fmt, "L2 Power Apparent: {}", self.l2_power_apparent)?;
        writeln!(fmt, "L3 Power Apparent: {}", self.l3_power_apparent)?;
        writeln!(fmt, "L1 Power Reactive: {}", self.l1_power_reactive)?;
        writeln!(fmt, "L2 Power Reactive: {}", self.l2_power_reactive)?;
        writeln!(fmt, "L3 Power Reactive: {}", self.l3_power_reactive)?;
        writeln!(fmt, "L1 Power Factor: {}", self.l1_power_factor)?;
        writeln!(fmt, "L2 Power Factor: {}", self.l2_power_factor)?;
        writeln!(fmt, "L3 Power Factor: {}", self.l3_power_factor)?;
        writeln!(fmt, "L-N average Voltage: {}", self.ln_average_voltage)?;
        writeln!(fmt, "L-N average Current: {}", self.ln_average_current)?;
        writeln!(fmt, "Total Line Current: {}", self.total_line_current)?;
        writeln!(fmt, "Total Power: {}", self.total_power)?;
        writeln!(fmt, "Total Power Apparent: {}", self.total_power_apparent)?;
        writeln!(fmt, "Total Power Reactive: {}", self.total_power_reactive)?;
        writeln!(fmt, "Total Power Factor: {}", self.total_power_factor)?;
        writeln!(fmt, "Frequency: {}", self.frequency)?;
        writeln!(fmt, "Import Energy Active: {}", self.import_energy_active)?;
        writeln!(fmt, "Export Energy Active: {}", self.export_energy_active)?;

        writeln!(fmt, "L1-L2 Voltage: {}", self.l1l2_voltage)?;
        writeln!(fmt, "L2-L3 Voltage: {}", self.l2l3_voltage)?;
        writeln!(fmt, "L3-L1 Voltage: {}", self.l3l1_voltage)?;
        writeln!(fmt, "L-L average Voltage: {}", self.ll_average_voltage)?;
        writeln!(fmt, "Neutral Current: {}", self.neutral_current)?;

        writeln!(fmt, "Total Energy Active: {}", self.total_energy_active)?;
        writeln!(fmt, "Total Energy Reactive: {}", self.total_energy_reactive)?;
        writeln!(
            fmt,
            "Resettable Total Energy Active: {}",
            self.resettable_total_energy_active
        )?;
        writeln!(
            fmt,
            "Resettable Total Energy Reactive: {}",
            self.resettable_total_energy_reactive
        )?;
        writeln!(
            fmt,
            "Resettable Import Energy Active: {}",
            self.resettable_import_energy_active
        )?;
        writeln!(
            fmt,
            "Resettable Export Energy Active: {}",
            self.resettable_export_energy_active
        )?;
        writeln!(fmt, "Net kWh (Import - Export): {}", self.net_kwh)?;

        writeln!(
            fmt,
            "Import Total Energy Active: {}",
            self.import_total_energy_active
        )?;
        write!(
            fmt,
            "Export Total Energy Active: {}",
            self.export_total_energy_active
        )?;

        Ok(())
    }
}
