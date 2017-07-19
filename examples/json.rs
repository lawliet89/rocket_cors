//! This example is to demonstrate the JSON serialization and deserialization of the Cors settings
extern crate rocket;
extern crate rocket_cors as cors;
extern crate serde_json;

use rocket::http::Method;
use cors::Cors;

fn main() {
    // The default demonstrates the "All" serialization of several of the settings
    let default: Cors = Default::default();

    let (allowed_origins, failed_origins) = cors::AllowedOrigins::some(&["https://www.acme.com"]);
    assert!(failed_origins.is_empty());

    let options = cors::Cors {
        allowed_origins: allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post, Method::Delete]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: cors::AllOrSome::Some(
            ["Authorization", "Accept"]
                .into_iter()
                .map(|s| s.to_string().into())
                .collect(),
        ),
        allow_credentials: true,
        expose_headers: ["Content-Type", "X-Custom"].iter().map(ToString::to_string).collect(),
        max_age: Some(42),
        send_wildcard: false,
        fairing_route_base: "/mycors".to_string(),
    };

    println!("Default settings");
    println!("{}", serde_json::to_string_pretty(&default).unwrap());

    println!("Defined settings");
    println!("{}", serde_json::to_string_pretty(&options).unwrap());
}
