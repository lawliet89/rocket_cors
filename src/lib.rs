//! [![Build Status](https://travis-ci.org/lawliet89/rocket_cors.svg)](https://travis-ci.org/lawliet89/rocket_cors)
//! [![Dependency Status](https://dependencyci.com/github/lawliet89/rocket_cors/badge)](https://dependencyci.com/github/lawliet89/rocket_cors)
//! [![Repository](https://img.shields.io/github/tag/lawliet89/rocket_cors.svg)](https://github.com/lawliet89/rocket_cors)
//! <!-- [![Crates.io](https://img.shields.io/crates/v/rocket_cors.svg)](https://crates.io/crates/rocket_cors) -->
//! <!-- [![Documentation](https://docs.rs/rocket_cors/badge.svg)](https://docs.rs/rocket_cors) -->
//!
//! - Documentation:  stable | [master branch](https://lawliet89.github.io/rocket_cors)
//!
//! Cross-origin resource sharing (CORS) for [Rocket](https://rocket.rs/) applications
//!
//! ## Requirements
//!
//! - Nightly Rust
//! - Rocket >= 0.3
//!
//! ### Nightly Rust
//!
//! Rocket requires nightly Rust. You should probably install Rust with
//! [rustup](https://www.rustup.rs/), then override the code directory to use nightly instead of
//! stable. See
//! [installation instructions](https://rocket.rs/guide/getting-started/#installing-rust).
//!
//! In particular, `rocket_cors` is currently targetted for `nightly-2017-07-13`. Newer nightlies
//! might work, but it's not guaranteed.
//!
//! ## Installation
//!
//! <!-- Add the following to Cargo.toml:
//!
//! ```toml
//! rocket_cors = "0.0.6"
//! ``` -->
//!
//! To use the latest `master` branch, for example:
//!
//! ```toml
//! rocket_cors = { git = "https://github.com/lawliet89/rocket_cors", branch = "master" }
//! ```
//!

#![allow(
    legacy_directory_ownership,
    missing_copy_implementations,
    missing_debug_implementations,
    unknown_lints,
    unsafe_code,
)]
#![deny(
    const_err,
    dead_code,
    deprecated,
    exceeding_bitshifts,
    fat_ptr_transmutes,
    improper_ctypes,
    missing_docs,
    mutable_transmutes,
    no_mangle_const_items,
    non_camel_case_types,
    non_shorthand_field_patterns,
    non_upper_case_globals,
    overflowing_literals,
    path_statements,
    plugin_as_library,
    private_no_mangle_fns,
    private_no_mangle_statics,
    stable_features,
    trivial_casts,
    trivial_numeric_casts,
    unconditional_recursion,
    unknown_crate_types,
    unreachable_code,
    unused_allocation,
    unused_assignments,
    unused_attributes,
    unused_comparisons,
    unused_extern_crates,
    unused_features,
    unused_imports,
    unused_import_braces,
    unused_qualifications,
    unused_must_use,
    unused_mut,
    unused_parens,
    unused_results,
    unused_unsafe,
    unused_variables,
    variant_size_differences,
    warnings,
    while_true,
)]

#![cfg_attr(test, feature(plugin, custom_derive))]
#![cfg_attr(test, plugin(rocket_codegen))]
#![doc(test(attr(allow(unused_variables), deny(warnings))))]

#[macro_use]
extern crate log;
#[macro_use]
extern crate rocket;
// extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate unicase;
extern crate url;
extern crate url_serde;

#[cfg(test)]
extern crate hyper;

#[cfg(test)]
#[macro_use]
mod test_macros;

pub mod headers;

// Public exports
pub use headers::Url;

use std::collections::{HashSet, HashMap};
use std::error;
use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;
use std::str::FromStr;

use rocket::{Outcome, State};
use rocket::fairing;
use rocket::http::{Method, Status};
use rocket::request::{Request, FromRequest};
use rocket::response;

use headers::{HeaderFieldName, HeaderFieldNamesSet, Origin, AccessControlRequestHeaders,
              AccessControlRequestMethod};

