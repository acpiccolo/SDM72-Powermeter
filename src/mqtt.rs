use anyhow::{Context, Result};
use paho_mqtt::{Client, ConnectOptionsBuilder, CreateOptionsBuilder};
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize, Clone)]
pub struct MqttConfig {
    uri: String,
    username: Option<String>,
    password: Option<String>,
    #[serde(default = "MqttConfig::default_topic")]
    topic: String,
    #[serde(default = "MqttConfig::default_qos")]
    qos: i32,
    #[serde(default = "MqttConfig::default_client_id")]
    client_id: String,
    #[serde(
        default = "MqttConfig::default_operation_timeout",
        with = "humantime_serde"
    )]
    oparation_timeout: Duration,
    #[serde(
        default = "MqttConfig::default_keep_alive_interval",
        with = "humantime_serde"
    )]
    keep_alive_interval: Duration,
    #[serde(
        default = "MqttConfig::default_auto_reconnect_interval_min",
        with = "humantime_serde"
    )]
    auto_reconnect_interval_min: Duration,
    #[serde(
        default = "MqttConfig::default_auto_reconnect_interval_max",
        with = "humantime_serde"
    )]
    auto_reconnect_interval_max: Duration,
}

impl MqttConfig {
    fn default_topic() -> String {
        "sdm72".into()
    }

    fn default_qos() -> i32 {
        0
    }

    fn generate_random_string(len: usize) -> String {
        use rand::Rng;
        use rand::distr::Alphanumeric;

        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(len)
            .map(char::from)
            .collect()
    }

    fn default_client_id() -> String {
        format!("sdm72-{}", Self::generate_random_string(8))
    }

    fn default_operation_timeout() -> Duration {
        Duration::from_secs(10)
    }

    fn default_keep_alive_interval() -> Duration {
        Duration::from_secs(30)
    }

    fn default_auto_reconnect_interval_min() -> Duration {
        Duration::from_secs(1)
    }

    fn default_auto_reconnect_interval_max() -> Duration {
        Duration::from_secs(30)
    }

    pub const DEFAULT_CONFIG_FILE: &str = "mqtt.yaml";

    pub fn load(config_file_path: &str) -> Result<Self> {
        log::debug!("Loading config file from {config_file_path:?}");
        let config_file = std::fs::File::open(config_file_path)
            .with_context(|| format!("Cannot open MQTT config file {config_file_path:?}"))?;
        let config: Self = serde_yaml::from_reader(&config_file)
            .with_context(|| format!("Cannot read MQTT config from file: {config_file_path:?}"))?;
        Ok(config)
    }

    pub fn create_client(&self) -> Result<Client> {
        let create_opts = CreateOptionsBuilder::new()
            .server_uri(&self.uri)
            .client_id(&self.client_id)
            .persistence(None) // In-memory persistence
            .finalize();

        let mut client = Client::new(create_opts)
            .with_context(|| format!("Error creating MQTT client for server: {}", self.uri))?;

        client.set_timeout(self.oparation_timeout);

        let mut conn_builder = ConnectOptionsBuilder::new();
        conn_builder
            .keep_alive_interval(self.keep_alive_interval)
            .clean_session(true) // Typically true for telemetry publishers
            .automatic_reconnect(
                self.auto_reconnect_interval_min,
                self.auto_reconnect_interval_max,
            ); // Enable auto-reconnect

        if let Some(user_name_str) = &self.username {
            conn_builder.user_name(user_name_str.as_str());
        }
        if let Some(password_str) = &self.password {
            conn_builder.password(password_str.as_str());
        }
        let conn_opts = conn_builder.finalize();

        log::info!(
            "Attempting to connect to MQTT broker: {} with client_id: {}",
            self.uri,
            self.client_id
        );

        client
            .connect(conn_opts)
            .with_context(|| "Failed to connect to MQTT broker")?;
        log::info!("Connected to MQTT broker.");
        Ok(client)
    }
}

pub fn run_mqtt_daemon(
    d: &mut sdm72_lib::tokio_sync_client::SDM72,
    delay: &Duration,
    poll_interval: &Duration,
    config_file: &str,
    no_json: bool,
) -> Result<()> {
    let config = MqttConfig::load(config_file)?;
    let cli = config.create_client()?;

    loop {
        let values = d
            .read_all(delay)
            .with_context(|| "Cannot read all values")?;

        macro_rules! pub_msg {
            ($label:expr, $val:expr) => {
                cli.publish(paho_mqtt::Message::new(
                    format!("{}/{}", config.topic, $label),
                    $val.to_string(),
                    config.qos as i32,
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

        if !no_json {
            let payload = serde_json::to_string(&values)?;
            let msg = paho_mqtt::Message::new(
                format!("{}/JSON", config.topic),
                payload,
                config.qos as i32,
            );
            cli.publish(msg)
                .with_context(|| "Cannot publish MQTT message")?;
        }
        std::thread::sleep(*delay.max(poll_interval));
    }
}
