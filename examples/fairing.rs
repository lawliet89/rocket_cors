#![feature(proc_macro_hygiene, decl_macro)]
use rocket;
use rocket_cors;

use rocket::http::Method;
use rocket::{get, routes};
use rocket_cors::{AllowedHeaders, AllowedOrigins, Error};

#[get("/")]
fn cors<'a>() -> &'a str {
    "Hello CORS"
}

fn main() -> Result<(), Error> {
    let (allowed_origins, failed_origins) = AllowedOrigins::some(&["https://www.acme.com"]);
    assert!(failed_origins.is_empty());

    // You can also deserialize this
    let cors = rocket_cors::CorsOptions {
        allowed_origins: allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()?;

    rocket::ignite()
        .mount("/", routes![cors])
        .attach(cors)
        .launch();

    Ok(())
}