/// Errors during operations
///
/// This enum implements `rocket::response::Responder` which will return an appropriate status code
/// while printing out the error in the console.
/// Because these errors are usually the result of an error while trying to respond to a CORS
/// request, CORS headers cannot be added to the response and your applications requesting CORS
/// will not be able to see the status code.
#[derive(Debug)]
pub enum Error {
    /// The HTTP request header `Origin` is required but was not provided
    MissingOrigin,
    /// The HTTP request header `Origin` could not be parsed correctly.
    BadOrigin(url::ParseError),
    /// The request header `Access-Control-Request-Method` is required but is missing
    MissingRequestMethod,
    /// The request header `Access-Control-Request-Method` has an invalid value
    BadRequestMethod(rocket::Error),
    /// The request header `Access-Control-Request-Headers`  is required but is missing.
    MissingRequestHeaders,
    /// Origin is not allowed to make this request
    OriginNotAllowed,
    /// Requested method is not allowed
    MethodNotAllowed,
    /// One or more headers requested are not allowed
    HeadersNotAllowed,
    /// Credentials are allowed, but the Origin is set to "*". This is not allowed by W3C
    ///
    /// This is a misconfiguration. Check the docuemntation for `Cors`.
    CredentialsWithWildcardOrigin,
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::MissingOrigin => "The request header `Origin` is required but is missing",
            Error::BadOrigin(_) => "The request header `Origin` contains an invalid URL",
            Error::MissingRequestMethod => {
                "The request header `Access-Control-Request-Method` \
                 is required but is missing"
            }
            Error::BadRequestMethod(_) => {
                "The request header `Access-Control-Request-Method` has an invalid value"
            }
            Error::MissingRequestHeaders => {
                "The request header `Access-Control-Request-Headers` \
                is required but is missing"
            }
            Error::OriginNotAllowed => "Origin is not allowed to request",
            Error::MethodNotAllowed => "Method is not allowed",
            Error::HeadersNotAllowed => "Headers are not allowed",
            Error::CredentialsWithWildcardOrigin => {
                "Credentials are allowed, but the Origin is set to \"*\". \
                 This is not allowed by W3C"
            }

        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::BadOrigin(ref e) => Some(e),
            _ => Some(self),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::BadOrigin(ref e) => fmt::Display::fmt(e, f),
            Error::BadRequestMethod(ref e) => fmt::Debug::fmt(e, f),
            _ => write!(f, "{}", error::Error::description(self)),
        }
    }
}

impl<'r> response::Responder<'r> for Error {
    fn respond_to(self, _: &Request) -> Result<response::Response<'r>, Status> {
        error_!("CORS Error: {:?}", self);
        Err(match self {
            Error::MissingOrigin | Error::OriginNotAllowed | Error::MethodNotAllowed |
            Error::HeadersNotAllowed => Status::Forbidden,
            Error::CredentialsWithWildcardOrigin => Status::InternalServerError,
            _ => Status::BadRequest,
        })
    }
}

/// An enum signifying that some of type T is allowed, or `All` (everything is allowed).
///
/// `Default` is implemented for this enum and is `All`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllOrSome<T> {
    /// Everything is allowed. Usually equivalent to the "*" value.
    All,
    /// Only some of `T` is allowed
    Some(T),
}

impl<T> Default for AllOrSome<T> {
    fn default() -> Self {
        AllOrSome::All
    }
}

impl<T> AllOrSome<T> {
    /// Returns whether this is an `All` variant
    pub fn is_all(&self) -> bool {
        match *self {
            AllOrSome::All => true,
            AllOrSome::Some(_) => false,
        }
    }

    /// Returns whether this is a `Some` variant
    pub fn is_some(&self) -> bool {
        !self.is_all()
    }
}

impl AllOrSome<HashSet<Url>> {
    /// New `AllOrSome` from a list of URL strings.
    /// Returns a tuple where the first element is the struct `AllOrSome`,
    /// and the second element
    /// is a map of strings which failed to parse into URLs and their associated parse errors.
    pub fn new_from_str_list(urls: &[&str]) -> (Self, HashMap<String, url::ParseError>) {
        let (ok_set, error_map): (Vec<_>, Vec<_>) = urls.iter()
            .map(|s| (s.to_string(), Url::from_str(s)))
            .partition(|&(_, ref r)| r.is_ok());

        let error_map = error_map
            .into_iter()
            .map(|(s, r)| (s.to_string(), r.unwrap_err()))
            .collect();

        let ok_set = ok_set.into_iter().map(|(_, r)| r.unwrap()).collect();

        (AllOrSome::Some(ok_set), error_map)
    }
}

