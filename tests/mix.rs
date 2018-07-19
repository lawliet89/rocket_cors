//! This is an example of how you can mix and match the "Truly manual" mode with "Guard".
//!
//! In this example, you typically have an application wide `Cors` struct except for one specific
//! `ping` route that you want to allow all Origins to access.

#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate hyper;
extern crate rocket;
extern crate rocket_cors;

use std::str::FromStr;

use rocket::http::{Header, Method, Status};
use rocket::local::Client;
use rocket::response::Responder;

use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, Guard};

/// The "usual" app route
#[get("/")]
fn app(cors: Guard) -> rocket_cors::Responder<&str> {
    cors.responder("Hello CORS!")
}

/// The special "ping" route
#[get("/ping")]
fn ping<'r>() -> impl Responder<'r> {
    let options = cors_options_all();
    options.respond_owned(|guard| guard.responder("Pong!"))
}

/// You need to define an OPTIONS route for preflight checks if you want to use `Cors` struct
/// that is not in Rocket's managed state.
/// These routes can just return the unit type `()`
#[options("/ping")]
fn ping_options<'r>() -> impl Responder<'r> {
    let options = cors_options_all();
    options.respond_owned(|guard| guard.responder(()))
}

/// Returns the "application wide" Cors struct
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

/// A special struct that allows all origins
///
/// Note: In your real application, you might want to use something like `lazy_static` to generate
/// a `&'static` reference to this instead of creating a new struct on every request.
fn cors_options_all() -> Cors {
    // You can also deserialize this
    Default::default()
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount(
            "/",
            routes![
                app,
                ping,
                ping_options,
            ],
        )
        .mount("/", rocket_cors::catch_all_options_routes()) // mount the catch all routes
        .manage(cors_options())
}

#[test]
fn smoke_test() {
    let client = Client::new(rocket()).unwrap();

    // `Options` pre-flight checks
    let origin_header =
        Header::from(hyper::header::Origin::from_str("https://www.acme.com").unwrap());
    let method_header = Header::from(hyper::header::AccessControlRequestMethod(
        hyper::method::Method::Get,
    ));
    let request_headers = hyper::header::AccessControlRequestHeaders(vec![
        FromStr::from_str("Authorization").unwrap(),
    ]);
    let request_headers = Header::from(request_headers);
    let req = client
        .options("/")
        .header(origin_header)
        .header(method_header)
        .header(request_headers);

    let response = req.dispatch();
    assert!(response.status().class().is_success());

    // "Actual" request
    let origin_header =
        Header::from(hyper::header::Origin::from_str("https://www.acme.com").unwrap());
    let authorization = Header::new("Authorization", "let me in");
    let req = client.get("/").header(origin_header).header(authorization);

    let mut response = req.dispatch();
    assert!(response.status().class().is_success());
    let body_str = response.body().and_then(|body| body.into_string());
    assert_eq!(body_str, Some("Hello CORS!".to_string()));

    let origin_header = response
        .headers()
        .get_one("Access-Control-Allow-Origin")
        .expect("to exist");
    assert_eq!("https://www.acme.com", origin_header);
}

#[test]
fn cors_options_check() {
    let client = Client::new(rocket()).unwrap();

    let origin_header =
        Header::from(hyper::header::Origin::from_str("https://www.acme.com").unwrap());
    let method_header = Header::from(hyper::header::AccessControlRequestMethod(
        hyper::method::Method::Get,
    ));
    let request_headers = hyper::header::AccessControlRequestHeaders(vec![
        FromStr::from_str("Authorization").unwrap(),
    ]);
    let request_headers = Header::from(request_headers);
    let req = client
        .options("/")
        .header(origin_header)
        .header(method_header)
        .header(request_headers);

    let response = req.dispatch();
    assert!(response.status().class().is_success());

    let origin_header = response
        .headers()
        .get_one("Access-Control-Allow-Origin")
        .expect("to exist");
    assert_eq!("https://www.acme.com", origin_header);
}

#[test]
fn cors_get_check() {
    let client = Client::new(rocket()).unwrap();

    let origin_header =
        Header::from(hyper::header::Origin::from_str("https://www.acme.com").unwrap());
    let authorization = Header::new("Authorization", "let me in");
    let req = client.get("/").header(origin_header).header(authorization);

    let mut response = req.dispatch();
    assert!(response.status().class().is_success());
    let body_str = response.body().and_then(|body| body.into_string());
    assert_eq!(body_str, Some("Hello CORS!".to_string()));

    let origin_header = response
        .headers()
        .get_one("Access-Control-Allow-Origin")
        .expect("to exist");
    assert_eq!("https://www.acme.com", origin_header);
}

/// This test is to check that non CORS compliant requests to GET should still work. (i.e. curl)
#[test]
fn cors_get_no_origin() {
    let client = Client::new(rocket()).unwrap();

    let authorization = Header::new("Authorization", "let me in");
    let req = client.get("/").header(authorization);

    let mut response = req.dispatch();
    assert!(response.status().class().is_success());
    let body_str = response.body().and_then(|body| body.into_string());
    assert_eq!(body_str, Some("Hello CORS!".to_string()));
}

