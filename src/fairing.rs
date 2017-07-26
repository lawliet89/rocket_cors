//! Fairing implementation
use std::str::FromStr;

use rocket::{self, Request, Outcome};
use rocket::http::{self, Status, Header};

use {Cors, Error, validate, preflight_response, actual_request_response, origin, request_headers};

/// An injected header to quickly give the result of CORS
static CORS_HEADER: &str = "ROCKET-CORS";
enum InjectedHeader {
    Success,
    Failure,
}

impl InjectedHeader {
    fn to_str(&self) -> &'static str {
        match *self {
            InjectedHeader::Success => "Success",
            InjectedHeader::Failure => "Failure",
        }
    }
}

impl FromStr for InjectedHeader {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "Success" => Ok(InjectedHeader::Success),
            "Failure" => Ok(InjectedHeader::Failure),
            other => {
                error_!(
                    "Unknown injected header encountered: {}\nThis is probably a bug.",
                    other
                );
                Err(Error::UnknownInjectedHeader)
            }
        }
    }
}

/// Route for Fairing error handling
pub(crate) fn fairing_error_route<'r>(
    request: &'r Request,
    _: rocket::Data,
) -> rocket::handler::Outcome<'r> {
    let status = request.get_param::<u16>(0).unwrap_or_else(|e| {
        error_!("Fairing Error Handling Route error: {:?}", e);
        500
    });
    let status = Status::from_code(status).unwrap_or_else(|| Status::InternalServerError);
    Outcome::Failure(status)
}

/// Create a new `Route` for Fairing handling
fn fairing_route() -> rocket::Route {
    rocket::Route::new(http::Method::Get, "/<status>", fairing_error_route)
}

/// Modifies a `Request` to route to Fairing error handler
fn route_to_fairing_error_handler(options: &Cors, status: u16, request: &mut Request) {
    request.set_method(http::Method::Get);
    request.set_uri(format!("{}/{}", options.fairing_route_base, status));
}

/// Inject a header into the Request with result
fn inject_request_header(header: &InjectedHeader, request: &mut Request) {
    request.replace_header(Header::new(CORS_HEADER, header.to_str()));
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

    // Get validation result from injected header
    let injected_header = request.headers().get_one(CORS_HEADER).ok_or_else(|| {
        Error::MissingInjectedHeader
    })?;
    let result = InjectedHeader::from_str(injected_header)?;

    if let InjectedHeader::Failure = result {
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
    if request.method() == http::Method::Options && request.method() == http::Method::Options &&
        request.route().is_none()
    {
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
            kind: rocket::fairing::Kind::Attach | rocket::fairing::Kind::Request |
                rocket::fairing::Kind::Response,
        }
    }

    fn on_attach(&self, rocket: rocket::Rocket) -> Result<rocket::Rocket, rocket::Rocket> {
        match self.validate() {
            Ok(()) => {
                Ok(rocket.mount(&self.fairing_route_base, vec![fairing_route()]))
            }
            Err(e) => {
                error_!("Error attaching CORS fairing: {}", e);
                Err(rocket)
            }
        }
    }

    fn on_request(&self, request: &mut Request, _: &rocket::Data) {
        let injected_header = match validate(self, request) {
            Ok(_) => InjectedHeader::Success,
            Err(err) => {
                error_!("CORS Error: {}", err);
                let status = err.status();
                route_to_fairing_error_handler(self, status.code, request);
                InjectedHeader::Failure
            }
        };

        inject_request_header(&injected_header, request);
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
    use rocket::Rocket;
    use rocket::http::{Method, Status};
    use rocket::local::Client;

    use {Cors, AllOrSome, AllowedOrigins, AllowedHeaders};

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
        let error_route = rocket.routes().find(|r| {
            r.method == Method::Get && r.uri.as_str() == expected_uri
        });
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