/// Responder generator and [Fairing](https://rocket.rs/guide/fairings/) for CORS
///
/// This struct can be used as Fairing for Rocket, or as an ad-hoc responder for any CORS requests.
///
/// You create a new copy of this struct by defining the configurations in the fields below.
/// This struct can also be deserialized by serde.
///
/// [`Default`](https://doc.rust-lang.org/std/default/trait.Default.html) is implemented for this
/// struct. The default for each field is described in the docuementation for the field.
#[derive(Clone, Debug)]
pub struct Cors {
    /// Origins that are allowed to make requests.
    /// Will be verified against the `Origin` request header.
    ///
    /// When `All` is set, and `send_wildcard` is set, "*" will be sent in
    /// the `Access-Control-Allow-Origin` response header. Otherwise, the client's `Origin` request
    /// header will be echoed back in the `Access-Control-Allow-Origin` response header.
    ///
    /// When `Some` is set, the client's `Origin` request header will be checked in a
    /// case-sensitive manner.
    ///
    /// This is the `list of origins` in the
    /// [Resource Processing Model](https://www.w3.org/TR/cors/#resource-processing-model).
    ///
    /// Defaults to `All`.
    ///
    /// ```
    // #[serde(default)]
    pub allowed_origins: AllOrSome<HashSet<Url>>,
    /// The list of methods which the allowed origins are allowed to access for
    /// non-simple requests.
    ///
    /// This is the `list of methods` in the
    /// [Resource Processing Model](https://www.w3.org/TR/cors/#resource-processing-model).
    ///
    /// Defaults to `[GET, HEAD, POST, OPTIONS, PUT, PATCH, DELETE]`
    // #[serde(default = "Cors::default_allowed_methods")]
    pub allowed_methods: HashSet<Method>,
    /// The list of header field names which can be used when this resource is accessed by allowed
    /// origins.
    ///
    /// If `All` is set, whatever is requested by the client in `Access-Control-Request-Headers`
    /// will be echoed back in the `Access-Control-Allow-Headers` header.
    ///
    /// This is the `list of headers` in the
    /// [Resource Processing Model](https://www.w3.org/TR/cors/#resource-processing-model).
    ///
    /// Defaults to `All`.
    // #[serde(default)]
    pub allowed_headers: AllOrSome<HashSet<HeaderFieldName>>,
    /// Allows users to make authenticated requests.
    /// If true, injects the `Access-Control-Allow-Credentials` header in responses.
    /// This allows cookies and credentials to be submitted across domains.
    ///
    /// This **CANNOT** be used in conjunction with `allowed_origins` set to `All` and
    /// `send_wildcard` set to `true`. Depending on the mode of usage, this will either result
    /// in an `Error::CredentialsWithWildcardOrigin` error during Rocket launch or runtime.
    ///
    /// Defaults to `false`.
    // #[serde(default)]
    pub allow_credentials: bool,
    /// The list of headers which are safe to expose to the API of a CORS API specification.
    /// This corresponds to the `Access-Control-Expose-Headers` responde header.
    ///
    /// This is the `list of exposed headers` in the
    /// [Resource Processing Model](https://www.w3.org/TR/cors/#resource-processing-model).
    ///
    /// This defaults to an empty set.
    // #[serde(default)]
    pub expose_headers: HashSet<String>,
    /// The maximum time for which this CORS request maybe cached. This value is set as the
    /// `Access-Control-Max-Age` header.
    ///
    /// This defaults to `None` (unset).
    // #[serde(default)]
    pub max_age: Option<usize>,
    /// If true, and the `allowed_origins` parameter is `All`, a wildcard
    /// `Access-Control-Allow-Origin` response header is sent, rather than the request’s
    /// `Origin` header.
    ///
    /// This is the `supports credentials flag` in the
    /// [Resource Processing Model](https://www.w3.org/TR/cors/#resource-processing-model).
    ///
    /// This **CANNOT** be used in conjunction with `allowed_origins` set to `All` and
    /// `allow_credentials` set to `true`. Depending on the mode of usage, this will either result
    /// in an `Error::CredentialsWithWildcardOrigin` error during Rocket launch or runtime.
    ///
    /// Defaults to `false`.
    // #[serde(default)]
    pub send_wildcard: bool,
}

impl Default for Cors {
    fn default() -> Self {
        Self {
            allowed_origins: Default::default(),
            allowed_methods: Self::default_allowed_methods(),
            allowed_headers: Default::default(),
            allow_credentials: Default::default(),
            expose_headers: Default::default(),
            max_age: Default::default(),
            send_wildcard: Default::default(),
        }
    }
}

impl Cors {
    /// Wrap any `Rocket::Response` and respond with CORS headers.
    /// This is only used for ad-hoc route CORS response
    fn respond<'a, 'r: 'a, R: response::Responder<'r>>(
        &'a self,
        responder: R,
    ) -> Responder<'a, 'r, R> {
        Responder::new(responder, self)
    }

    fn default_allowed_methods() -> HashSet<Method> {
        vec![
            Method::Get,
            Method::Head,
            Method::Post,
            Method::Options,
            Method::Put,
            Method::Patch,
            Method::Delete,
        ].into_iter()
            .collect()
    }

    /// Validates if any of the settings are disallowed or incorrect
    ///
    /// This is run during initial Fairing attachment
    pub fn validate(&self) -> Result<(), Error> {
        if self.allowed_origins.is_all() && self.send_wildcard && self.allow_credentials {
            Err(Error::CredentialsWithWildcardOrigin)?;
        }

        Ok(())
    }
}

impl fairing::Fairing for Cors {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "CORS",
            kind: fairing::Kind::Attach | fairing::Kind::Response,
        }
    }

    fn on_attach(&self, rocket: rocket::Rocket) -> Result<rocket::Rocket, rocket::Rocket> {
        match self.validate() {
            Ok(()) => Ok(rocket),
            Err(e) => {
                error_!("Error attaching CORS fairing: {}", e);
                Err(rocket)
            }
        }
    }

    fn on_response(&self, request: &Request, response: &mut rocket::Response) {
        use rocket::response::Responder;

        // Build and merge CORS response
        match build_cors_response(self, request) {
            Err(err) => {
                // CORS error -- overwrite the original response
                let error_response = match err.respond_to(request) {
                    Err(err) => {
                        unreachable!(
                            "Should not happen! The Error responder does not Err: {:?}",
                            err
                        )
                    }
                    Ok(error_response) => error_response,
                };
                response.merge(error_response)
            }
            Ok(cors_response) => {
                cors_response.merge(response);

                // If this was an OPTIONS request and no route can be found, we should turn this
                // into a HTTP 204 with no content body.
                // This allows the user to not have to specify an OPTIONS route for everything.
                //
                // TODO: Is there anyway we can make this smarter? Only modify status codes for
                // requests where an actual route exist?
                if has_allow_origin(&response) && request.method() == Method::Options &&
                    request.route().is_none()
                {
                    response.set_status(Status::NoContent);
                    let _ = response.take_body();
                }
            }
        };


    }
}

