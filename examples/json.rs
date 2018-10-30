//! This example is to demonstrate the JSON serialization and deserialization of the Cors settings
//!
//! Note: This requires the `serialization` feature which is enabled by default.
#![feature(proc_macro_hygiene, decl_macro)]
extern crate rocket;
extern crate rocket_cors as cors;
extern crate serde_json;

use cors::{AllowedHeaders, AllowedOrigins, Cors};
use rocket::http::Method;

fn main() {
    // The default demonstrates the "All" serialization of several of the settings
    let default: Cors = Default::default();

    let (allowed_origins, failed_origins) = AllowedOrigins::some(&["https://www.acme.com"]);
    assert!(failed_origins.is_empty());

    let options = cors::Cors {
        allowed_origins: allowed_origins,
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
