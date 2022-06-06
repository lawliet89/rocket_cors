//! This is an example of how you can mix and match the "Truly manual" mode with "Guard".
//!
//! In this example, you typically have an application wide `Cors` struct except for one specific
//! `ping` route that you want to allow all Origins to access.

use rocket::error::Error;
use rocket::http::Method;
use rocket::response::Responder;
use rocket::{get, options, routes};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions, Guard};

/// The "usual" app route
#[get("/")]
fn app(cors: Guard<'_>) -> rocket_cors::Responder<&str> {
    cors.responder("Hello CORS!")
}

/// The special "ping" route
#[get("/ping")]
fn ping<'r, 'o: 'r>() -> impl Responder<'r, 'o> {
    let cors = cors_options_all().to_cors()?;
    cors.respond_owned(|guard| guard.responder("Pong!"))
}

/// You need to define an OPTIONS route for preflight checks if you want to use `Cors` struct
/// that is not in Rocket's managed state.
/// These routes can just return the unit type `()`
#[options("/ping")]
fn ping_options<'r, 'o: 'r>() -> impl Responder<'r, 'o> {
    let cors = cors_options_all().to_cors()?;
    cors.respond_owned(|guard| guard.responder(()))
}

/// Returns the "application wide" Cors struct
fn cors_options() -> CorsOptions {
    let allowed_origins = AllowedOrigins::some_exact(&["https://www.acme.com"]);

    // You can also deserialize this
    rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }
}

/// A special struct that allows all origins
///
/// Note: In your real application, you might want to use something like `lazy_static` to generate
/// a `&'static` reference to this instead of creating a new struct on every request.
fn cors_options_all() -> CorsOptions {
    // You can also deserialize this
    Default::default()
}

#[rocket::main]
async fn main() -> Result<(), Error> {
    let _ = rocket::build()
        .mount("/", routes![app, ping, ping_options,])
        .mount("/", rocket_cors::catch_all_options_routes()) // mount the catch all routes
        .manage(cors_options().to_cors().expect("To not fail"))
        .launch()
        .await?;

    Ok(())
}