/// A CORS [Responder](https://rocket.rs/guide/responses/#responder)
/// which will inspect the incoming requests and respond accordingly.
///
/// If the wrapped `Responder` already has the `Access-Control-Allow-Origin` header set,
/// this responder will leave the response untouched.
/// This allows for chaining of several CORS responders.
///
/// Otherwise, the following headers may be set for the final Rocket `Response`, overwriting any
/// existing headers defined:
///
/// - `Access-Control-Allow-Origin`
/// - `Access-Control-Expose-Headers`
/// - `Access-Control-Max-Age`
/// - `Access-Control-Allow-Credentials`
/// - `Access-Control-Allow-Methods`
/// - `Access-Control-Allow-Headers`
/// - `Vary`
#[derive(Debug)]
pub struct Responder<'a, 'r: 'a, R> {
    responder: R,
    options: &'a Cors,
    marker: PhantomData<response::Responder<'r>>,
}

impl<'a, 'r: 'a, R: response::Responder<'r>> Responder<'a, 'r, R> {
    fn new(responder: R, options: &'a Cors) -> Self {
        Self {
            responder,
            options,
            marker: PhantomData,
        }
    }

    /// Respond to a request
    fn respond(self, request: &Request) -> response::Result<'r> {
        let mut response = self.responder.respond_to(request)?; // handle status errors?

        match build_cors_response(self.options, request) {
            Ok(cors_response) => {
                cors_response.merge(&mut response);
                Ok(response)
            },
            Err(e) => response::Responder::respond_to(e, request),
        }

    }
}

impl<'a, 'r: 'a, R: response::Responder<'r>> response::Responder<'r> for Responder<'a, 'r, R> {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        self.respond(request)
    }
}


/// A CORS Response which provides the following CORS headers:
///
/// - `Access-Control-Allow-Origin`
/// - `Access-Control-Expose-Headers`
/// - `Access-Control-Max-Age`
/// - `Access-Control-Allow-Credentials`
/// - `Access-Control-Allow-Methods`
/// - `Access-Control-Allow-Headers`
/// - `Vary`
#[derive(Debug)]
struct Response {
    allow_origin: Option<AllOrSome<String>>,
    allow_methods: HashSet<Method>,
    allow_headers: HeaderFieldNamesSet,
    allow_credentials: bool,
    expose_headers: HeaderFieldNamesSet,
    max_age: Option<usize>,
    vary_origin: bool,
}

impl Response {
    /// Consumes the responder and return an empty `Response`
    fn new() -> Self {
        Self {
            allow_origin: None,
            allow_headers: HashSet::new(),
            allow_methods: HashSet::new(),
            allow_credentials: false,
            expose_headers: HashSet::new(),
            max_age: None,
            vary_origin: false,
        }
    }

    /// Consumes the `Response` and return an altered response with origin and `vary_origin` set
    fn origin(mut self, origin: &str, vary_origin: bool) -> Self {
        self.allow_origin = Some(AllOrSome::Some(origin.to_string()));
        self.vary_origin = vary_origin;
        self
    }

    /// Consumes the `Response` and return an altered response with origin set to "*"
    fn any(mut self) -> Self {
        self.allow_origin = Some(AllOrSome::All);
        self
    }

    /// Consumes the Response and set credentials
    fn credentials(mut self, value: bool) -> Self {
        self.allow_credentials = value;
        self
    }

    /// Consumes the CORS, set expose_headers to
    /// passed headers and returns changed CORS
    fn exposed_headers(mut self, headers: &[&str]) -> Self {
        self.expose_headers = headers.into_iter().map(|s| s.to_string().into()).collect();
        self
    }

    /// Consumes the CORS, set max_age to
    /// passed value and returns changed CORS
    fn max_age(mut self, value: Option<usize>) -> Self {
        self.max_age = value;
        self
    }

    /// Consumes the CORS, set allow_methods to
    /// passed methods and returns changed CORS
    fn methods(mut self, methods: &HashSet<Method>) -> Self {
        self.allow_methods = methods.clone();
        self
    }

    /// Consumes the CORS, set allow_headers to
    /// passed headers and returns changed CORS
    fn headers(mut self, headers: &[&str]) -> Self {
        self.allow_headers = headers.into_iter().map(|s| s.to_string().into()).collect();
        self
    }

