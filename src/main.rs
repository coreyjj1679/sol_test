use serde_json::Value;
use sol_streamer::parser::*;
use std::time::SystemTime;
use warp::http::StatusCode;
use warp::Filter;

#[tokio::main]
async fn main() {
    let webhook = warp::post()
        .and(warp::path!("webhook"))
        .and(warp::body::bytes())
        .map(|bytes: bytes::Bytes| {
            let v: Result<Value, _> = serde_json::from_slice(bytes.as_ref());
            if let Err(e) = v {
                eprintln!("Error deserializing JSON: {}", e);
                return StatusCode::BAD_REQUEST;
            }

            let now = SystemTime::now();
            let timestamp = now
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0); // Default to 0 if an error occurs

            match parse_transaction(&v.unwrap().to_string()) {
                None => {
                    eprintln!("Failed to parse transaction.");
                    StatusCode::BAD_REQUEST
                }
                Some(tx) => {
                    println!("New tx found.");
                    println!("{:#?}", tx);
                    println!(
                        "tx_timestamp: {}, current_timestamp: {}, latency(s): {}",
                        tx.timestamp,
                        timestamp,
                        timestamp.abs_diff(tx.timestamp)
                    );
                    println!("==========================");
                    StatusCode::OK
                }
            }
        });

    //Create a simple healthcheck endpoint
    let health_route = warp::path!("health").map(|| StatusCode::OK);

    //Create the routes to pass to the server
    let routes = health_route
        .or(webhook)
        .with(warp::cors().allow_any_origin());

    println!("Webhook Started!");

    //Start the server on port 3000
    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}
