#![feature(plugin, conservative_impl_trait)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate rocket_cors;

use std::io::Cursor;

use rocket::{State, Response};
use rocket::http::Method;
use rocket::response::Responder;
use rocket_cors::{Cors, AllowedOrigins, AllowedHeaders};

/// Using a borrowed Cors
#[get("/")]
fn borrowed<'r>(options: State<'r, Cors>) -> impl Responder<'r> {
    options.inner().respond_borrowed(
        |guard| guard.responder("Hello CORS"),
    )
}

/// You need to define an OPTIONS route for preflight checks.
/// These routes can just return the unit type `()`
#[options("/")]
fn borrowed_options<'r>(options: State<'r, Cors>) -> impl Responder<'r> {
    options.inner().respond_borrowed(
        |guard| guard.responder(()),
    )
}

/// Using a `Response` instead of a `Responder`. You generally won't have to do this.
#[get("/response")]
fn response<'r>(options: State<'r, Cors>) -> impl Responder<'r> {
    let mut response = Response::new();
    response.set_sized_body(Cursor::new("Hello CORS!"));

    options.inner().respond_borrowed(
        move |guard| guard.response(response),
    )
}

/// You need to define an OPTIONS route for preflight checks.
/// These routes can just return the unit type `()`
#[options("/response")]
fn response_options<'r>(options: State<'r, Cors>) -> impl Responder<'r> {
    options.inner().respond_borrowed(
        move |guard| guard.response(Response::new()),
    )
}

/// Create and use an ad-hoc Cors
#[get("/owned")]
fn owned<'r>() -> impl Responder<'r> {
    let options = cors_options();
    options.respond_owned(|guard| guard.responder("Hello CORS"))
}

/// You need to define an OPTIONS route for preflight checks.
/// These routes can just return the unit type `()`
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
        .mount(
            "/",
            routes![
                borrowed,
                borrowed_options,
                response,
                response_options,
                owned,
                owned_options,
            ],
        )
        .manage(cors_options())
        .launch();
}
