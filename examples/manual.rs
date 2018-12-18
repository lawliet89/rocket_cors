#![feature(proc_macro_hygiene, decl_macro)]
use rocket;
use rocket_cors;

use std::io::Cursor;

use rocket::http::Method;
use rocket::response::Responder;
use rocket::{get, options, routes, Response, State};
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};

/// Using a borrowed Cors
///
/// You might want to borrow the `Cors` struct from Rocket's state, for example. Unless you have
/// special handling, you might want to use the Guard method instead which has less hassle.
///
/// Note that the `'r` lifetime annotation is not requred here because `State` borrows with lifetime
/// `'r` and so does `Responder`!
#[get("/")]
fn borrowed(options: State<'_, Cors>) -> impl Responder<'_> {
    options
        .inner()
        .respond_borrowed(|guard| guard.responder("Hello CORS"))
}

/// Using a `Response` instead of a `Responder`. You generally won't have to do this.
/// Note that the `'r` lifetime annotation is not requred here because `State` borrows with lifetime
/// `'r` and so does `Responder`!
#[get("/response")]
fn response(options: State<'_, Cors>) -> impl Responder<'_> {
    let mut response = Response::new();
    response.set_sized_body(Cursor::new("Hello CORS!"));

    options
        .inner()
        .respond_borrowed(move |guard| guard.response(response))
}

/// Create and use an ad-hoc Cors
/// Note that the `'r` lifetime is needed because the compiler cannot elide anything.
///
/// This is the most likely scenario when you want to have manual CORS validation. You can use this
/// when the settings you want to use for a route is not the same as the rest of the application
/// (which you might have put in Rocket's state).
#[get("/owned")]
fn owned<'r>() -> impl Responder<'r> {
    let options = cors_options().to_cors()?;
    options.respond_owned(|guard| guard.responder("Hello CORS"))
}

/// You need to define an OPTIONS route for preflight checks if you want to use `Cors` struct
/// that is not in Rocket's managed state.
/// These routes can just return the unit type `()`
/// Note that the `'r` lifetime is needed because the compiler cannot elide anything.
#[options("/owned")]
fn owned_options<'r>() -> impl Responder<'r> {
    let options = cors_options().to_cors()?;
    options.respond_owned(|guard| guard.responder(()))
}

fn cors_options() -> CorsOptions {
    let (allowed_origins, failed_origins) = AllowedOrigins::some(&["https://www.acme.com"]);
    assert!(failed_origins.is_empty());

    // You can also deserialize this
    rocket_cors::CorsOptions {
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
        .manage(cors_options().to_cors().expect("To not fail"))
        .launch();
}
