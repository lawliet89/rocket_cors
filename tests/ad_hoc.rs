//! This crate tests using rocket_cors using the "classic" ad-hoc per-route handling

#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
extern crate hyper;
extern crate rocket;
extern crate rocket_cors as cors;

use std::str::FromStr;

use rocket::Response;
use rocket::http::Method;
use rocket::http::{Header, Status};
use rocket::local::Client;

#[options("/")]
fn cors_options<'a>(cors: cors::Response) -> cors::Responder<'a, &'a str> {
    cors.responder("")
}

#[get("/")]
fn cors<'a>(cors: cors::Response) -> cors::Responder<'a, &'a str> {
    cors.responder("Hello CORS")
}

// The following routes tests that the routes can be compiled with ad-hoc CORS Response/Responders

/// Using a `Response` instead of a `Responder`
#[allow(unmounted_route)]
#[get("/")]
fn response<'a>(cors: cors::Response) -> Response<'a> {
    cors.response(Response::new())
}

/// `Responder` with String
#[allow(unmounted_route)]
#[get("/")]
fn responder_string<'a>(cors: cors::Response) -> cors::Responder<'a, String> {
    cors.responder("Hello CORS".to_string())
}

/// `Responder` with 'static ()
#[allow(unmounted_route)]
#[get("/")]
fn responder_unit(cors: cors::Response) -> cors::Responder<'static, ()> {
    cors.responder(())
}

fn make_cors_options() -> cors::Cors {
    let (allowed_origins, failed_origins) =
        cors::AllOrSome::new_from_str_list(&["https://www.acme.com"]);
    assert!(failed_origins.is_empty());

    cors::Cors {
        allowed_origins: allowed_origins,
        allowed_methods: [Method::Get].iter().cloned().collect(),
        allowed_headers: cors::AllOrSome::Some(
            ["Authorization"]
                .into_iter()
                .map(|s| s.to_string().into())
                .collect(),
        ),
        allow_credentials: true,
        ..Default::default()
    }
}

#[test]
fn smoke_test() {
    let (allowed_origins, failed_origins) =
        cors::AllOrSome::new_from_str_list(&["https://www.acme.com"]);
    assert!(failed_origins.is_empty());
    let cors_options = cors::Cors {
        allowed_origins: allowed_origins,
        allowed_methods: [Method::Get].iter().cloned().collect(),
        allowed_headers: cors::AllOrSome::Some(
            ["Authorization"]
                .iter()
                .map(|s| s.to_string().into())
                .collect(),
        ),
        allow_credentials: true,
        ..Default::default()
    };
    let rocket = rocket::ignite()
        .mount("/", routes![cors, cors_options])
        .manage(cors_options);
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

}

#[test]
fn cors_options_check() {
    let rocket = rocket::ignite()
        .mount("/", routes![cors, cors_options])
        .manage(make_cors_options());
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
}

#[test]
fn cors_get_check() {
    let rocket = rocket::ignite()
        .mount("/", routes![cors, cors_options])
        .manage(make_cors_options());
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
}

/// This test is to check that non CORS compliant requests to GET should still work. (i.e. curl)
#[test]
fn cors_get_no_origin() {
    let rocket = rocket::ignite()
        .mount("/", routes![cors, cors_options])
        .manage(make_cors_options());
    let client = Client::new(rocket).unwrap();

    let authorization = Header::new("Authorization", "let me in");
    let req = client.get("/").header(authorization);

    let mut response = req.dispatch();
    assert!(response.status().class().is_success());
    let body_str = response.body().and_then(|body| body.into_string());
    assert_eq!(body_str, Some("Hello CORS".to_string()));
}

#[test]
fn cors_options_bad_origin() {
    let rocket = rocket::ignite()
        .mount("/", routes![cors, cors_options])
        .manage(make_cors_options());
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
}

#[test]
fn cors_options_missing_origin() {
    let rocket = rocket::ignite()
        .mount("/", routes![cors, cors_options])
        .manage(make_cors_options());
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
}

#[test]
fn cors_options_bad_request_method() {
    let rocket = rocket::ignite()
        .mount("/", routes![cors, cors_options])
        .manage(make_cors_options());
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
}

#[test]
fn cors_options_bad_request_header() {
    let rocket = rocket::ignite()
        .mount("/", routes![cors, cors_options])
        .manage(make_cors_options());
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
}

#[test]
fn cors_get_bad_origin() {
    let rocket = rocket::ignite()
        .mount("/", routes![cors, cors_options])
        .manage(make_cors_options());
    let client = Client::new(rocket).unwrap();

    let origin_header = Header::from(
        hyper::header::Origin::from_str("https://www.bad-origin.com").unwrap(),
    );
    let authorization = Header::new("Authorization", "let me in");
    let req = client.get("/").header(origin_header).header(authorization);

    let response = req.dispatch();
    assert_eq!(response.status(), Status::Forbidden);
}
