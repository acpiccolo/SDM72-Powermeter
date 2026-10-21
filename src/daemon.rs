use anyhow::{Context, Result};
use log::{info, warn};
use sdm72_lib::tokio_sync_safe_client::SafeClient;
use std::{
    sync::{Arc, Condvar, Mutex},
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

    let shutdown_pair = Arc::new((Mutex::new(false), Condvar::new()));

    if let Err(e) = ctrlc::set_handler({
        let shutdown_pair = shutdown_pair.clone();
        move || {
            info!("Received shutdown signal. Stopping daemon gracefully...");
            let (lock, cvar) = &*shutdown_pair;
            if let Ok(mut shutdown) = lock.lock() {
                *shutdown = true;
                cvar.notify_all();
            }
        }
    }) {
        warn!("Failed to set Ctrl-C handler: {e}");
    }

    let (lock, cvar) = &*shutdown_pair;
    let mut shutdown = lock.lock().unwrap();

    while !*shutdown {
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

        if *shutdown {
            break;
        }

        let result = cvar.wait_timeout(shutdown, interval).unwrap();
        shutdown = result.0;
    }

    if let Some(publisher) = &mqtt_publisher {
        publisher.disconnect();
    }

    info!("Daemon stopped cleanly.");
    Ok(())
}
