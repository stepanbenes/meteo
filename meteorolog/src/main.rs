mod rtl;

use influxdb::{Client, ReadQuery, Timestamp, Type, WriteQuery};
use std::env;
use std::sync::mpsc;

use crate::rtl::RTL433Message;

#[tokio::main]
async fn main() -> Result<(), influxdb::Error> {
    let url = env::var("INFLUXDB_URL").expect("INFLUXDB_URL not set");
    let influxdb_name = env::var("INFLUXDB_DATABASE").expect("INFLUXDB_DATABASE not set");

    let client = Client::new(&url, "");

    // Create database if it doesn't exist
    let create_db_query = ReadQuery::new("CREATE DATABASE meteo_db");
    client.query(create_db_query).await?;

    let client = Client::new(url, influxdb_name);

    // Query average temperature for last hour
    //let query = ReadQuery::new("SELECT MEAN(temperature) FROM weather WHERE time > now() - 1h");
    //let result = client.query(query).await?;
    //println!("Raw result: {:?}", result);

    let (tx, rx) = mpsc::channel();
    let join_handle = rtl::start(tx);

    for message in rx {
        // Process the received RTL-433 messages
        println!("Received message: {:?}", message);

        match message {
            RTL433Message {
                model,
                channel: Some(channel),
                time,
                temperature_c: Some(temperature),
                humidity: Some(humidity),
                ..
            } if model == "LaCrosse-TX141THBv2" => {
                let write_query =
                    WriteQuery::new(Timestamp::Seconds(time.timestamp() as u128), "weather")
                        .add_field("temperature", Type::Float(temperature))
                        .add_field("humidity", Type::Float(humidity))
                        .add_tag("location", format!("indoor_{}", channel).as_str());
                let response = client.query(write_query).await?;
                println!("Write response: {:?}", response);
            }
            _ => {}
        }
    }

    join_handle.join().expect("RTL-433 thread panicked");

    Ok(())
}