    /// Builds a `rocket::Response` from this struct based off some base `rocket::Response`
    ///
    /// This will overwrite any existing CORS headers
    #[cfg(test)]
    fn build<'r>(&self, base: response::Response<'r>) -> response::Response<'r> {
        let mut response = response::Response::build_from(base).finalize();
        self.merge(&mut response);
        response
    }

    /// Merge CORS headers with an existing `rocket::Response`.
    ///
    /// This will overwrite any existing CORS headers
    fn merge(&self, response: &mut response::Response) {
        // TODO: We should be able to remove this
        let origin = match self.allow_origin {
            None => {
                // This is not a CORS response
                return;
            }
            Some(ref origin) => origin,
        };

        let origin = match *origin {
            AllOrSome::All => "*".to_string(),
            AllOrSome::Some(ref origin) => origin.to_string(),
        };

        response.set_raw_header("Access-Control-Allow-Origin", origin);

        if self.allow_credentials {
            response.set_raw_header("Access-Control-Allow-Credentials", "true");
        } else {
            response.remove_header("Access-Control-Allow-Credentials");
        }

        if !self.expose_headers.is_empty() {
            let headers: Vec<String> = self.expose_headers
                .iter()
                .map(|s| s.deref().to_string())
                .collect();
            let headers = headers.join(", ");

            response.set_raw_header("Access-Control-Expose-Headers", headers);
        } else {
            response.remove_header("Access-Control-Expose-Headers");
        }

        if !self.allow_headers.is_empty() {
            let headers: Vec<String> = self.allow_headers
                .iter()
                .map(|s| s.deref().to_string())
                .collect();
            let headers = headers.join(", ");

            response.set_raw_header("Access-Control-Allow-Headers", headers);
        } else {
            response.remove_header("Access-Control-Allow-Headers");
        }

        if !self.allow_methods.is_empty() {
            let methods: Vec<_> = self.allow_methods.iter().map(|m| m.as_str()).collect();
            let methods = methods.join(", ");

            response.set_raw_header("Access-Control-Allow-Methods", methods);
        } else {
            response.remove_header("Access-Control-Allow-Methods");
        }

        if self.max_age.is_some() {
            let max_age = self.max_age.unwrap();
            response.set_raw_header("Access-Control-Max-Age", max_age.to_string());
        } else {
            response.remove_header("Access-Control-Max-Age");
        }

        if self.vary_origin {
            response.set_raw_header("Vary", "Origin");
        } else {
            response.remove_header("Vary");
        }
    }
}

/// Ad-hoc per route CORS response to requests
///
/// Note: If you use this, the lifetime parameter `'r` of your `rocket:::response::Responder<'r>`
/// CANNOT be `'static`. This is because the code generated by Rocket will implicitly try to
/// to restrain the `Request` object passed to the route to `&'static Request`, and it is not
/// possible to have such a reference.
/// See [this PR on Rocket](https://github.com/SergioBenitez/Rocket/pull/345).
pub fn respond<'a, 'r: 'a, R: response::Responder<'r>>(
    options: State<'a, Cors>,
    responder: R,
) -> Responder<'a, 'r, R> {
    options.inner().respond(responder)
}

/// Build a CORS response and merge with an existing `rocket::Response` for the request
fn build_cors_response(
    options: &Cors,
    request: &Request,
) -> Result<Response, Error> {
    // Existing CORS response?
    // if has_allow_origin(response) {
    //     return Ok(());
    // }

    // 1. If the Origin header is not present terminate this set of steps.
    // The request is outside the scope of this specification.
    let origin = origin(request)?;
    let origin = match origin {
        None => {
            // Not a CORS request
            return Ok(Response::new());
        }
        Some(origin) => origin,
    };

    // Check if the request verb is an OPTION or something else
    let cors_response = match request.method() {
        Method::Options => {
            let method = request_method(request)?;
            let headers = request_headers(request)?;
            preflight(options, origin, method, headers)
        }
        _ => actual_request(options, origin),
    }?;

    Ok(cors_response)
}

/// Consumes the responder and based on the provided list of allowed origins,
/// check if the requested origin is allowed.
/// Useful for pre-flight and during requests
fn validate_origin(
    origin: &Origin,
    allowed_origins: &AllOrSome<HashSet<Url>>,
) -> Result<(), Error> {
    match *allowed_origins {
        // Always matching is acceptable since the list of origins can be unbounded.
        AllOrSome::All => Ok(()),
        AllOrSome::Some(ref allowed_origins) => {
            allowed_origins
                .get(origin)
                .and_then(|_| Some(()))
                .ok_or_else(|| Error::OriginNotAllowed)
        }
    }
}

/// Validate allowed methods
fn validate_allowed_method(
    method: &AccessControlRequestMethod,
    allowed_methods: &HashSet<Method>,
) -> Result<(), Error> {
    let &AccessControlRequestMethod(ref request_method) = method;
    if !allowed_methods.iter().any(|m| m == request_method) {
        Err(Error::MethodNotAllowed)?
    }

    // TODO: Subset to route? Or just the method requested for?
    Ok(())
}

