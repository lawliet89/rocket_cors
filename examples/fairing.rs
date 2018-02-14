#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate rocket_cors;

use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins};

#[get("/")]
fn cors<'a>() -> &'a str {
    "Hello CORS"
}

fn main() {
    let (allowed_origins, failed_origins) = AllowedOrigins::some(&["https://www.acme.com"]);
    assert!(failed_origins.is_empty());

    // You can also deserialize this
    let options = rocket_cors::Cors {
        allowed_origins: allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    };

    rocket::ignite()
        .mount("/", routes![cors])
        .attach(options)
        .launch();
}
