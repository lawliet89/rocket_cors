//! This crate tests using rocket_cors using the per-route handling with request guard

#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
extern crate hyper;
extern crate rocket;
extern crate rocket_cors as cors;

use std::str::FromStr;

use rocket::{Response, State};
use rocket::http::Method;
use rocket::http::{Header, Status};
use rocket::local::Client;

#[options("/")]
fn cors_options(cors: cors::Guard) -> cors::Responder<&str> {
    cors.responder("")
}

#[get("/")]
fn cors(cors: cors::Guard) -> cors::Responder<&str> {
    cors.responder("Hello CORS")
}

#[get("/panic")]
fn panicking_route(_cors: cors::Guard) {
    panic!("This route will panic");
}

// The following routes tests that the routes can be compiled with ad-hoc CORS Response/Responders

/// Using a `Response` instead of a `Responder`
#[allow(unmounted_route)]
#[get("/")]
fn response(cors: cors::Guard) -> Response {
    cors.response(Response::new())
}

/// `Responder` with String
#[allow(unmounted_route)]
#[get("/")]
fn responder_string(cors: cors::Guard) -> cors::Responder<String> {
    cors.responder("Hello CORS".to_string())
}

/// `Responder` with 'static ()
#[allow(unmounted_route)]
#[get("/")]
fn responder_unit(cors: cors::Guard) -> cors::Responder<()> {
    cors.responder(())
}

struct SomeState;
/// Borrow `SomeState` from Rocket
#[allow(unmounted_route)]
#[get("/")]
fn state<'r>(cors: cors::Guard<'r>, _state: State<'r, SomeState>) -> cors::Responder<'r, &'r str> {
    cors.responder("hmm")
}

fn make_cors_options() -> cors::Cors {
    let (allowed_origins, failed_origins) =
        cors::AllOrSome::new_from_str_list(&["https://www.acme.com"]);
    assert!(failed_origins.is_empty());

    cors::Cors {
        allowed_origins: allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: cors::AllOrSome::Some(
            ["Authorization", "Accept"]
                .into_iter()
                .map(|s| s.to_string().into())
                .collect(),
        ),
        allow_credentials: true,
        ..Default::default()
    }
}

fn make_rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![cors, cors_options, panicking_route])
        .manage(make_cors_options())
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
    assert_eq!("https://www.acme.com/", origin_header);
}

#[test]
fn cors_options_check() {
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
    assert_eq!("https://www.acme.com/", origin_header);
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
    println!("{:?}", response);
    assert!(response.status().class().is_success());
    let body_str = response.body().and_then(|body| body.into_string());
    assert_eq!(body_str, Some("Hello CORS".to_string()));

    let origin_header = response
        .headers()
        .get_one("Access-Control-Allow-Origin")
        .expect("to exist");
    assert_eq!("https://www.acme.com/", origin_header);
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
