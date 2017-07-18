//! Fairing implementation
use rocket::{self, Request, Outcome};
use rocket::http::{self, Status};

use {Cors, Error, validate, preflight_response, actual_request_response, origin, request_headers};

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

    let cors_response = if request.method() == http::Method::Options {
        let headers = request_headers(request)?;
        preflight_response(options, origin, headers)
    } else {
        actual_request_response(options, origin)
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
        // Build and merge CORS response
        // Type annotation is for sanity check
        let cors_response = validate(self, request);
        if let Err(ref err) = cors_response {
            error_!("CORS Error: {}", err);
            let status = err.status();
            route_to_fairing_error_handler(self, status.code, request);
        }
    }

    fn on_response(&self, request: &Request, response: &mut rocket::Response) {
        if let Err(err) = on_response_wrapper(self, request, response) {
            error_!("Fairings on_response error: {}\nMost likely a bug", err);
            response.set_status(Status::InternalServerError);
            let _ = response.take_body();
        }
    }
}
