#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
extern crate rocket_cors;

use std::io::Cursor;

use rocket::http::Method;
use rocket::response::Responder;
use rocket::{Response, State};
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors};

/// Using a borrowed Cors
/// Note that the `'r` lifetime annotation is not requred here because `State` borrows with lifetime
/// `'r` and so does `Responder`!
#[get("/")]
fn borrowed(options: State<Cors>) -> impl Responder {
    options
        .inner()
        .respond_borrowed(|guard| guard.responder("Hello CORS"))
}

/// Using a `Response` instead of a `Responder`. You generally won't have to do this.
/// Note that the `'r` lifetime annotation is not requred here because `State` borrows with lifetime
/// `'r` and so does `Responder`!
#[get("/response")]
fn response(options: State<Cors>) -> impl Responder {
    let mut response = Response::new();
    response.set_sized_body(Cursor::new("Hello CORS!"));

    options
        .inner()
        .respond_borrowed(move |guard| guard.response(response))
}

/// Create and use an ad-hoc Cors
/// Note that the `'r` lifetime is needed because the compiler cannot elide anything.
#[get("/owned")]
fn owned<'r>() -> impl Responder<'r> {
    let options = cors_options();
    options.respond_owned(|guard| guard.responder("Hello CORS"))
}

/// You need to define an OPTIONS route for preflight checks if you want to use `Cors` struct
/// that is not in Rocket's managed state.
/// These routes can just return the unit type `()`
/// Note that the `'r` lifetime is needed because the compiler cannot elide anything.
#[options("/owned")]
fn owned_options<'r>() -> impl Responder<'r> {
    let options = cors_options();
    options.respond_owned(|guard| guard.responder(()))
}

fn cors_options() -> Cors {
    let (allowed_origins, failed_origins) = AllowedOrigins::some(&["https://www.acme.com"]);
    assert!(failed_origins.is_empty());

    // You can also deserialize this
    rocket_cors::Cors {
        allowed_origins: allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }
}

fn main() {
    rocket::ignite()
        .mount("/", routes![borrowed, response, owned, owned_options,])
        .mount("/", rocket_cors::catch_all_options_routes()) // mount the catch all routes
        .manage(cors_options())
        .launch();
}
