//! Fairing implementation

#[allow(unused_imports)]
use ::log::{error, info};
use rocket::http::{self, uri::Origin, Status};
use rocket::{self, error_, info_, outcome::Outcome, Request};

use crate::{
    actual_request_response, origin, preflight_response, request_headers, validate, Cors, Error,
};

/// Request Local State to store CORS validation results
enum CorsValidation {
    Success,
    Failure,
}

/// Create a `Handler` for Fairing error handling
#[derive(Clone)]
struct FairingErrorRoute {}

#[rocket::async_trait]
impl rocket::route::Handler for FairingErrorRoute {
    async fn handle<'r>(
        &self,
        request: &'r Request<'_>,
        _: rocket::Data<'r>,
    ) -> rocket::route::Outcome<'r> {
        let status = request
            .param::<u16>(0)
            .unwrap_or(Ok(0))
            .unwrap_or_else(|e| {
                error_!("Fairing Error Handling Route error: {:?}", e);
                500
            });
        let status = Status::from_code(status).unwrap_or(Status::InternalServerError);
        Outcome::Error(status)
    }
}

/// Create a new `Route` for Fairing handling
fn fairing_route(rank: isize) -> rocket::Route {
    rocket::Route::ranked(rank, http::Method::Get, "/<status>", FairingErrorRoute {})
}

/// Modifies a `Request` to route to Fairing error handler
fn route_to_fairing_error_handler(options: &Cors, status: u16, request: &mut Request<'_>) {
    let origin = Origin::parse_owned(format!("{}/{}", options.fairing_route_base, status)).unwrap();

    request.set_method(http::Method::Get);
    request.set_uri(origin);
}

fn on_response_wrapper(
    options: &Cors,
    request: &Request<'_>,
    response: &mut rocket::Response<'_>,
) -> Result<(), Error> {
    let origin = match origin(request)? {
        None => {
            // Not a CORS request
            return Ok(());
        }
        Some(origin) => origin,
    };

    let result = request.local_cache(|| unreachable!("This should not be executed so late"));

    if let CorsValidation::Failure = *result {
        // Nothing else for us to do
        return Ok(());
    }

    let origin = origin.to_string();
    let cors_response = if request.method() == http::Method::Options {
        let headers = request_headers(request)?;
        preflight_response(options, &origin, headers.as_ref())
    } else {
        actual_request_response(options, &origin)
    };

    cors_response.merge(response);

    // If this was an OPTIONS request and no route can be found, we should turn this
    // into a HTTP 204 with no content body.
    // This allows the user to not have to specify an OPTIONS route for everything.
    //
    // TODO: Is there anyway we can make this smarter? Only modify status codes for
    // requests where an actual route exist?
    if request.method() == http::Method::Options && request.route().is_none() {
        info_!(
            "CORS Fairing: Turned missing route {} into an OPTIONS pre-flight request",
            request
        );
        response.set_status(Status::NoContent);
        let _ = response.body_mut().take();
    }
    Ok(())
}

#[rocket::async_trait]
impl rocket::fairing::Fairing for Cors {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "CORS",
            kind: rocket::fairing::Kind::Ignite
                | rocket::fairing::Kind::Request
                | rocket::fairing::Kind::Response,
        }
    }

    async fn on_ignite(&self, rocket: rocket::Rocket<rocket::Build>) -> rocket::fairing::Result {
        Ok(rocket.mount(
            &self.fairing_route_base,
            vec![fairing_route(self.fairing_route_rank)],
        ))
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut rocket::Data<'_>) {
        let result = match validate(self, request) {
            Ok(_) => CorsValidation::Success,
            Err(err) => {
                error_!("CORS Error: {}", err);
                let status = err.status();
                route_to_fairing_error_handler(self, status.code, request);
                CorsValidation::Failure
            }
        };

        let _ = request.local_cache(|| result);
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut rocket::Response<'r>) {
        if let Err(err) = on_response_wrapper(self, request, response) {
            error_!("Fairings on_response error: {}\nMost likely a bug", err);
            response.set_status(Status::InternalServerError);
            let _ = response.body();
        }
    }
}

#[cfg(test)]
mod tests {
    use rocket::http::{Method, Status};
    use rocket::local::blocking::Client;
    use rocket::Rocket;

    use crate::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};

    const CORS_ROOT: &str = "/my_cors";

    fn make_cors_options() -> Cors {
        let allowed_origins = AllowedOrigins::some_exact(&["https://www.acme.com"]);

        CorsOptions {
            allowed_origins,
            allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
            allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
            allow_credentials: true,
            fairing_route_base: CORS_ROOT.to_string(),

            ..Default::default()
        }
        .to_cors()
        .expect("Not to fail")
    }

    fn rocket(fairing: Cors) -> Rocket<rocket::Build> {
        Rocket::build().attach(fairing)
    }

    #[test]
    #[allow(non_snake_case)]
    fn FairingErrorRoute_returns_passed_in_status() {
        let client = Client::tracked(rocket(make_cors_options())).expect("to not fail");
        let request = client.get(format!("{}/403", CORS_ROOT));
        let response = request.dispatch();
        assert_eq!(Status::Forbidden, response.status());
    }

    #[test]
    #[allow(non_snake_case)]
    fn FairingErrorRoute_returns_500_for_unknown_status() {
        let client = Client::tracked(rocket(make_cors_options())).expect("to not fail");
        let request = client.get(format!("{}/999", CORS_ROOT));
        let response = request.dispatch();
        assert_eq!(Status::InternalServerError, response.status());
    }

    #[rocket::async_test]
    async fn error_route_is_mounted_on_ignite() {
        let rocket = rocket(make_cors_options())
            .ignite()
            .await
            .expect("to ignite");

        let expected_uri = format!("{}/<status>", CORS_ROOT);
        let error_route = rocket
            .routes()
            .find(|r| r.method == Method::Get && r.uri.to_string() == expected_uri);
        assert!(error_route.is_some());
    }

    // Rest of the things can only be tested in integration tests
}
