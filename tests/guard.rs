//! This crate tests using `rocket_cors` using the per-route handling with request guard

#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate hyper;
extern crate rocket;
extern crate rocket_cors as cors;

use std::str::FromStr;

use rocket::{Response, State};
use rocket::http::Method;
use rocket::http::{Header, Status};
use rocket::local::Client;

#[get("/")]
fn cors(cors: cors::Guard) -> cors::Responder<&str> {
    cors.responder("Hello CORS")
}

#[get("/panic")]
fn panicking_route(_cors: cors::Guard) {
    panic!("This route will panic");
}

/// Manually specify our own OPTIONS route
#[options("/manual")]
fn cors_manual_options(cors: cors::Guard) -> cors::Responder<&str> {
    cors.responder("Manual CORS Preflight")
}

/// Manually specify our own OPTIONS route
#[get("/manual")]
fn cors_manual(cors: cors::Guard) -> cors::Responder<&str> {
    cors.responder("Hello CORS")
}

/// Using a `Response` instead of a `Responder`
#[get("/response")]
fn response(cors: cors::Guard) -> Response {
    cors.response(Response::new())
}

/// `Responder` with String
#[get("/responder/string")]
fn responder_string(cors: cors::Guard) -> cors::Responder<String> {
    cors.responder("Hello CORS".to_string())
}

/// `Responder` with 'static ()
#[get("/responder/unit")]
fn responder_unit(cors: cors::Guard) -> cors::Responder<()> {
    cors.responder(())
}

struct SomeState;
/// Borrow `SomeState` from Rocket
#[get("/state")]
fn state<'r>(cors: cors::Guard<'r>, _state: State<'r, SomeState>) -> cors::Responder<'r, &'r str> {
    cors.responder("hmm")
}

fn make_cors_options() -> cors::Cors {
    let (allowed_origins, failed_origins) = cors::AllowedOrigins::some(&["https://www.acme.com"]);
    assert!(failed_origins.is_empty());

    cors::Cors {
        allowed_origins: allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: cors::AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }
}

fn make_rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![cors, panicking_route])
        .mount("/", routes![response, responder_string, responder_unit, state])
        .mount("/", cors::catch_all_options_routes()) // mount the catch all routes
        .mount("/", routes![cors_manual, cors_manual_options]) // manual OPTIOONS routes
        .manage(make_cors_options())
        .manage(SomeState)
}

#[test]
fn smoke_test() {
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

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

/// Check the "catch all" OPTIONS route works for `/`
#[test]
fn cors_options_catch_all_check() {
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

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


/// Check the "catch all" OPTIONS route works for other routes
#[test]
fn cors_options_catch_all_check_other_routes() {
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

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
        .options("/response/unit")
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
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

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

/// This test is to check that non CORS compliant requests to GET should still work. (i.e. curl)
#[test]
fn cors_get_no_origin() {
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

    let authorization = Header::new("Authorization", "let me in");
    let req = client.get("/").header(authorization);

    let mut response = req.dispatch();
    assert!(response.status().class().is_success());
    let body_str = response.body().and_then(|body| body.into_string());
    assert_eq!(body_str, Some("Hello CORS".to_string()));
    assert!(
        response
            .headers()
            .get_one("Access-Control-Allow-Origin")
            .is_none()
    );
}

#[test]
fn cors_options_bad_origin() {
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

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
    assert!(
        response
            .headers()
            .get_one("Access-Control-Allow-Origin")
            .is_none()
    );
}

#[test]
fn cors_options_missing_origin() {
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

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
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

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
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

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
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

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
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

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

/// This test ensures that manually mounted CORS OPTIONS routes are used even in the presence of
/// a "catch all" route.
#[test]
fn overridden_options_routes_are_used() {
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

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
        .options("/manual")
        .header(origin_header)
        .header(method_header)
        .header(request_headers);

    let mut response = req.dispatch();
    let body_str = response.body().and_then(|body| body.into_string());
    assert!(response.status().class().is_success());
    assert_eq!(body_str, Some("Manual CORS Preflight".to_string()));

    let origin_header = response
        .headers()
        .get_one("Access-Control-Allow-Origin")
        .expect("to exist");
    assert_eq!("https://www.acme.com", origin_header);
}
