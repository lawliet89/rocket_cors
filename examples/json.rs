//! This example is to demonstrate the JSON serialization and deserialization of the Cors settings
//!
//! Note: This requires the `serialization` feature which is enabled by default.
use rocket_cors as cors;

use crate::cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use rocket::http::Method;

fn main() {
    // The default demonstrates the "All" serialization of several of the settings
    let default: CorsOptions = Default::default();

    let allowed_origins =
        AllowedOrigins::some(&["https://www.acme.com"], &["^https://(.+).acme.com$"]);

    let options = cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post, Method::Delete]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        expose_headers: ["Content-Type", "X-Custom"]
            .iter()
            .map(ToString::to_string)
            .collect(),
        max_age: Some(42),
        send_wildcard: false,
        fairing_route_base: "/mycors".to_string(),
        fairing_route_rank: 0,
    };

    println!("Default settings");
    println!("{}", serde_json::to_string_pretty(&default).unwrap());

    println!("Defined settings");
    println!("{}", serde_json::to_string_pretty(&options).unwrap());
}
