#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
extern crate rocket_cors;

use std::io::Cursor;

use rocket::http::Method;
use rocket::Response;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Guard, Responder};

/// Using a `Responder` -- the usual way you would use this
#[get("/")]
fn responder(cors: Guard) -> Responder<&str> {
    cors.responder("Hello CORS!")
}

/// Using a `Response` instead of a `Responder`. You generally won't have to do this.
#[get("/response")]
fn response(cors: Guard) -> Response {
    let mut response = Response::new();
    response.set_sized_body(Cursor::new("Hello CORS!"));
    cors.response(response)
}

/// Manually mount an OPTIONS route for your own handling
#[options("/manual")]
fn manual_options(cors: Guard) -> Responder<&str> {
    cors.responder("Manual OPTIONS preflight handling")
}

/// Manually mount an OPTIONS route for your own handling
#[get("/manual")]
fn manual(cors: Guard) -> Responder<&str> {
    cors.responder("Manual OPTIONS preflight handling")
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
        .mount(
            "/",
            routes![responder, response],
        )
        // Mount the routes to catch all the OPTIONS pre-flight requests
        .mount("/", rocket_cors::catch_all_options_routes())
        // You can also manually mount an OPTIONS route that will be used instead
        .mount("/", routes![manual, manual_options])
        .manage(options)
        .launch();
}
