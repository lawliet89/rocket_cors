//! This crate tests using rocket_cors using Fairings

#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate hyper;
extern crate rocket;
extern crate rocket_cors;

use std::str::FromStr;

use rocket::http::Method;
use rocket::http::{Header, Status};
use rocket::local::Client;
use rocket_cors::*;

#[get("/")]
fn cors<'a>() -> &'a str {
    "Hello CORS"
}

#[get("/panic")]
fn panicking_route() {
    panic!("This route will panic");
}

fn make_cors_options() -> Cors {
    let (allowed_origins, failed_origins) = AllowedOrigins::some(&["https://www.acme.com"]);
    assert!(failed_origins.is_empty());

    Cors {
        allowed_origins: allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![cors, panicking_route])
        .attach(make_cors_options())
}

#[test]
fn smoke_test() {
    let client = Client::new(rocket()).unwrap();

    // `Options` pre-flight checks
    let origin_header = Header::from(
        hyper::header::Origin::from_str("https://www.acme.com").unwrap(),
    );
    let method_header = Header::from(hyper::header::AccessControlRequestMethod(
        hyper::method::Method::Get,
    ));
    let request_headers = hyper::header::AccessControlRequestHeaders(
        vec![FromStr::from_str("Authorization").unwrap()],
    );
    let request_headers = Header::from(request_headers);
    let req = client
        .options("/")
        .header(origin_header)
        .header(method_header)
        .header(request_headers);

    let response = req.dispatch();
    assert!(response.status().class().is_success());

    // "Actual" request
    let origin_header = Header::from(
        hyper::header::Origin::from_str("https://www.acme.com").unwrap(),
    );
    let authorization = Header::new("Authorization", "let me in");
    let req = client.get("/").header(origin_header).header(authorization);

    let mut response = req.dispatch();
    assert!(response.status().class().is_success());
    let body_str = response.body().and_then(|body| body.into_string());
    assert_eq!(body_str, Some("Hello CORS".to_string()));

    let origin_header = response
        .headers()
        .get_one("Access-Control-Allow-Origin")
        .expect("to exist");
    assert_eq!("https://www.acme.com", origin_header);
}

#[test]
fn cors_options_check() {
    let client = Client::new(rocket()).unwrap();

    let origin_header = Header::from(
        hyper::header::Origin::from_str("https://www.acme.com").unwrap(),
    );
    let method_header = Header::from(hyper::header::AccessControlRequestMethod(
        hyper::method::Method::Get,
    ));
    let request_headers = hyper::header::AccessControlRequestHeaders(
        vec![FromStr::from_str("Authorization").unwrap()],
    );
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

    let origin_header = Header::from(
        hyper::header::Origin::from_str("https://www.acme.com").unwrap(),
    );
    let authorization = Header::new("Authorization", "let me in");
    let req = client.get("/").header(origin_header).header(authorization);

    let mut response = req.dispatch();
    println!("{:?}", response);
    assert!(response.status().class().is_success());
    let body_str = response.body().and_then(|body| body.into_string());
    assert_eq!(body_str, Some("Hello CORS".to_string()));

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
    assert_eq!(body_str, Some("Hello CORS".to_string()));
}

#[test]
fn cors_options_bad_origin() {
    let client = Client::new(rocket()).unwrap();

    let origin_header = Header::from(
        hyper::header::Origin::from_str("https://www.bad-origin.com").unwrap(),
    );
    let method_header = Header::from(hyper::header::AccessControlRequestMethod(
        hyper::method::Method::Get,
    ));
    let request_headers = hyper::header::AccessControlRequestHeaders(
        vec![FromStr::from_str("Authorization").unwrap()],
    );
    let request_headers = Header::from(request_headers);
    let req = client
        .options("/")
        .header(origin_header)
        .header(method_header)
        .header(request_headers);

    let response = req.dispatch();
    assert_eq!(response.status(), Status::Forbidden);
}

/// Unlike the "ad-hoc" mode, this should return 404 because we don't have such a route
#[test]
fn cors_options_missing_origin() {
    let client = Client::new(rocket()).unwrap();

    let method_header = Header::from(hyper::header::AccessControlRequestMethod(
        hyper::method::Method::Get,
    ));
    let request_headers = hyper::header::AccessControlRequestHeaders(
        vec![FromStr::from_str("Authorization").unwrap()],
    );
    let request_headers = Header::from(request_headers);
    let req = client.options("/").header(method_header).header(
        request_headers,
    );

    let response = req.dispatch();
    assert_eq!(response.status(), Status::NotFound);

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

    let origin_header = Header::from(
        hyper::header::Origin::from_str("https://www.acme.com").unwrap(),
    );
    let method_header = Header::from(hyper::header::AccessControlRequestMethod(
        hyper::method::Method::Post,
    ));
    let request_headers = hyper::header::AccessControlRequestHeaders(
        vec![FromStr::from_str("Authorization").unwrap()],
    );
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

    let origin_header = Header::from(
        hyper::header::Origin::from_str("https://www.acme.com").unwrap(),
    );
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

    let origin_header = Header::from(
        hyper::header::Origin::from_str("https://www.bad-origin.com").unwrap(),
    );
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

/// This test ensures that on a failing CORS request, the route (along with its side effects)
/// should never be executed.
/// The route used will panic if executed
#[test]
fn routes_failing_checks_are_not_executed() {
    let client = Client::new(rocket()).unwrap();

    let origin_header = Header::from(
        hyper::header::Origin::from_str("https://www.bad-origin.com").unwrap(),
    );
    let method_header = Header::from(hyper::header::AccessControlRequestMethod(
        hyper::method::Method::Get,
    ));
    let request_headers = hyper::header::AccessControlRequestHeaders(
        vec![FromStr::from_str("Authorization").unwrap()],
    );
    let request_headers = Header::from(request_headers);
    let req = client
        .options("/panic")
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
