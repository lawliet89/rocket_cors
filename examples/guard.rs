#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate rocket_cors;

use std::io::Cursor;

use rocket::Response;
use rocket::http::Method;
use rocket_cors::{Guard, AllOrSome, Responder};

/// Using a `Responder` -- the usual way you would use this
#[get("/")]
fn responder(cors: Guard) -> Responder<&str> {
    cors.responder("Hello CORS!")
}

/// You need to define an OPTIONS route for preflight checks.
/// These routes can just return the unit type `()`
#[options("/")]
fn responder_options(cors: Guard) -> Responder<()> {
    cors.responder(())
}

/// Using a `Response` instead of a `Responder`. You generally won't have to do this.
#[get("/responder")]
fn response(cors: Guard) -> Response {
    let mut response = Response::new();
    response.set_sized_body(Cursor::new("Hello CORS!"));
    cors.response(response)
}

/// You need to define an OPTIONS route for preflight checks.
/// These routes can just return the unit type `()`
#[options("/responder")]
fn response_options(cors: Guard) -> Response {
    let response = Response::new();
    cors.response(response)
}

fn main() {
    let (allowed_origins, failed_origins) = AllOrSome::new_from_str_list(&["https://www.acme.com"]);
    assert!(failed_origins.is_empty());

    // You can also deserialize this
    let options = rocket_cors::Cors {
        allowed_origins: allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllOrSome::Some(
            ["Authorization", "Accept"]
                .into_iter()
                .map(|s| s.to_string().into())
                .collect(),
        ),
        allow_credentials: true,
        ..Default::default()
    };

    rocket::ignite()
        .mount("/", routes![responder, responder_options, response, response_options])
        .manage(options)
        .launch();
}
