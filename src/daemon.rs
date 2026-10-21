use anyhow::{Context, Result};
use log::{info, warn};
use sdm72_lib::tokio_sync_safe_client::SafeClient;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::{commandline, mqtt};

pub fn run(
    client: &mut SafeClient,
    output: commandline::DaemonOutput,
    interval: Duration,
    no_json: bool,
) -> Result<()> {
    let mqtt_publisher = match &output {
        commandline::DaemonOutput::Mqtt { mqtt_config } => {
            Some(mqtt::MqttPublisher::new(mqtt_config)?)
        }
        commandline::DaemonOutput::Console => None,
    };

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    if let Err(e) = ctrlc::set_handler(move || {
        info!("Received shutdown signal. Stopping daemon gracefully...");
        r.store(false, Ordering::SeqCst);
    }) {
        warn!("Failed to set Ctrl-C handler: {e}");
    }

    while running.load(Ordering::SeqCst) {
        let values = client
            .read_all(&interval)
            .with_context(|| "Cannot read all values")?;

        match &output {
            commandline::DaemonOutput::Console => {
                if no_json {
                    println!("{values}");
                } else {
                    println!("{}", serde_json::to_string_pretty(&values)?);
                }
            }
            commandline::DaemonOutput::Mqtt { .. } => {
                if let Some(publisher) = &mqtt_publisher {
                    publisher.publish_values(&values, no_json)?;
                }
            }
        }

        let sleep_step = Duration::from_millis(100).min(interval);
        let mut elapsed = Duration::ZERO;
        while elapsed < interval && running.load(Ordering::SeqCst) {
            std::thread::sleep(sleep_step);
            elapsed += sleep_step;
        }
    }

    if let Some(publisher) = &mqtt_publisher {
        if let Err(e) = publisher.disconnect() {
            warn!("Error disconnecting from MQTT broker: {e}");
        }
    }

    info!("Daemon stopped cleanly.");
    Ok(())
}
