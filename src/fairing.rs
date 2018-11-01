//! Fairing implementation

use log::{error, info, log};
use rocket::http::{self, uri::Origin, Status};
use rocket::{self, error_, info_, log_, Outcome, Request};

use crate::{
    actual_request_response, origin, preflight_response, request_headers, validate, Cors, Error,
};

/// Request Local State to store CORS validation results
enum CorsValidation {
    Success,
    Failure,
}

/// Route for Fairing error handling
pub(crate) fn fairing_error_route<'r>(
    request: &'r Request,
    _: rocket::Data,
) -> rocket::handler::Outcome<'r> {
    let status = request
        .get_param::<u16>(0)
        .unwrap_or(Ok(0))
        .unwrap_or_else(|e| {
            error_!("Fairing Error Handling Route error: {:?}", e);
            500
        });
    let status = Status::from_code(status).unwrap_or_else(|| Status::InternalServerError);
    Outcome::Failure(status)
}

/// Create a new `Route` for Fairing handling
fn fairing_route(rank: isize) -> rocket::Route {
    rocket::Route::ranked(rank, http::Method::Get, "/<status>", fairing_error_route)
}

/// Modifies a `Request` to route to Fairing error handler
fn route_to_fairing_error_handler(options: &Cors, status: u16, request: &mut Request) {
    let origin = Origin::parse_owned(format!("{}/{}", options.fairing_route_base, status)).unwrap();

    request.set_method(http::Method::Get);
    request.set_uri(origin);
}

fn on_response_wrapper(
    options: &Cors,
    request: &Request,
    response: &mut rocket::Response,
) -> Result<(), Error> {
    let origin = match origin(request)? {
        None => {
            // Not a CORS request
            return Ok(());
        }
        Some(origin) => origin,
    };

    let result = request.local_cache(|| unreachable!("This should not be executed so late"));

    if let &CorsValidation::Failure = result {
        // Nothing else for us to do
        return Ok(());
    }

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
        let _ = response.take_body();
    }
    Ok(())
}

impl rocket::fairing::Fairing for Cors {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "CORS",
            kind: rocket::fairing::Kind::Attach
                | rocket::fairing::Kind::Request
                | rocket::fairing::Kind::Response,
        }
    }

    fn on_attach(&self, rocket: rocket::Rocket) -> Result<rocket::Rocket, rocket::Rocket> {
        match self.validate() {
            Ok(()) => Ok(rocket.mount(
                &self.fairing_route_base,
                vec![fairing_route(self.fairing_route_rank)],
            )),
            Err(e) => {
                error_!("Error attaching CORS fairing: {}", e);
                Err(rocket)
            }
        }
    }

    fn on_request(&self, request: &mut Request, _: &rocket::Data) {
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

    fn on_response(&self, request: &Request, response: &mut rocket::Response) {
        if let Err(err) = on_response_wrapper(self, request, response) {
            error_!("Fairings on_response error: {}\nMost likely a bug", err);
            response.set_status(Status::InternalServerError);
            let _ = response.take_body();
        }
    }
}

#[cfg(test)]
mod tests {
    use rocket::http::{Method, Status};
    use rocket::local::Client;
    use rocket::Rocket;

    use crate::{AllOrSome, AllowedHeaders, AllowedOrigins, Cors};

    const CORS_ROOT: &'static str = "/my_cors";

    fn make_cors_options() -> Cors {
        let (allowed_origins, failed_origins) = AllowedOrigins::some(&["https://www.acme.com"]);
        assert!(failed_origins.is_empty());

        Cors {
            allowed_origins: allowed_origins,
            allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
            allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
            allow_credentials: true,
            fairing_route_base: CORS_ROOT.to_string(),

            ..Default::default()
        }
    }

    fn rocket(fairing: Cors) -> Rocket {
        Rocket::ignite().attach(fairing)
    }

    #[test]
    fn fairing_error_route_returns_passed_in_status() {
        let client = Client::new(rocket(make_cors_options())).expect("to not fail");
        let request = client.get(format!("{}/403", CORS_ROOT));
        let response = request.dispatch();
        assert_eq!(Status::Forbidden, response.status());
    }

    #[test]
    fn fairing_error_route_returns_500_for_unknown_status() {
        let client = Client::new(rocket(make_cors_options())).expect("to not fail");
        let request = client.get(format!("{}/999", CORS_ROOT));
        let response = request.dispatch();
        assert_eq!(Status::InternalServerError, response.status());
    }

    #[test]
    fn error_route_is_mounted_on_attach() {
        let rocket = rocket(make_cors_options());

        let expected_uri = format!("{}/<status>", CORS_ROOT);
        let error_route = rocket
            .routes()
            .find(|r| r.method == Method::Get && r.uri.to_string() == expected_uri);
        assert!(error_route.is_some());
    }

    #[test]
    #[should_panic(expected = "launch fairing failure")]
    fn options_are_validated_on_attach() {
        let mut options = make_cors_options();
        options.allowed_origins = AllOrSome::All;
        options.send_wildcard = true;

        let _ = rocket(options).launch();
    }

    // Rest of the things can only be tested in integration tests
}
