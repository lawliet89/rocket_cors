//! This crate tests using `rocket_cors` using the per-route handling with request guard
#![feature(proc_macro_hygiene, decl_macro)]
use rocket_cors as cors;

use rocket::http::hyper;
use rocket::http::Method;
use rocket::http::{Header, Status};
use rocket::local::blocking::Client;
use rocket::{get, options, routes};
use rocket::{Response, State};

static ORIGIN: hyper::HeaderName = hyper::header::ORIGIN;
static ACCESS_CONTROL_REQUEST_METHOD: hyper::HeaderName =
    hyper::header::ACCESS_CONTROL_REQUEST_METHOD;
static ACCESS_CONTROL_REQUEST_HEADERS: hyper::HeaderName =
    hyper::header::ACCESS_CONTROL_REQUEST_HEADERS;

#[get("/")]
fn cors(cors: cors::Guard<'_>) -> cors::Responder<'_, '_, &str> {
    cors.responder("Hello CORS")
}

#[get("/panic")]
fn panicking_route(_cors: cors::Guard<'_>) -> cors::Responder<'_, '_, &str> {
    panic!("This route will panic");
}

/// Manually specify our own OPTIONS route
#[options("/manual")]
fn cors_manual_options(cors: cors::Guard<'_>) -> cors::Responder<'_, '_, &str> {
    cors.responder("Manual CORS Preflight")
}

/// Manually specify our own OPTIONS route
#[get("/manual")]
fn cors_manual(cors: cors::Guard<'_>) -> cors::Responder<'_, '_, &str> {
    cors.responder("Hello CORS")
}

/// Using a `Response` instead of a `Responder`
#[get("/response")]
fn response(cors: cors::Guard<'_>) -> Response<'_> {
    cors.response(Response::new())
}

/// `Responder` with String
#[get("/responder/string")]
fn responder_string(cors: cors::Guard<'_>) -> cors::Responder<'_, 'static, String> {
    cors.responder("Hello CORS".to_string())
}

/// `Responder` with 'static ()
#[get("/responder/unit")]
fn responder_unit(cors: cors::Guard<'_>) -> cors::Responder<'_, 'static, ()> {
    cors.responder(())
}

struct SomeState;
/// Borrow `SomeState` from Rocket
#[get("/state")]
fn state<'r, 'o: 'r>(
    cors: cors::Guard<'r>,
    _state: State<'r, SomeState>,
) -> cors::Responder<'r, 'o, &'r str> {
    cors.responder("hmm")
}

fn make_cors() -> cors::Cors {
    let allowed_origins = cors::AllowedOrigins::some_exact(&["https://www.acme.com"]);

    cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: cors::AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("To not fail")
}

fn make_rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![cors, panicking_route])
        .mount(
            "/",
            routes![response, responder_string, responder_unit, state],
        )
        .mount("/", cors::catch_all_options_routes()) // mount the catch all routes
        .mount("/", routes![cors_manual, cors_manual_options]) // manual OPTIOONS routes
        .manage(make_cors())
        .manage(SomeState)
}

#[test]
fn smoke_test() {
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

    // `Options` pre-flight checks
    let origin_header = Header::new(ORIGIN.as_str(), "https://www.acme.com");
    let method_header = Header::new(
        ACCESS_CONTROL_REQUEST_METHOD.as_str(),
        hyper::Method::GET.as_str(),
    );
    let request_headers = Header::new(ACCESS_CONTROL_REQUEST_HEADERS.as_str(), "Authorization");
    let req = client
        .options("/")
        .header(origin_header)
        .header(method_header)
        .header(request_headers);

    let response = req.dispatch();
    assert!(response.status().class().is_success());

    // "Actual" request
    let origin_header = Header::new(ORIGIN.as_str(), "https://www.acme.com");
    let authorization = Header::new("Authorization", "let me in");
    let req = client.get("/").header(origin_header).header(authorization);

    let response = req.dispatch();
    assert!(response.status().class().is_success());
    let origin_header = response
        .headers()
        .get_one("Access-Control-Allow-Origin")
        .expect("to exist");
    assert_eq!("https://www.acme.com", origin_header);
    let body_str = response.into_string();
    assert_eq!(body_str, Some("Hello CORS".to_string()));
}

/// Check the "catch all" OPTIONS route works for `/`
#[test]
fn cors_options_catch_all_check() {
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

    let origin_header = Header::new(ORIGIN.as_str(), "https://www.acme.com");
    let method_header = Header::new(
        ACCESS_CONTROL_REQUEST_METHOD.as_str(),
        hyper::Method::GET.as_str(),
    );
    let request_headers = Header::new(ACCESS_CONTROL_REQUEST_HEADERS.as_str(), "Authorization");
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

    let origin_header = Header::new(ORIGIN.as_str(), "https://www.acme.com");
    let method_header = Header::new(
        ACCESS_CONTROL_REQUEST_METHOD.as_str(),
        hyper::Method::GET.as_str(),
    );
    let request_headers = Header::new(ACCESS_CONTROL_REQUEST_HEADERS.as_str(), "Authorization");
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

    let origin_header = Header::new(ORIGIN.as_str(), "https://www.acme.com");
    let authorization = Header::new("Authorization", "let me in");
    let req = client.get("/").header(origin_header).header(authorization);

    let response = req.dispatch();
    assert!(response.status().class().is_success());
    let origin_header = response
        .headers()
        .get_one("Access-Control-Allow-Origin")
        .expect("to exist");
    assert_eq!("https://www.acme.com", origin_header);
    let body_str = response.into_string();
    assert_eq!(body_str, Some("Hello CORS".to_string()));
}

