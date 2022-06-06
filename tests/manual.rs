//! This crate tests using `rocket_cors` using manual mode
use rocket::http::hyper;
use rocket::http::Method;
use rocket::http::{Header, Status};
use rocket::local::blocking::Client;
use rocket::response::Responder;
use rocket::State;
use rocket::{get, options, routes};
use rocket_cors::*;

static ORIGIN: http::header::HeaderName = hyper::header::ORIGIN;
static ACCESS_CONTROL_REQUEST_METHOD: http::header::HeaderName =
    hyper::header::ACCESS_CONTROL_REQUEST_METHOD;
static ACCESS_CONTROL_REQUEST_HEADERS: http::header::HeaderName =
    hyper::header::ACCESS_CONTROL_REQUEST_HEADERS;

/// Using a borrowed `Cors`
#[get("/")]
fn cors(options: &State<Cors>) -> impl Responder<'_, '_> {
    options
        .inner()
        .respond_borrowed(|guard| guard.responder("Hello CORS"))
}

#[get("/panic")]
fn panicking_route(options: &State<Cors>) -> impl Responder<'_, '_> {
    options.inner().respond_borrowed(|_| {
        panic!("This route will panic");
    })
}

/// Respond with an owned option instead
#[options("/owned")]
fn owned_options<'r, 'o: 'r>() -> impl Responder<'r, 'o> {
    let borrow = make_different_cors_options().to_cors()?;

    borrow.respond_owned(|guard| guard.responder("Manual CORS Preflight"))
}

/// Respond with an owned option instead
#[get("/owned")]
fn owned<'r, 'o: 'r>() -> impl Responder<'r, 'o> {
    let borrow = make_different_cors_options().to_cors()?;

    borrow.respond_owned(|guard| guard.responder("Hello CORS Owned"))
}

// The following routes tests that the routes can be compiled with manual CORS

/// `Responder` with String
#[get("/")]
#[allow(dead_code)]
fn responder_string(options: &State<Cors>) -> impl Responder<'_, '_> {
    options
        .inner()
        .respond_borrowed(|guard| guard.responder("Hello CORS".to_string()))
}

struct TestState;
/// Borrow something else from Rocket with lifetime `'r`
#[get("/")]
#[allow(dead_code)]
fn borrow<'r, 'o: 'r>(
    options: &'r State<Cors>,
    test_state: &'r State<TestState>,
) -> impl Responder<'r, 'o> {
    let borrow = test_state.inner();
    options.inner().respond_borrowed(move |guard| {
        let _ = borrow;
        guard.responder("Hello CORS".to_string())
    })
}

fn make_cors_options() -> CorsOptions {
    let allowed_origins = AllowedOrigins::some_exact(&["https://www.acme.com"]);

    CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }
}

fn make_different_cors_options() -> CorsOptions {
    let allowed_origins = AllowedOrigins::some_exact(&["https://www.example.com"]);

    CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }
}

fn rocket() -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .mount("/", routes![cors, panicking_route])
        .mount("/", routes![owned, owned_options])
        .mount("/", catch_all_options_routes()) // mount the catch all routes
        .manage(make_cors_options().to_cors().expect("Not to fail"))
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
fn cors_options_borrowed_check() {
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
fn cors_get_borrowed_check() {
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
    assert!(response.status().class().is_success());
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

/// Manual OPTIONS routes are called
#[test]
fn cors_options_owned_check() {
    let rocket = rocket();
    let client = Client::tracked(rocket).unwrap();

    let origin_header = Header::new(ORIGIN.as_str(), "https://www.example.com");
    let method_header = Header::new(
        ACCESS_CONTROL_REQUEST_METHOD.as_str(),
        hyper::Method::GET.as_str(),
    );
    let request_headers = Header::new(ACCESS_CONTROL_REQUEST_HEADERS.as_str(), "Authorization");
    let req = client
        .options("/owned")
        .header(origin_header)
        .header(method_header)
        .header(request_headers);

    let response = req.dispatch();
    assert!(response.status().class().is_success());
    let origin_header = response
        .headers()
        .get_one("Access-Control-Allow-Origin")
        .expect("to exist");
    assert_eq!("https://www.example.com", origin_header);

    let body_str = response.into_string();
    assert_eq!(body_str, Some("Manual CORS Preflight".to_string()));
}

/// Owned manual response works
#[test]
fn cors_get_owned_check() {
    let client = Client::tracked(rocket()).unwrap();

    let origin_header = Header::new(ORIGIN.as_str(), "https://www.example.com");
    let authorization = Header::new("Authorization", "let me in");
    let req = client
        .get("/owned")
        .header(origin_header)
        .header(authorization);

    let response = req.dispatch();
    assert!(response.status().class().is_success());
    let origin_header = response
        .headers()
        .get_one("Access-Control-Allow-Origin")
        .expect("to exist");
    assert_eq!("https://www.example.com", origin_header);
    let body_str = response.into_string();
    assert_eq!(body_str, Some("Hello CORS Owned".to_string()));
}
