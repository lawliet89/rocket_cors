//! This crate tests using rocket_cors using Fairings

#![feature(plugin, custom_derive)]
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

fn make_cors_options() -> Cors {
    let (allowed_origins, failed_origins) = AllOrSome::new_from_str_list(&["https://www.acme.com"]);
    assert!(failed_origins.is_empty());

    Cors {
        allowed_origins: allowed_origins,
        allowed_methods: [Method::Get].iter().cloned().collect(),
        allowed_headers: AllOrSome::Some(
            ["Authorization"]
                .into_iter()
                .map(|s| s.to_string().into())
                .collect(),
        ),
        allow_credentials: true,
        ..Default::default()
    }
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![cors])
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
    assert_eq!(response.status(), Status::Ok);

    // "Actual" request
    let origin_header = Header::from(
        hyper::header::Origin::from_str("https://www.acme.com").unwrap(),
    );
    let authorization = Header::new("Authorization", "let me in");
    let req = client.get("/").header(origin_header).header(authorization);

    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|body| body.into_string());
    assert_eq!(body_str, Some("Hello CORS".to_string()));

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
    assert_eq!(response.status(), Status::Ok);
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
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|body| body.into_string());
    assert_eq!(body_str, Some("Hello CORS".to_string()));
}

/// This test is to check that non CORS compliant requests to GET should still work. (i.e. curl)
#[test]
fn cors_get_no_origin() {
    let client = Client::new(rocket()).unwrap();

    let authorization = Header::new("Authorization", "let me in");
    let req = client.get("/").header(authorization);

    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
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
    assert_eq!(response.status(), Status::Ok);
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
}
