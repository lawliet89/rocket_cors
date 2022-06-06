//! This crate tests using `rocket_cors` using Fairings
use rocket::http::hyper;
use rocket::http::Method;
use rocket::http::{Header, Status};
use rocket::local::blocking::Client;
use rocket::{get, routes};
use rocket_cors::*;

static ORIGIN: http::header::HeaderName = hyper::header::ORIGIN;
static ACCESS_CONTROL_REQUEST_METHOD: http::header::HeaderName =
    hyper::header::ACCESS_CONTROL_REQUEST_METHOD;
static ACCESS_CONTROL_REQUEST_HEADERS: http::header::HeaderName =
    hyper::header::ACCESS_CONTROL_REQUEST_HEADERS;

#[get("/")]
fn cors<'a>() -> &'a str {
    "Hello CORS"
}

#[get("/panic")]
fn panicking_route<'a>() -> &'a str {
    panic!("This route will panic");
}

fn make_cors() -> Cors {
    let allowed_origins = AllowedOrigins::some_exact(&["https://www.acme.com"]);

    CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("To not fail")
}

fn rocket() -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .mount("/", routes![cors, panicking_route])
        .attach(make_cors())
}

#[test]
fn smoke_test() {
    let client = Client::tracked(rocket()).unwrap();

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

#[test]
fn cors_options_check() {
    let client = Client::tracked(rocket()).unwrap();

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

#[test]
fn cors_get_check() {
    let client = Client::tracked(rocket()).unwrap();

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
    let client = Client::tracked(rocket()).unwrap();

    let authorization = Header::new("Authorization", "let me in");
    let req = client.get("/").header(authorization);

    let response = req.dispatch();
    assert!(response.status().class().is_success());
    let body_str = response.into_string();
    assert_eq!(body_str, Some("Hello CORS".to_string()));
}

#[test]
fn cors_options_bad_origin() {
    let client = Client::tracked(rocket()).unwrap();

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
}

/// Unlike the "ad-hoc" mode, this should return 404 because we don't have such a route
#[test]
fn cors_options_missing_origin() {
    let client = Client::tracked(rocket()).unwrap();

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
    assert_eq!(response.status(), Status::NotFound);

    assert!(response
        .headers()
        .get_one("Access-Control-Allow-Origin")
        .is_none());
}

#[test]
fn cors_options_bad_request_method() {
    let client = Client::tracked(rocket()).unwrap();

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
    let client = Client::tracked(rocket()).unwrap();

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
    let client = Client::tracked(rocket()).unwrap();

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
    let client = Client::tracked(rocket()).unwrap();

    let origin_header = Header::new(ORIGIN.as_str(), "https://www.bad-origin.com");
    let method_header = Header::new(
        ACCESS_CONTROL_REQUEST_METHOD.as_str(),
        hyper::Method::GET.as_str(),
    );
    let request_headers = Header::new(ACCESS_CONTROL_REQUEST_HEADERS.as_str(), "Authorization");
    let req = client
        .options("/panic")
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
