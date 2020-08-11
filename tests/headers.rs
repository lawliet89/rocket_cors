//! This crate tests that all the request headers are parsed correctly in the round trip
#![feature(proc_macro_hygiene, decl_macro)]
use std::ops::Deref;

use rocket::http::hyper;
use rocket::http::Header;
use rocket::local::blocking::Client;
use rocket::{get, routes};
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

    let origin_header = Header::new(hyper::header::ORIGIN.as_str(), "https://foo.bar.xyz");
    let method_header = Header::new(
        hyper::header::ACCESS_CONTROL_REQUEST_METHOD.as_str(),
        hyper::Method::GET.as_str(),
    );
    let request_headers = Header::new(
        hyper::header::ACCESS_CONTROL_REQUEST_HEADERS.as_str(),
        "accept-language, X-Ping",
    );
    let req = client
        .get("/request_headers")
        .header(origin_header)
        .header(method_header)
        .header(request_headers);
    let response = req.dispatch();

    assert!(response.status().class().is_success());
    let body_str = response.into_string();
    let expected_body = r#"https://foo.bar.xyz
GET
X-Ping, accept-language"#
        .to_string();
    assert_eq!(body_str, Some(expected_body));
}
