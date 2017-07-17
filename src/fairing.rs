//! Fairing implementation
use base64;
use rocket::{self, Request, Outcome};
use rocket::http::{self, Status, Header};
use rmps;

use {Cors, Response, Error, build_cors_response};

static HEADER_NAME: &'static str = "ROCKET-CORS";

/// Type of the Request header the `on_request` fairing handler will inject into requests
/// for `on_response` to deal with
pub(crate) type CorsInjectedHeader = Result<Response, String>;

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

/// Inject `CorsInjectedHeader` into the request header
fn inject_request_header(
    response: &CorsInjectedHeader,
    request: &mut Request,
) -> Result<(), Error> {
    let serialized = rmps::to_vec(response).map_err(Error::RmpSerializationError)?;
    let base64 = base64::encode_config(&serialized, base64::URL_SAFE);
    request.replace_header(Header::new(HEADER_NAME, base64));
    Ok(())
}

/// Extract the injected `CorsInjectedHeader`
fn extract_request_header(request: &Request) -> Result<Option<CorsInjectedHeader>, Error> {
    let header = match request.headers().get_one(HEADER_NAME) {
        Some(header) => header,
        None => return Ok(None),
    };

    let bytes = base64::decode_config(header, base64::URL_SAFE).map_err(
        Error::Base64DecodeError,
    )?;
    let deserialized: CorsInjectedHeader = rmps::from_slice(&bytes).map_err(
        Error::RmpDeserializationError,
    )?;
    Ok(Some(deserialized))
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
        // Build and merge CORS response
        // Type annotation is for sanity check
        let cors_response = build_cors_response(self, request);
        if let Err(ref err) = cors_response {
            error_!("CORS Error: {}", err);
            let status = err.status();
            route_to_fairing_error_handler(self, status.code, request);
        }

        let cors_response = cors_response.map_err(|e| e.to_string());

        if let Err(err) = inject_request_header(&cors_response, request) {
            // Internal server error -- probably a bug
            error_!(
                "Fairing had an error injecting headers: {}\nThis might be a bug. Please report.",
                err
            );
            let status = err.status();
            route_to_fairing_error_handler(self, status.code, request);
        }
    }

    fn on_response(&self, request: &Request, response: &mut rocket::Response) {
        let header = match extract_request_header(request) {
            Err(err) => {
                // We have a bug
                error_!(
                    "Fairing had an error extracting headers: {}\nThis might be a bug. \
                    Please report.",
                    err
                );

                // Let's respond with an internal server error
                response.set_status(Status::InternalServerError);
                let _ = response.take_body();
                return;
            }
            Ok(header) => header,
        };

        let header = match header {
            None => {
                // This is not a CORS request
                return;
            }
            Some(header) => header,
        };

        match header {
            Err(_) => {
                // We have dealt with this already
            }
            Ok(cors_response) => {
                cors_response.merge(response);

                // If this was an OPTIONS request and no route can be found, we should turn this
                // into a HTTP 204 with no content body.
                // This allows the user to not have to specify an OPTIONS route for everything.
                //
                // TODO: Is there anyway we can make this smarter? Only modify status codes for
                // requests where an actual route exist?
                if request.method() == http::Method::Options && request.route().is_none() {
                    response.set_status(Status::NoContent);
                    let _ = response.take_body();
                }
            }
        }
    }
}
