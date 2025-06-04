use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use std::error::Error;
use std::io::{self, BufRead};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};

// Custom deserialization for ID that can be either string or integer
fn deserialize_flexible_id<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    // First, deserialize to a dynamic JSON value
    let value = serde_json::Value::deserialize(deserializer)?;

    match value {
        // For string IDs (like "a1b2c3" or "12345")
        serde_json::Value::String(s) => Ok(Some(s)),

        // For numeric IDs (like 12345)
        serde_json::Value::Number(num) => {
            if let Some(n) = num.as_i64() {
                Ok(Some(n.to_string()))
            } else if let Some(n) = num.as_u64() {
                Ok(Some(n.to_string()))
            } else if let Some(n) = num.as_f64() {
                // Convert to integer if it's a whole number
                if n.fract() == 0.0 {
                    Ok(Some(n as i64).map(|i| i.to_string()))
                } else {
                    Ok(Some(n.to_string()))
                }
            } else {
                Ok(None)
            }
        }

        // For any other type, return None
        _ => Ok(None),
    }
}

// Custom deserialization for "Yes"/"No" string to Option<bool>
fn deserialize_yes_no<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        None => Ok(None),
        Some(s) => match s.as_str() {
            "Yes" | "yes" | "YES" | "true" | "TRUE" | "True" | "1" => Ok(Some(true)),
            "No" | "no" | "NO" | "false" | "FALSE" | "False" | "0" => Ok(Some(false)),
            _ => Ok(None),
        },
    }
}

// Custom deserializer for timestamp strings
fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;

    match s {
        None => Ok(Utc::now()),
        Some(time_str) => {
            // Handle multiple possible date formats from rtl_433

            // Format: "2023-04-15 14:32:56" (most common)
            if let Ok(naive_time) = NaiveDateTime::parse_from_str(&time_str, "%Y-%m-%d %H:%M:%S") {
                return Ok(Utc.from_utc_datetime(&naive_time));
            }

            // Format with fractional seconds: "2023-04-15 14:32:56.123"
            if let Ok(naive_time) = NaiveDateTime::parse_from_str(&time_str, "%Y-%m-%d %H:%M:%S%.f")
            {
                return Ok(Utc.from_utc_datetime(&naive_time));
            }

            // ISO 8601 format: "2023-04-15T14:32:56Z"
            if let Ok(datetime) = DateTime::parse_from_rfc3339(&time_str) {
                return Ok(datetime.with_timezone(&Utc));
            }

            // Unix timestamp (seconds since epoch)
            if let Ok(timestamp) = time_str.parse::<i64>() {
                return Ok(Utc
                    .timestamp_opt(timestamp, 0)
                    .single()
                    .unwrap_or_else(Utc::now));
            }

            // If none of the formats match, return the current time as fallback
            eprintln!("Unknown timestamp format: {}", time_str);
            Ok(Utc::now())
        }
    }
}

// Define flexible structures to handle various rtl_433 output formats
#[derive(Debug, Deserialize, Serialize)]
pub struct RTL433Message {
    // Common fields often found in rtl_433 JSON output
    #[serde(default, deserialize_with = "deserialize_timestamp")]
    pub time: DateTime<Utc>,
    #[serde(default)]
    pub model: String,
    #[serde(default, deserialize_with = "deserialize_flexible_id")]
    pub id: Option<String>,
    #[serde(default)]
    pub channel: Option<i64>,
    #[serde(default, rename = "temperature_C")]
    pub temperature_c: Option<f64>,
    #[serde(default)]
    pub humidity: Option<f64>,
    #[serde(default)]
    pub battery_ok: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_yes_no")]
    pub test: Option<bool>,
    #[serde(default)]
    pub mic: String, // Integrity

    #[serde(default, rename = "pressure_kPa")]
    pub pressure_kpa: Option<f64>,
    #[serde(default, rename = "pressure_PSI")]
    pub pressure_psi: Option<f64>,
}

pub fn start(tx: Sender<RTL433Message>) -> JoinHandle<()> {
    println!("RTL-433 Parser starting...");

    // Spawn a new thread
    thread::spawn(move || {
        parse_from_rtl433_process(tx).unwrap();
    })
}

fn parse_from_rtl433_process(tx: Sender<RTL433Message>) -> Result<(), Box<dyn Error>> {
    // Start rtl_433 process with JSON output
    let mut child = Command::new("rtl_433")
        .args(["-F", "json", "-M", "time:utc"])
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("Failed to open stdout");
    let reader = io::BufReader::new(stdout);

    for line in reader.lines() {
        match line {
            //Ok(json_line) => process_json_line(&json_line)?,
            Ok(json_line) => {
                match serde_json::from_str::<RTL433Message>(&json_line) {
                    Ok(message) => {
                        // Send the parsed message to the channel
                        tx.send(message)
                            .unwrap_or_else(|e| eprintln!("Failed to send message: {}", e));
                    }
                    Err(e) => eprintln!("Error processing JSON line: {}", e),
                }
            }
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn parse_from_stdin() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let reader = stdin.lock();

    for line in reader.lines() {
        match line {
            Ok(json_line) => process_json_line(&json_line)?,
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }

    Ok(())
}

fn process_json_line(json_line: &str) -> Result<(), Box<dyn Error>> {
    // Parse JSON
    match serde_json::from_str::<RTL433Message>(json_line) {
        Ok(message) => {
            println!(
                "Received message from model: {} with id: {} at {}",
                message.model,
                message.id.unwrap_or_default(),
                message.time
            );

            // Print temperature if available
            if let Some(temp) = message.temperature_c {
                println!("  Temperature: {:.1}°C", temp);
            }

            // Print humidity if available
            if let Some(humidity) = message.humidity {
                println!("  Humidity: {}%", humidity);
            }

            // Print pressure if available
            if let Some(pressure) = message.pressure_kpa {
                println!("  Pressure: {} kPa", pressure);
            }
            if let Some(pressure) = message.pressure_psi {
                println!("  Pressure: {} PSI", pressure);
            }

            // Print battery status if available
            if let Some(test) = message.test {
                println!("  Is test: {}", test);
            }

            println!(); // Empty line for readability
        }
        Err(e) => {
            eprintln!("Failed to parse JSON: {}", e);
            eprintln!("Raw line: {}", json_line);
        }
    }

    Ok(())
}
