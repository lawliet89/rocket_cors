//! This crate tests that all the request headers are parsed correctly in the round trip
#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
extern crate hyper;
extern crate rocket;
extern crate rocket_cors;

use std::ops::Deref;
use std::str::FromStr;

use rocket::local::Client;
use rocket::http::Header;
use rocket_cors::headers::*;

#[get("/request_headers")]
fn request_headers(
    origin: Origin,
    method: AccessControlRequestMethod,
    headers: AccessControlRequestHeaders,
) -> String {
    let AccessControlRequestMethod(method) = method;
    let AccessControlRequestHeaders(headers) = headers;
    let mut headers = headers
        .iter()
        .map(|s| s.deref().to_string())
        .collect::<Vec<String>>();
    headers.sort();
    format!("{}\n{}\n{}", origin, method, headers.join(", "))
}

/// Tests that all the request headers are parsed correcly in a HTTP request
#[test]
fn request_headers_round_trip_smoke_test() {
    let rocket = rocket::ignite().mount("/", routes![request_headers]);
    let client = Client::new(rocket).expect("A valid Rocket client");

    let origin_header = Header::from(
        hyper::header::Origin::from_str("https://foo.bar.xyz").unwrap(),
    );
    let method_header = Header::from(hyper::header::AccessControlRequestMethod(
        hyper::method::Method::Get,
    ));
    let request_headers = hyper::header::AccessControlRequestHeaders(vec![
        FromStr::from_str("accept-language").unwrap(),
        FromStr::from_str("X-Ping").unwrap(),
    ]);
    let request_headers = Header::from(request_headers);
    let req = client
        .get("/request_headers")
        .header(origin_header)
        .header(method_header)
        .header(request_headers);
    let mut response = req.dispatch();

    assert!(response.status().class().is_success());
    let body_str = response.body().and_then(|body| body.into_string()).expect(
        "Non-empty body",
    );
    let expected_body = r#"https://foo.bar.xyz/
GET
X-Ping, accept-language"#;
    assert_eq!(expected_body, body_str);
}