/// Validate allowed headers
fn validate_allowed_headers(
    headers: &AccessControlRequestHeaders,
    allowed_headers: &AllOrSome<HashSet<HeaderFieldName>>,
) -> Result<(), Error> {
    let &AccessControlRequestHeaders(ref headers) = headers;

    match *allowed_headers {
        AllOrSome::All => Ok(()),
        AllOrSome::Some(ref allowed_headers) => {
            if !headers.is_empty() && !headers.is_subset(allowed_headers) {
                Err(Error::HeadersNotAllowed)?
            }
            Ok(())
        }
    }
}

/// Gets the `Origin` request header from the request
fn origin(request: &Request) -> Result<Option<Origin>, Error> {
    match Origin::from_request(request) {
        Outcome::Forward(()) => Ok(None),
        Outcome::Success(origin) => Ok(Some(origin)),
        Outcome::Failure((_, err)) => Err(err),
    }
}

/// Gets the `Access-Control-Request-Method` request header from the request
fn request_method(request: &Request) -> Result<Option<AccessControlRequestMethod>, Error> {
    match AccessControlRequestMethod::from_request(request) {
        Outcome::Forward(()) => Ok(None),
        Outcome::Success(method) => Ok(Some(method)),
        Outcome::Failure((_, err)) => Err(err),
    }
}

/// Gets the `Access-Control-Request-Headers` request header from the request
fn request_headers(request: &Request) -> Result<Option<AccessControlRequestHeaders>, Error> {
    match AccessControlRequestHeaders::from_request(request) {
        Outcome::Forward(()) => Ok(None),
        Outcome::Success(geaders) => Ok(Some(geaders)),
        Outcome::Failure((_, err)) => Err(err),
    }
}

/// Checks if an existing Response already has the header `Access-Control-Allow-Origin`
fn has_allow_origin<'r>(response: &response::Response<'r>) -> bool {
    response.headers().get("Access-Control-Allow-Origin").next() != None
}

/// Construct a preflight response based on the options. Will return an `Err`
/// if any of the preflight checks fail.
///
/// This implementation references the
/// [W3C recommendation](https://www.w3.org/TR/cors/#resource-preflight-requests).
fn preflight(
    options: &Cors,
    origin: Origin,
    method: Option<AccessControlRequestMethod>,
    headers: Option<AccessControlRequestHeaders>,
) -> Result<Response, Error> {
    options.validate()?;
    let response = Response::new();

    // Note: All header parse failures are dealt with in the `FromRequest` trait implementation

    // 2. If the value of the Origin header is not a case-sensitive match for any of the values
    // in list of origins do not set any additional headers and terminate this set of steps.
    validate_origin(&origin, &options.allowed_origins)?;
    let response = match options.allowed_origins {
        AllOrSome::All => {
            if options.send_wildcard {
                response.any()
            } else {
                response.origin(origin.as_str(), true)
            }
        }
        AllOrSome::Some(_) => response.origin(origin.as_str(), false),
    };

    // 3. Let `method` be the value as result of parsing the Access-Control-Request-Method
    // header.
    // If there is no Access-Control-Request-Method header or if parsing failed,
    // do not set any additional headers and terminate this set of steps.
    // The request is outside the scope of this specification.

    let method = method.ok_or_else(|| Error::MissingRequestMethod)?;

    // 4. Let header field-names be the values as result of parsing the
    // Access-Control-Request-Headers headers.
    // If there are no Access-Control-Request-Headers headers
    // let header field-names be the empty list.
    // If parsing failed do not set any additional headers and terminate this set of steps.
    // The request is outside the scope of this specification.

    // 5. If method is not a case-sensitive match for any of the values in list of methods
    // do not set any additional headers and terminate this set of steps.

    validate_allowed_method(&method, &options.allowed_methods)?;
    let response = response.methods(&options.allowed_methods);

    // 6. If any of the header field-names is not a ASCII case-insensitive match for any of the
    // values in list of headers do not set any additional headers and terminate this set of
    // steps.

    let response = if let Some(ref headers) = headers {
        validate_allowed_headers(headers, &options.allowed_headers)?;
        let &AccessControlRequestHeaders(ref headers) = headers;
        response.headers(
            headers
                .iter()
                .map(|s| &**s.deref())
                .collect::<Vec<&str>>()
                .as_slice(),
        )
    } else {
        response
    };

    // 7. If the resource supports credentials add a single Access-Control-Allow-Origin header,
    // with the value of the Origin header as value, and add a
    // single Access-Control-Allow-Credentials header with the case-sensitive string "true" as
    // value.
    // Otherwise, add a single Access-Control-Allow-Origin header,
    // with either the value of the Origin header or the string "*" as value.
    // Note: The string "*" cannot be used for a resource that supports credentials.

    // Validation has been done in options.validate
    let response = response.credentials(options.allow_credentials);

    // 8. Optionally add a single Access-Control-Max-Age header
    // with as value the amount of seconds the user agent is allowed to cache the result of the
    // request.
    let response = response.max_age(options.max_age);

    // 9. If method is a simple method this step may be skipped.
    // Add one or more Access-Control-Allow-Methods headers consisting of
    // (a subset of) the list of methods.
    // If a method is a simple method it does not need to be listed, but this is not prohibited.
    // Since the list of methods can be unbounded,
    // simply returning the method indicated by Access-Control-Request-Method
    // (if supported) can be enough.

    // Done above

    // 10. If each of the header field-names is a simple header and none is Content-Type,
    // this step may be skipped.
    // Add one or more Access-Control-Allow-Headers headers consisting of (a subset of)
    // the list of headers.
    // If a header field name is a simple header and is not Content-Type,
    // it is not required to be listed. Content-Type is to be listed as only a
    // subset of its values makes it qualify as simple header.
    // Since the list of headers can be unbounded, simply returning supported headers
    // from Access-Control-Allow-Headers can be enough.

    // Done above -- we do not do anything special with simple headers

    Ok(response)
}