#[test]
fn cors_options_bad_origin() {
    let client = Client::new(rocket()).unwrap();

    let origin_header =
        Header::from(hyper::header::Origin::from_str("https://www.bad-origin.com").unwrap());
    let method_header = Header::from(hyper::header::AccessControlRequestMethod(
        hyper::method::Method::Get,
    ));
    let request_headers = hyper::header::AccessControlRequestHeaders(vec![
        FromStr::from_str("Authorization").unwrap(),
    ]);
    let request_headers = Header::from(request_headers);
    let req = client
        .options("/")
        .header(origin_header)
        .header(method_header)
        .header(request_headers);

    let response = req.dispatch();
    assert_eq!(response.status(), Status::Forbidden);
}

#[test]
fn cors_options_missing_origin() {
    let client = Client::new(rocket()).unwrap();

    let method_header = Header::from(hyper::header::AccessControlRequestMethod(
        hyper::method::Method::Get,
    ));
    let request_headers = hyper::header::AccessControlRequestHeaders(vec![
        FromStr::from_str("Authorization").unwrap(),
    ]);
    let request_headers = Header::from(request_headers);
    let req = client
        .options("/")
        .header(method_header)
        .header(request_headers);

    let response = req.dispatch();
    assert!(response.status().class().is_success());
    assert!(
        response
            .headers()
            .get_one("Access-Control-Allow-Origin")
            .is_none()
    );
}

#[test]
fn cors_options_bad_request_method() {
    let client = Client::new(rocket()).unwrap();

    let origin_header =
        Header::from(hyper::header::Origin::from_str("https://www.acme.com").unwrap());
    let method_header = Header::from(hyper::header::AccessControlRequestMethod(
        hyper::method::Method::Post,
    ));
    let request_headers = hyper::header::AccessControlRequestHeaders(vec![
        FromStr::from_str("Authorization").unwrap(),
    ]);
    let request_headers = Header::from(request_headers);
    let req = client
        .options("/")
        .header(origin_header)
        .header(method_header)
        .header(request_headers);

    let response = req.dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    assert!(
        response
            .headers()
            .get_one("Access-Control-Allow-Origin")
            .is_none()
    );
}

#[test]
fn cors_options_bad_request_header() {
    let client = Client::new(rocket()).unwrap();

    let origin_header =
        Header::from(hyper::header::Origin::from_str("https://www.acme.com").unwrap());
    let method_header = Header::from(hyper::header::AccessControlRequestMethod(
        hyper::method::Method::Get,
    ));
    let request_headers =
        hyper::header::AccessControlRequestHeaders(vec![FromStr::from_str("Foobar").unwrap()]);
    let request_headers = Header::from(request_headers);
    let req = client
        .options("/")
        .header(origin_header)
        .header(method_header)
        .header(request_headers);

    let response = req.dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    assert!(
        response
            .headers()
            .get_one("Access-Control-Allow-Origin")
            .is_none()
    );
}

#[test]
fn cors_get_bad_origin() {
    let client = Client::new(rocket()).unwrap();

    let origin_header =
        Header::from(hyper::header::Origin::from_str("https://www.bad-origin.com").unwrap());
    let authorization = Header::new("Authorization", "let me in");
    let req = client.get("/").header(origin_header).header(authorization);

    let response = req.dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    assert!(
        response
            .headers()
            .get_one("Access-Control-Allow-Origin")
            .is_none()
    );
}

/// Tests that the `ping` route accepts other Origins
#[test]
fn cors_options_ping_check() {
    let client = Client::new(rocket()).unwrap();

    let origin_header =
        Header::from(hyper::header::Origin::from_str("https://www.example.com").unwrap());
    let method_header = Header::from(hyper::header::AccessControlRequestMethod(
        hyper::method::Method::Get,
    ));

    let req = client
        .options("/ping")
        .header(origin_header)
        .header(method_header);

    let response = req.dispatch();
    assert!(response.status().class().is_success());

    let origin_header = response
        .headers()
        .get_one("Access-Control-Allow-Origin")
        .expect("to exist");
    assert_eq!("https://www.example.com", origin_header);
}

/// Tests that the `ping` route accepts other Origins
#[test]
fn cors_get_ping_check() {
    let client = Client::new(rocket()).unwrap();

    let origin_header =
        Header::from(hyper::header::Origin::from_str("https://www.example.com").unwrap());

    let req = client.get("/ping").header(origin_header);

    let mut response = req.dispatch();
    assert!(response.status().class().is_success());
    let body_str = response.body().and_then(|body| body.into_string());
    assert_eq!(body_str, Some("Pong!".to_string()));

    let origin_header = response
        .headers()
        .get_one("Access-Control-Allow-Origin")
        .expect("to exist");
    assert_eq!("https://www.example.com", origin_header);
}
