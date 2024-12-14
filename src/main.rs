use chrono::Utc;
use clap::Parser;
use ctrlc;
use run_script;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
extern crate yaml_rust;
use serde_json::{json, Map, Value};
use ureq;
use yaml_rust::YamlLoader;

mod hashenc;
mod scheduler;
mod url;
mod validator;

// Define the command line arguments
#[derive(Parser, Debug)]
#[command(
    version = "0.1.0",
    about = "A worker process that pushes data to the dmon server.",
    long_about = None
)]
struct Args {
    /// YAML configuration file path.
    #[arg(short, long, default_value = "worker_config.yaml", value_hint = clap::ValueHint::FilePath)]
    config: String,

    /// Run once immediately and exit. Default is to run on schedule.
    #[arg(short, long)]
    run_now: bool,
}

fn scheduled_push(yaml_config: &yaml_rust::Yaml) -> Result<(), Box<dyn std::error::Error>> {
    let key_string = yaml_config["worker"]["pk"]
        .as_str()
        .ok_or("missing encryption key string in config!")?;
    let iv_u8 = hashenc::generate_rand_iv().unwrap();

    let url = url::Url::new(
        &yaml_config["server"]["host"].as_str().unwrap_or_default(),
        Some(yaml_config["server"]["port"].as_i64().unwrap_or_default() as u16),
        &format!(
            "/api/workers/{}/push",
            yaml_config["worker"]["id"].as_str().unwrap_or_default()
        ),
    );

    let mut object_map = Map::new();
    match yaml_config["worker"]["modules"].as_vec() {
        Some(modules) => {
            for module in modules {
                // Each module is a hash with one key-value pair
                if let Some(hash) = module.as_hash() {
                    // Get the first (and only) key-value pair
                    if let Some((key, value)) = hash.iter().next() {
                        if let (Some(name), Some(exec)) = (key.as_str(), value.as_str()) {
                            println!(
                                "[{}] module running: {} | {}",
                                yaml_config["worker"]["id"].as_str().unwrap_or_default(),
                                name,
                                exec
                            );
                            let (_code, output, _error) = run_script::run_script!(exec).unwrap();

                            object_map.insert(name.into(), serde_json::from_str(&output)?);
                        }
                    }
                }
            }
        }
        None => {
            eprintln!(
                "[{}] no modules found in configuration!",
                yaml_config["worker"]["id"].as_str().unwrap_or_default()
            );
        }
    }

    let post_payload = json!({
        "payload": hashenc::encrypt_payload(&Value::Object(object_map).to_string(), key_string, iv_u8)?,
        "iv": hashenc::generate_base64_iv(iv_u8)?,
    });

    let resp: String = ureq::post(&url.format(false))
        .send_string(&post_payload.to_string())?
        .into_string()?;

    println!(
        "[{}] response: {}",
        yaml_config["worker"]["id"].as_str().unwrap_or_default(),
        resp
    );

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load arguments and configuration
    let args = Args::parse();
    let data = fs::read_to_string(&args.config).expect("Unable to read configuration file");

    if data.is_empty() {
        return Err("No data in configuration file!".into());
    }

    let yaml_raw = YamlLoader::load_from_str(&data)?;
    if yaml_raw.is_empty() {
        return Err("Invalid YAML configuration!".into());
    }
    let yaml_config = &yaml_raw[0];

    validator::validate_config(&yaml_config)?;

    // Create a flag for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("Stopping gracefully...");
        r.store(false, Ordering::SeqCst);
    })?;

    if args.run_now {
        scheduled_push(&yaml_config)?;
        return Ok(());
    }

    // Create scheduled task
    let yaml_config_clone = yaml_config.clone();
    let task = scheduler::ScheduledTask::new(
        yaml_config["worker"]["id"].as_str().unwrap(),
        yaml_config["worker"]["schedule"].as_str().unwrap(), // Every 5 minutes
        move || -> Result<(), Box<dyn std::error::Error>> {
            thread::sleep(Duration::from_secs(1));

            scheduled_push(&yaml_config_clone)?;
            Ok(())
        },
    )?;

    println!("Waiting for time to push... Press Ctrl+C to stop");

    while running.load(Ordering::SeqCst) {
        let now = Utc::now();

        if let Some(next_time) = task.schedule.upcoming(Utc).next() {
            let duration = next_time.signed_duration_since(now);
            let wait_time = Duration::from_secs(duration.num_seconds().max(0) as u64);

            println!(
                "[{}] next execution scheduled for: {}",
                yaml_config["worker"]["id"].as_str().unwrap(),
                next_time
            );

            // Wait for either the next scheduled time or shutdown signal
            for _ in 0..wait_time.as_secs() {
                if !running.load(Ordering::SeqCst) {
                    return Ok(());
                }
                thread::sleep(Duration::from_secs(1));
            }

            if let Err(e) = task.run() {
                eprintln!("error executing task: {}", e);
            }
        }
    }

    println!("exiting...");
    Ok(())
}