/// Respond to an actual request based on the settings.
/// If the `Origin` is not provided, then this request was not made by a browser and there is no
/// CORS enforcement.
fn actual_request(options: &Cors, origin: Origin) -> Result<Response, Error> {
    options.validate()?;
    let response = Response::new();

    // Note: All header parse failures are dealt with in the `FromRequest` trait implementation

    // 2. If the value of the Origin header is not a case-sensitive match for any of the values
    // in list of origins, do not set any additional headers and terminate this set of steps.
    // Always matching is acceptable since the list of origins can be unbounded.

    validate_origin(&origin, &options.allowed_origins)?;
    let response = match options.allowed_origins {
        AllOrSome::All => {
            if options.send_wildcard {
                response.any()
            } else {
                response.origin(origin.as_str(), true)
            }
        }
        AllOrSome::Some(_) => response.origin(origin.as_str(), false),
    };

    // 3. If the resource supports credentials add a single Access-Control-Allow-Origin header,
    // with the value of the Origin header as value, and add a
    // single Access-Control-Allow-Credentials header with the case-sensitive string "true" as
    // value.
    // Otherwise, add a single Access-Control-Allow-Origin header,
    // with either the value of the Origin header or the string "*" as value.
    // Note: The string "*" cannot be used for a resource that supports credentials.

    // Validation has been done in options.validate
    let response = response.credentials(options.allow_credentials);

    // 4. If the list of exposed headers is not empty add one or more
    // Access-Control-Expose-Headers headers, with as values the header field names given in
    // the list of exposed headers.
    // By not adding the appropriate headers resource can also clear the preflight result cache
    // of all entries where origin is a case-sensitive match for the value of the Origin header
    // and url is a case-sensitive match for the URL of the resource.

    let response = response.exposed_headers(
        options
            .expose_headers
            .iter()
            .map(|s| &**s)
            .collect::<Vec<&str>>()
            .as_slice(),
    );
    Ok(response)
}

#[cfg(test)]
#[allow(unmounted_route)]
mod tests {
    use std::str::FromStr;
    use rocket::http::Method;
    use super::*;

