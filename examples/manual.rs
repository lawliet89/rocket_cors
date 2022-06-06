use rocket::error::Error;
use rocket::http::Method;
use rocket::response::Responder;
use rocket::{get, options, routes, State};
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};

/// Using a borrowed Cors
///
/// You might want to borrow the `Cors` struct from Rocket's state, for example. Unless you have
/// special handling, you might want to use the Guard method instead which has less hassle.
///
/// Note that the `'r` lifetime annotation is not requred here because `State` borrows with lifetime
/// `'r` and so does `Responder`!
#[get("/")]
fn borrowed(options: &State<Cors>) -> impl Responder<'_, '_> {
    options
        .inner()
        .respond_borrowed(|guard| guard.responder("Hello CORS"))
}

/// Create and use an ad-hoc Cors
/// Note that the `'r` lifetime is needed because the compiler cannot elide anything.
///
/// This is the most likely scenario when you want to have manual CORS validation. You can use this
/// when the settings you want to use for a route is not the same as the rest of the application
/// (which you might have put in Rocket's state).
#[get("/owned")]
fn owned<'r, 'o: 'r>() -> impl Responder<'r, 'o> {
    let options = cors_options().to_cors()?;
    options.respond_owned(|guard| guard.responder("Hello CORS"))
}

/// You need to define an OPTIONS route for preflight checks if you want to use `Cors` struct
/// that is not in Rocket's managed state.
/// These routes can just return the unit type `()`
/// Note that the `'r` lifetime is needed because the compiler cannot elide anything.
#[options("/owned")]
fn owned_options<'r, 'o: 'r>() -> impl Responder<'r, 'o> {
    let options = cors_options().to_cors()?;
    options.respond_owned(|guard| guard.responder(()))
}

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

#[rocket::main]
async fn main() -> Result<(), Error> {
    let _ = rocket::build()
        .mount("/", routes![borrowed, owned, owned_options,])
        .mount("/", rocket_cors::catch_all_options_routes()) // mount the catch all routes
        .manage(cors_options().to_cors().expect("To not fail"))
        .ignite()
        .await?;

    Ok(())
}