/// This test is to check that non CORS compliant requests to GET should still work. (i.e. curl)
#[test]
fn cors_get_no_origin() {
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

    let authorization = Header::new("Authorization", "let me in");
    let req = client.get("/").header(authorization);

    let response = req.dispatch();
    assert!(response.status().class().is_success());
    assert!(response
        .headers()
        .get_one("Access-Control-Allow-Origin")
        .is_none());
    let body_str = response.into_string();
    assert_eq!(body_str, Some("Hello CORS".to_string()));
}

#[test]
fn cors_options_bad_origin() {
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

    let origin_header = Header::new(ORIGIN.as_str(), "https://www.bad-origin.com");
    let method_header = Header::new(
        ACCESS_CONTROL_REQUEST_METHOD.as_str(),
        hyper::Method::GET.as_str(),
    );
    let request_headers = Header::new(ACCESS_CONTROL_REQUEST_HEADERS.as_str(), "Authorization");
    let req = client
        .options("/")
        .header(origin_header)
        .header(method_header)
        .header(request_headers);

    let response = req.dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    assert!(response
        .headers()
        .get_one("Access-Control-Allow-Origin")
        .is_none());
}

#[test]
fn cors_options_missing_origin() {
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

    let method_header = Header::new(
        ACCESS_CONTROL_REQUEST_METHOD.as_str(),
        hyper::Method::GET.as_str(),
    );
    let request_headers = Header::new(ACCESS_CONTROL_REQUEST_HEADERS.as_str(), "Authorization");
    let req = client
        .options("/")
        .header(method_header)
        .header(request_headers);

    let response = req.dispatch();
    assert!(response.status().class().is_success());
    assert!(response
        .headers()
        .get_one("Access-Control-Allow-Origin")
        .is_none());
}

#[test]
fn cors_options_bad_request_method() {
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

    let origin_header = Header::new(ORIGIN.as_str(), "https://www.acme.com");
    let method_header = Header::new(
        ACCESS_CONTROL_REQUEST_METHOD.as_str(),
        hyper::Method::POST.as_str(),
    );
    let request_headers = Header::new(ACCESS_CONTROL_REQUEST_HEADERS.as_str(), "Authorization");
    let req = client
        .options("/")
        .header(origin_header)
        .header(method_header)
        .header(request_headers);

    let response = req.dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    assert!(response
        .headers()
        .get_one("Access-Control-Allow-Origin")
        .is_none());
}

#[test]
fn cors_options_bad_request_header() {
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

    let origin_header = Header::new(ORIGIN.as_str(), "https://www.acme.com");
    let method_header = Header::new(
        ACCESS_CONTROL_REQUEST_METHOD.as_str(),
        hyper::Method::GET.as_str(),
    );
    let request_headers = Header::new(ACCESS_CONTROL_REQUEST_HEADERS.as_str(), "Foobar");
    let req = client
        .options("/")
        .header(origin_header)
        .header(method_header)
        .header(request_headers);

    let response = req.dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    assert!(response
        .headers()
        .get_one("Access-Control-Allow-Origin")
        .is_none());
}

#[test]
fn cors_get_bad_origin() {
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

    let origin_header = Header::new(ORIGIN.as_str(), "https://www.bad-origin.com");
    let authorization = Header::new("Authorization", "let me in");
    let req = client.get("/").header(origin_header).header(authorization);

    let response = req.dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    assert!(response
        .headers()
        .get_one("Access-Control-Allow-Origin")
        .is_none());
}

/// This test ensures that on a failing CORS request, the route (along with its side effects)
/// should never be executed.
/// The route used will panic if executed
#[test]
fn routes_failing_checks_are_not_executed() {
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

    let origin_header = Header::new(ORIGIN.as_str(), "https://www.bad-origin.com");
    let authorization = Header::new("Authorization", "let me in");
    let req = client.get("/").header(origin_header).header(authorization);

    let response = req.dispatch();
    assert_eq!(response.status(), Status::Forbidden);
    assert!(response
        .headers()
        .get_one("Access-Control-Allow-Origin")
        .is_none());
}

/// This test ensures that manually mounted CORS OPTIONS routes are used even in the presence of
/// a "catch all" route.
#[test]
fn overridden_options_routes_are_used() {
    let rocket = make_rocket();
    let client = Client::new(rocket).unwrap();

    let origin_header = Header::new(ORIGIN.as_str(), "https://www.acme.com");
    let method_header = Header::new(
        ACCESS_CONTROL_REQUEST_METHOD.as_str(),
        hyper::Method::GET.as_str(),
    );
    let request_headers = Header::new(ACCESS_CONTROL_REQUEST_HEADERS.as_str(), "Authorization");
    let req = client
        .options("/manual")
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
    let body_str = response.into_string();
    assert_eq!(body_str, Some("Manual CORS Preflight".to_string()));
}