    fn make_cors_options() -> Cors {
        let (allowed_origins, failed_origins) =
            AllOrSome::new_from_str_list(&["https://www.acme.com"]);
        assert!(failed_origins.is_empty());

        Cors {
            allowed_origins: allowed_origins,
            allowed_methods: [Method::Get].iter().cloned().collect(),
            allowed_headers: AllOrSome::Some(
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
    fn cors_is_validated() {
        assert!(make_cors_options().validate().is_ok())
    }

    #[test]
    #[should_panic(expected = "CredentialsWithWildcardOrigin")]
    fn cors_validates_illegal_allow_credentials() {
        let mut cors = make_cors_options();
        cors.allow_credentials = true;
        cors.allowed_origins = AllOrSome::All;
        cors.send_wildcard = true;

        cors.validate().unwrap();
    }

    // The following tests check validation

    #[test]
    fn validate_origin_allows_all_origins() {
        let url = "https://www.example.com";
        let origin = Origin::from_str(url).unwrap();
        let allowed_origins = AllOrSome::All;

        not_err!(validate_origin(&origin, &allowed_origins));
    }

    #[test]
    fn response_allows_origin() {
        let url = "https://www.example.com";
        let origin = Origin::from_str(url).unwrap();
        let (allowed_origins, failed_origins) =
            AllOrSome::new_from_str_list(&["https://www.example.com"]);
        assert!(failed_origins.is_empty());

        not_err!(validate_origin(&origin, &allowed_origins));
    }

    #[test]
    #[should_panic(expected = "OriginNotAllowed")]
    fn response_rejects_invalid_origin() {
        let url = "https://www.acme.com";
        let origin = Origin::from_str(url).unwrap();
        let (allowed_origins, failed_origins) =
            AllOrSome::new_from_str_list(&["https://www.example.com"]);
        assert!(failed_origins.is_empty());

        validate_origin(&origin, &allowed_origins).unwrap();
    }

    #[test]
    fn response_sets_exposed_headers_correctly() {
        let headers = vec!["Bar", "Baz", "Foo"];
        let response = Response::new();
        let response = response.origin("https://www.example.com", false);
        let response = response.exposed_headers(&headers);

        // Build response and check built response header
        let response = response.build(response::Response::new());
        let actual_header: Vec<_> = response
            .headers()
            .get("Access-Control-Expose-Headers")
            .collect();

        assert_eq!(1, actual_header.len());
        let mut actual_headers: Vec<String> = actual_header[0]
            .split(',')
            .map(|header| header.trim().to_string())
            .collect();
        actual_headers.sort();
        assert_eq!(headers, actual_headers);
    }

    #[test]
    fn response_sets_max_age_correctly() {
        let response = Response::new();
        let response = response.origin("https://www.example.com", false);

        let response = response.max_age(Some(42));

        // Build response and check built response header
        let expected_header = vec!["42"];
        let response = response.build(response::Response::new());
        let actual_header: Vec<_> = response.headers().get("Access-Control-Max-Age").collect();
        assert_eq!(expected_header, actual_header);
    }

    #[test]
    fn response_does_not_set_max_age_when_none() {
        let response = Response::new();
        let response = response.origin("https://www.example.com", false);

        let response = response.max_age(None);

        // Build response and check built response header
        let response = response.build(response::Response::new());
        assert!(
            response
                .headers()
                .get("Access-Control-Max-Age")
                .next()
                .is_none()
        )
    }

    #[test]
    fn allowed_methods_validated_correctly() {
        let allowed_methods = vec![Method::Get, Method::Head, Method::Post]
            .into_iter()
            .collect();

        let method = "GET";

        not_err!(validate_allowed_method(
            &FromStr::from_str(method).expect("not to fail"),
            &allowed_methods,
        ));
    }

    #[test]
    #[should_panic(expected = "MethodNotAllowed")]
    fn allowed_methods_errors_on_disallowed_method() {
        let allowed_methods = vec![Method::Get, Method::Head, Method::Post]
            .into_iter()
            .collect();

        let method = "DELETE";

        validate_allowed_method(
            &FromStr::from_str(method).expect("not to fail"),
            &allowed_methods,
        ).unwrap()
    }

    #[test]
    fn all_allowed_headers_are_validated_correctly() {
        let allowed_headers = AllOrSome::All;
        let requested_headers = vec!["Bar", "Foo"];

        not_err!(validate_allowed_headers(
            &FromStr::from_str(&requested_headers.join(",")).unwrap(),
            &allowed_headers,
        ));
    }

    /// `Response::allowed_headers` should check that headers are allowed, and only
    /// echoes back the list that is actually requested for and not the whole list
    #[test]
    fn allowed_headers_are_validated_correctly() {
        let allowed_headers = vec!["Bar", "Baz", "Foo"];
        let requested_headers = vec!["Bar", "Foo"];

        not_err!(validate_allowed_headers(
            &FromStr::from_str(&requested_headers.join(",")).unwrap(),
            &AllOrSome::Some(
                allowed_headers
                    .iter()
                    .map(|s| FromStr::from_str(*s).unwrap())
                    .collect(),
            ),
        ));
    }

    #[test]
    #[should_panic(expected = "HeadersNotAllowed")]
    fn allowed_headers_errors_on_non_subset() {
        let allowed_headers = vec!["Bar", "Baz", "Foo"];
        let requested_headers = vec!["Bar", "Foo", "Unknown"];

        validate_allowed_headers(
            &FromStr::from_str(&requested_headers.join(",")).unwrap(),
            &AllOrSome::Some(
                allowed_headers
                    .iter()
                    .map(|s| FromStr::from_str(*s).unwrap())
                    .collect(),
            ),
        ).unwrap();

    }

    #[test]
    fn response_does_not_build_if_origin_is_not_set() {
        let response = Response::new();
        let response = response.build(response::Response::new());

        let headers: Vec<_> = response.headers().iter().collect();
        assert_eq!(headers.len(), 0);
    }

    #[test]
    fn response_build_removes_existing_cors_headers_and_keeps_others() {
        use std::io::Cursor;

        let original = response::Response::build()
            .status(Status::ImATeapot)
            .raw_header("X-Teapot-Make", "Rocket")
            .raw_header("Access-Control-Max-Age", "42")
            .sized_body(Cursor::new("Brewing the best coffee!"))
            .finalize();

        let response = Response::new();
        let response = response.origin("https://www.example.com", false);
        let response = response.build(original);
        // Check CORS header
        let expected_header = vec!["https://www.example.com"];
        let actual_header: Vec<_> = response
            .headers()
            .get("Access-Control-Allow-Origin")
            .collect();
        assert_eq!(expected_header, actual_header);

        // Check other header
        let expected_header = vec!["Rocket"];
        let actual_header: Vec<_> = response.headers().get("X-Teapot-Make").collect();
        assert_eq!(expected_header, actual_header);

        // Check that `Access-Control-Max-Age` is removed
        assert!(
            response
                .headers()
                .get("Access-Control-Max-Age")
                .next()
                .is_none()
        );


    }

    // TODO: Preflight tests
    // TODO: Actual requests tests

    // Origin all (wildcard + echoed with Vary). Origin Echo
}
