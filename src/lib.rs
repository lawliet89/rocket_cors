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
//! - Rocket > 0.3
//!
//! ### Nightly Rust
//!
//! Rocket requires nightly Rust. You should probably install Rust with
//! [rustup](https://www.rustup.rs/), then override the code directory to use nightly instead of
//! stable. See
//! [installation instructions](https://rocket.rs/guide/getting-started/#installing-rust).
//!
//! In particular, `rocket_cors` is currently targetted for `nightly-2017-07-13`.
//!
//! ### Rocket > 0.3
//!
//! Rocket > 0.3 is needed. At this moment, `0.3` is not released, and this crate will not be
//! published
//! to Crates.io until Rocket 0.3 is released to Crates.io.
//!
//! We currently tie this crate to revision
//! [aa51fe0](https://github.com/SergioBenitez/Rocket/tree/aa51fe0) of Rocket.
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

use std::collections::{HashSet, HashMap};
use std::error;
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use rocket::request::{self, Request, FromRequest};
use rocket::response;
use rocket::http::{Method, Status};
use rocket::Outcome;
use unicase::UniCase;

#[cfg(test)]
#[macro_use]
mod test_macros;

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
    /// This is a misconfiguration. Check the docuemntation for `Options`.
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

/// A wrapped `url::Url` to allow for deserialization
#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize)]
pub struct Url(
    #[serde(with = "url_serde")]
    url::Url
);

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for Url {
    type Target = url::Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Url {
    type Err = url::ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let url = url::Url::from_str(input)?;
        Ok(Url(url))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Url {
    type Error = Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Error> {
        match request.headers().get_one("Origin") {
            Some(origin) => {
                match Self::from_str(origin) {
                    Ok(origin) => Outcome::Success(origin),
                    Err(e) => Outcome::Failure((Status::BadRequest, Error::BadOrigin(e))),
                }
            }
            None => Outcome::Forward(()),
        }
    }
}


/// The `Origin` request header used in CORS
///
/// You can use this as a rocket [Request Guard](https://rocket.rs/guide/requests/#request-guards)
/// to ensure that Origins are passed in correctly.
pub type Origin = Url;

/// The `Access-Control-Request-Method` request header
#[derive(Debug)]
pub struct AccessControlRequestMethod(pub Method);

impl FromStr for AccessControlRequestMethod {
    type Err = rocket::Error;

    fn from_str(method: &str) -> Result<Self, Self::Err> {
        Ok(AccessControlRequestMethod(Method::from_str(method)?))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for AccessControlRequestMethod {
    type Error = Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Error> {
        match request.headers().get_one("Access-Control-Request-Method") {
            Some(request_method) => {
                match Self::from_str(request_method) {
                    Ok(request_method) => Outcome::Success(request_method),
                    Err(e) => Outcome::Failure((Status::BadRequest, Error::BadRequestMethod(e))),
                }
            }
            None => Outcome::Forward(()),
        }
    }
}

type HeaderFieldName = UniCase<String>;
type HeaderFieldNamesSet = HashSet<HeaderFieldName>;

/// The `Access-Control-Request-Headers` request header
#[derive(Debug)]
pub struct AccessControlRequestHeaders(pub HeaderFieldNamesSet);

/// Will never fail
impl FromStr for AccessControlRequestHeaders {
    type Err = ();

    /// Will never fail
    fn from_str(headers: &str) -> Result<Self, Self::Err> {
        if headers.trim().is_empty() {
            return Ok(AccessControlRequestHeaders(HashSet::new()));
        }

        let set: HeaderFieldNamesSet = headers
            .split(',')
            .map(|header| UniCase(header.trim().to_string()))
            .collect();
        Ok(AccessControlRequestHeaders(set))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for AccessControlRequestHeaders {
    type Error = Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Error> {
        match request.headers().get_one("Access-Control-Request-Headers") {
            Some(request_headers) => {
                match Self::from_str(request_headers) {
                    Ok(request_headers) => Outcome::Success(request_headers),
                    Err(()) => {
                        unreachable!("`AccessControlRequestHeaders::from_str` should never fail")
                    }
                }
            }
            None => Outcome::Forward(()),
        }
    }
}

/// An enum signifying that some of type T is allowed, or `All` (everything is allowed).
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

/// Responder and Fairing for CORS
///
/// This struct can be used as Fairing for Rocket, or as an ad-hoc responder for any CORS requests.
/// You create a new copy of this struct by defining the configurations in the fields below.
/// This struct can also be deserialized by serde.
///
/// [`Default`](https://doc.rust-lang.org/std/default/trait.Default.html) is implemented for this
/// struct. The default for each field is described in the docuementation for the field.
#[derive(Clone, Debug)]
pub struct Options {
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
    // #[serde(default = "Options::default_allowed_methods")]
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

impl Default for Options {
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

impl Options {
    /// Wrap any `Rocket::Response` and respond with CORS headers.
    /// This is only used for ad-hoc route CORS response
    pub fn respond<'r, R: response::Responder<'r>>(&'r self, responder: R) -> Responder<'r, R> {
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
}

/// A CORS Responder which will inspect the incoming requests and respond accoridingly.
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
pub struct Responder<'r, R> {
    responder: R,
    options: &'r Options,
}

impl<'r, R: response::Responder<'r>> Responder<'r, R> {
    fn new(responder: R, options: &'r Options) -> Self {
        Self { responder, options }
    }

    /// Respond to a request
    fn respond(self, request: &Request) -> response::Result<'r> {
        match self.build_cors_response(request) {
            Ok(response) => response,
            Err(e) => response::Responder::respond_to(e, request),
        }
    }

    /// Build a CORS response and merge with an existing `rocket::Response` for the request
    fn build_cors_response(self, request: &Request) -> Result<response::Result<'r>, Error> {
        let original_response = match self.responder.respond_to(request) {
            Ok(response) => response,
            Err(status) => return Ok(Err(status)),
        };

        if Self::has_allow_origin(&original_response) {
            return Ok(Ok(original_response));
        }

        // 1. If the Origin header is not present terminate this set of steps.
        // The request is outside the scope of this specification.
        let origin = Self::origin(request)?;
        let origin = match origin {
            None => {
                // Not a CORS request
                return Ok(Ok(original_response));
            }
            Some(origin) => origin,
        };

        // Check if the request verb is an OPTION or something else
        let cors_response = match request.method() {
            Method::Options => {
                let method = Self::request_method(request)?;
                let headers = Self::request_headers(request)?;
                Self::preflight(&self.options, origin, method, headers)
            }
            _ => Self::actual_request(&self.options, origin),
        }?;


        // If the original response is an error status, we can turn it into an em

        // TODO
        Ok(Ok(Self::merge(cors_response.build(), original_response)))
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
    fn has_allow_origin(response: &response::Response<'r>) -> bool {
        response.headers().get("Access-Control-Allow-Origin").next() != None
    }

    /// Construct a preflight response based on the options. Will return an `Err`
    /// if any of the preflight checks fail.
    ///
    /// This implementation references the
    /// [W3C recommendation](https://www.w3.org/TR/cors/#resource-preflight-requests).
    fn preflight(
        options: &Options,
        origin: Origin,
        method: Option<AccessControlRequestMethod>,
        headers: Option<AccessControlRequestHeaders>,
    ) -> Result<Response, Error> {

        let response = Response::new();

        // Note: All header parse failures are dealt with in the `FromRequest` trait implementation

        // 2. If the value of the Origin header is not a case-sensitive match for any of the values
        // in list of origins do not set any additional headers and terminate this set of steps.
        let response = response.allowed_origin(
            &origin,
            &options.allowed_origins,
            options.send_wildcard,
        )?;

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

        let response = response.allowed_methods(&method, &options.allowed_methods)?;

        // 6. If any of the header field-names is not a ASCII case-insensitive match for any of the
        // values in list of headers do not set any additional headers and terminate this set of
        // steps.
        let response = if let Some(headers) = headers {
            response.allowed_headers(&headers, &options.allowed_headers)?
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

        let response = response.credentials(options.allow_credentials)?;

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
    fn actual_request(options: &Options, origin: Origin) -> Result<Response, Error> {
        let response = Response::new();

        // Note: All header parse failures are dealt with in the `FromRequest` trait implementation

        // 2. If the value of the Origin header is not a case-sensitive match for any of the values
        // in list of origins, do not set any additional headers and terminate this set of steps.
        // Always matching is acceptable since the list of origins can be unbounded.

        let response = response.allowed_origin(
            &origin,
            &options.allowed_origins,
            options.send_wildcard,
        )?;

        // 3. If the resource supports credentials add a single Access-Control-Allow-Origin header,
        // with the value of the Origin header as value, and add a
        // single Access-Control-Allow-Credentials header with the case-sensitive string "true" as
        // value.
        // Otherwise, add a single Access-Control-Allow-Origin header,
        // with either the value of the Origin header or the string "*" as value.
        // Note: The string "*" cannot be used for a resource that supports credentials.

        let response = response.credentials(options.allow_credentials)?;

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

    /// Merge a `wrapped` Response with a `cors` response
    ///
    /// If the `wrapped` response has the `Access-Control-Allow-Origin` header already defined,
    /// it will be left untouched. This allows for chaining of several CORS responders.
    ///
    /// Otherwise, the merging will be done according to the rules of `rocket::Response::merge`.
    fn merge(
        mut wrapped: response::Response<'r>,
        cors: response::Response<'r>,
    ) -> response::Response<'r> {

        let existing_cors = {
            wrapped.headers().get("Access-Control-Allow-Origin").next() == None
        };

        if existing_cors {
            wrapped.merge(cors);
        }

        wrapped
    }
}

impl<'r, R: response::Responder<'r>> response::Responder<'r> for Responder<'r, R> {
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

    /// Consumes the responder and based on the provided list of allowed origins,
    /// check if the requested origin is allowed.
    /// Useful for pre-flight and during requests
    fn allowed_origin(
        self,
        origin: &Origin,
        allowed_origins: &AllOrSome<HashSet<Url>>,
        send_wildcard: bool,
    ) -> Result<Self, Error> {
        let origin = origin.origin().unicode_serialization();
        match *allowed_origins {
            // Always matching is acceptable since the list of origins can be unbounded.
            AllOrSome::All => {
                if send_wildcard {
                    Ok(self.any())
                } else {
                    Ok(self.origin(&origin, true))
                }
            }
            AllOrSome::Some(ref allowed_origins) => {
                let allowed_origins: HashSet<_> = allowed_origins
                    .iter()
                    .map(|o| o.origin().unicode_serialization())
                    .collect();
                let _ = allowed_origins.get(&origin).ok_or_else(
                    || Error::OriginNotAllowed,
                )?;
                Ok(self.origin(&origin, false))
            }
        }
    }

    /// Consumes the Response and validate whether credentials can be allowed
    fn credentials(mut self, value: bool) -> Result<Self, Error> {
        if value {
            if let Some(AllOrSome::All) = self.allow_origin {
                Err(Error::CredentialsWithWildcardOrigin)?;
            }
        }

        self.allow_credentials = value;
        Ok(self)
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

    /// Consumes the CORS, check if requested method is allowed.
    /// Useful for pre-flight checks
    fn allowed_methods(
        self,
        method: &AccessControlRequestMethod,
        allowed_methods: &HashSet<Method>,
    ) -> Result<Self, Error> {
        let &AccessControlRequestMethod(ref request_method) = method;
        if !allowed_methods.iter().any(|m| m == request_method) {
            Err(Error::MethodNotAllowed)?
        }

        // TODO: Subset to route? Or just the method requested for?
        Ok(self.methods(&allowed_methods))
    }

    /// Consumes the CORS, set allow_headers to
    /// passed headers and returns changed CORS
    fn headers(mut self, headers: &[&str]) -> Self {
        self.allow_headers = headers.into_iter().map(|s| s.to_string().into()).collect();
        self
    }

    /// Consumes the CORS, check if requested headers are allowed.
    /// Useful for pre-flight checks
    fn allowed_headers(
        self,
        headers: &AccessControlRequestHeaders,
        allowed_headers: &AllOrSome<HashSet<HeaderFieldName>>,
    ) -> Result<Self, Error> {
        let &AccessControlRequestHeaders(ref headers) = headers;

        match *allowed_headers {
            AllOrSome::All => {}
            AllOrSome::Some(ref allowed_headers) => {
                if !headers.is_empty() && !headers.is_subset(allowed_headers) {
                    Err(Error::HeadersNotAllowed)?
                }
            }
        };

        Ok(
            self.headers(
                headers
                    .iter()
                    .map(|s| &**s.deref())
                    .collect::<Vec<&str>>()
                    .as_slice(),
            ),
        )
    }

    /// Builds a `rocket::Response` from this struct containing only the CORS headers.
    #[allow(unused_results)]
    fn build<'r>(&self) -> response::Response<'r> {
        let mut builder = response::Response::build();

        let origin = match self.allow_origin {
            None => {
                // This is not a CORS response
                return builder.finalize();
            }
            Some(ref origin) => origin,
        };

        let origin = match *origin {
            AllOrSome::All => "*".to_string(),
            AllOrSome::Some(ref origin) => origin.to_string(),
        };

        builder.raw_header("Access-Control-Allow-Origin", origin);

        if self.allow_credentials {
            builder.raw_header("Access-Control-Allow-Credentials", "true");
        }

        if !self.expose_headers.is_empty() {
            let headers: Vec<String> = self.expose_headers
                .iter()
                .map(|s| s.deref().to_string())
                .collect();
            let headers = headers.join(", ");

            builder.raw_header("Access-Control-Expose-Headers", headers);
        }

        if !self.allow_headers.is_empty() {
            let headers: Vec<String> = self.allow_headers
                .iter()
                .map(|s| s.deref().to_string())
                .collect();
            let headers = headers.join(", ");

            builder.raw_header("Access-Control-Allow-Headers", headers);
        }

        if !self.allow_methods.is_empty() {
            let methods: Vec<_> = self.allow_methods.iter().map(|m| m.as_str()).collect();
            let methods = methods.join(", ");

            builder.raw_header("Access-Control-Allow-Methods", methods);
        }

        if self.max_age.is_some() {
            let max_age = self.max_age.unwrap();
            builder.raw_header("Access-Control-Max-Age", max_age.to_string());
        }

        if self.vary_origin {
            builder.raw_header("Vary", "Origin");
        }

        builder.finalize()
    }
}

// impl<'r> response::Responder<'r> for Response {
//     fn respond_to(self, request: &Request) -> response::Result<'r> {
//         self.build().respond_to(request)
//     }
// }

#[cfg(test)]
#[allow(unmounted_route)]
mod tests {
    use std::str::FromStr;

    use hyper;
    use rocket;
    use rocket::local::Client;
    use rocket::http::Method;

    use super::*;

    /// Make a client with no routes for unit testing
    fn make_client() -> Client {
        let rocket = rocket::ignite();
        Client::new(rocket).expect("valid rocket instance")
    }

    // The following tests check that CORS Request headers are parsed correctly

    #[test]
    fn origin_header_conversion() {
        let url = "https://foo.bar.xyz";
        let parsed = not_err!(Origin::from_str(url));
        let expected = not_err!(Url::from_str(url));
        assert_eq!(parsed, expected);

        let url = "https://foo.bar.xyz/path/somewhere"; // this should never really be used
        let parsed = not_err!(Origin::from_str(url));
        let expected = not_err!(Url::from_str(url));
        assert_eq!(parsed, expected);

        let url = "invalid_url";
        let _ = is_err!(Origin::from_str(url));
    }

    #[test]
    fn origin_header_parsing() {
        let client = make_client();
        let mut request = client.get("/");

        let origin = hyper::header::Origin::new("https", "www.example.com", None);
        request.add_header(origin);

        let outcome: request::Outcome<Origin, Error> = FromRequest::from_request(request.inner());
        let parsed_header = assert_matches!(outcome, Outcome::Success(s), s);
        assert_eq!("https://www.example.com/", parsed_header.as_str());
    }

    #[test]
    fn request_method_conversion() {
        let method = "POST";
        let parsed_method = not_err!(AccessControlRequestMethod::from_str(method));
        assert_matches!(parsed_method, AccessControlRequestMethod(Method::Post));

        let method = "options";
        let parsed_method = not_err!(AccessControlRequestMethod::from_str(method));
        assert_matches!(parsed_method, AccessControlRequestMethod(Method::Options));

        let method = "INVALID";
        let _ = is_err!(AccessControlRequestMethod::from_str(method));
    }

    #[test]
    fn request_method_parsing() {
        let client = make_client();
        let mut request = client.get("/");
        let method = hyper::header::AccessControlRequestMethod(hyper::method::Method::Get);
        request.add_header(method);
        let outcome: request::Outcome<AccessControlRequestMethod, Error> =
            FromRequest::from_request(request.inner());

        let parsed_header = assert_matches!(outcome, Outcome::Success(s), s);
        let AccessControlRequestMethod(parsed_method) = parsed_header;
        assert_eq!("GET", parsed_method.as_str());
    }

    #[test]
    fn request_headers_conversion() {
        let headers = ["foo", "bar", "baz"];
        let parsed_headers = not_err!(AccessControlRequestHeaders::from_str(&headers.join(", ")));
        let expected_headers: HeaderFieldNamesSet =
            headers.iter().map(|s| s.to_string().into()).collect();
        let AccessControlRequestHeaders(actual_headers) = parsed_headers;
        assert_eq!(actual_headers, expected_headers);
    }

    #[test]
    fn request_headers_parsing() {
        let client = make_client();
        let mut request = client.get("/");
        let headers = hyper::header::AccessControlRequestHeaders(vec![
            FromStr::from_str("accept-language").unwrap(),
            FromStr::from_str("date").unwrap(),
        ]);
        request.add_header(headers);
        let outcome: request::Outcome<AccessControlRequestHeaders, Error> =
            FromRequest::from_request(request.inner());

        let parsed_header = assert_matches!(outcome, Outcome::Success(s), s);
        let AccessControlRequestHeaders(parsed_headers) = parsed_header;
        let mut parsed_headers: Vec<String> =
            parsed_headers.iter().map(|s| s.to_string()).collect();
        parsed_headers.sort();
        assert_eq!(
            vec!["accept-language".to_string(), "date".to_string()],
            parsed_headers
        );
    }

    // The following tests check `Response`'s validation

    #[test]
    fn response_allows_all_origin_with_wildcard() {
        let url = "https://www.example.com";
        let origin = Origin::from_str(url).unwrap();
        let allowed_origins = AllOrSome::All;
        let send_wildcard = true;

        let response = Response::new();
        let response = not_err!(response.allowed_origin(
            &origin,
            &allowed_origins,
            send_wildcard,
        ));

        assert_matches!(response.allow_origin, Some(AllOrSome::All));
        assert_eq!(response.vary_origin, false);

        // Build response and check built response header
        let expected_header = vec!["*"];
        let response = response.build();
        let actual_header: Vec<_> = response
            .headers()
            .get("Access-Control-Allow-Origin")
            .collect();
        assert_eq!(expected_header, actual_header);
    }

    #[test]
    fn response_allows_all_origin_with_echoed_domain() {
        let url = "https://www.example.com";
        let origin = Origin::from_str(url).unwrap();
        let allowed_origins = AllOrSome::All;
        let send_wildcard = false;

        let response = Response::new();
        let response = not_err!(response.allowed_origin(
            &origin,
            &allowed_origins,
            send_wildcard,
        ));

        let actual_origin = assert_matches!(
            response.allow_origin,
            Some(AllOrSome::Some(ref origin)),
            origin
        );
        assert_eq!(url, actual_origin);
        assert_eq!(response.vary_origin, true);

        // Build response and check built response header
        let expected_header = vec![url];
        let response = response.build();
        let actual_header: Vec<_> = response
            .headers()
            .get("Access-Control-Allow-Origin")
            .collect();
        assert_eq!(expected_header, actual_header);
    }

    #[test]
    fn response_allows_origin() {
        let url = "https://www.example.com";
        let origin = Origin::from_str(url).unwrap();
        let (allowed_origins, failed_origins) =
            AllOrSome::new_from_str_list(&["https://www.example.com"]);
        assert!(failed_origins.is_empty());
        let send_wildcard = false;

        let response = Response::new();
        let response = not_err!(response.allowed_origin(
            &origin,
            &allowed_origins,
            send_wildcard,
        ));

        let actual_origin = assert_matches!(
            response.allow_origin,
            Some(AllOrSome::Some(ref origin)),
            origin
        );

        assert_eq!(url, actual_origin);
        assert_eq!(response.vary_origin, false);

        // Build response and check built response header
        let expected_header = vec![url];
        let response = response.build();
        let actual_header: Vec<_> = response
            .headers()
            .get("Access-Control-Allow-Origin")
            .collect();
        assert_eq!(expected_header, actual_header);
    }

    #[test]
    #[should_panic(expected = "OriginNotAllowed")]
    fn response_rejects_invalid_origin() {
        let url = "https://www.acme.com";
        let origin = Origin::from_str(url).unwrap();
        let (allowed_origins, failed_origins) =
            AllOrSome::new_from_str_list(&["https://www.example.com"]);
        assert!(failed_origins.is_empty());
        let send_wildcard = false;

        let response = Response::new();
        let _ = response
            .allowed_origin(&origin, &allowed_origins, send_wildcard)
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "CredentialsWithWildcardOrigin")]
    fn response_credentials_does_not_allow_wildcard_with_all_origins() {
        let response = Response::new();
        let response = response.any();

        let _ = response.credentials(true).unwrap();
    }

    #[test]
    fn response_credentials_allows_specific_origins() {
        let response = Response::new();
        let response = response.origin("https://www.example.com", false);

        let response = response.credentials(true).expect(
            "to allow specific origins",
        );
        assert_eq!(response.allow_credentials, true);

        // Build response and check built response header
        let expected_header = vec!["true"];
        let response = response.build();
        let actual_header: Vec<_> = response
            .headers()
            .get("Access-Control-Allow-Credentials")
            .collect();
        assert_eq!(expected_header, actual_header);
    }

    #[test]
    fn response_sets_exposed_headers_correctly() {
        let headers = vec!["Bar", "Baz", "Foo"];
        let response = Response::new();
        let response = response.origin("https://www.example.com", false);
        let response = response.exposed_headers(&headers);

        // Build response and check built response header
        let response = response.build();
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
        let response = response.build();
        let actual_header: Vec<_> = response.headers().get("Access-Control-Max-Age").collect();
        assert_eq!(expected_header, actual_header);
    }

    #[test]
    fn response_does_not_set_max_age_when_none() {
        let response = Response::new();
        let response = response.origin("https://www.example.com", false);

        let response = response.max_age(None);

        // Build response and check built response header
        let response = response.build();
        assert!(response
            .headers()
            .get("Access-Control-Max-Age")
            .next().is_none())
    }

    /// When all headers are allowed, tests that the requested headers are echoed back
    #[test]
    fn response_allowed_headers_echoes_back_requested_headers() {
        let allowed_headers = AllOrSome::All;
        let requested_headers = vec!["Bar", "Foo"];

        let response = Response::new();
        let response = response.origin("https://www.example.com", false);
        let response = response
            .allowed_headers(
                &FromStr::from_str(&requested_headers.join(",")).unwrap(),
                &allowed_headers,
            )
            .expect("to not fail");

        // Build response and check built response header
        let response = response.build();
        let actual_header: Vec<_> = response
            .headers()
            .get("Access-Control-Allow-Headers")
            .collect();

        assert_eq!(1, actual_header.len());
        let mut actual_headers: Vec<String> = actual_header[0]
            .split(',')
            .map(|header| header.trim().to_string())
            .collect();
        actual_headers.sort();
        assert_eq!(requested_headers, actual_headers);
    }

    #[test]
    fn response_allowed_methods_sets_headers_properly() {
        let allowed_methods = vec![
            Method::Get,
            Method::Head,
            Method::Post,
        ].into_iter()
            .collect();

        let method = "GET";

        let response = Response::new();
        let response = response.origin("https://www.example.com", false);
        let response = response
            .allowed_methods(
                &FromStr::from_str(method).expect("not to fail"),
                &allowed_methods,
            )
            .expect("not to fail");

        // Build response and check built response header
        let response = response.build();
        let actual_header: Vec<_> = response
            .headers()
            .get("Access-Control-Allow-Methods")
            .collect();

        assert_eq!(1, actual_header.len());
        let mut actual_headers: Vec<String> = actual_header[0]
            .split(',')
            .map(|header| header.trim().to_string())
            .collect();
        actual_headers.sort();
        let mut expected_headers: Vec<_> = allowed_methods.iter().map(|m| m.as_str()).collect();
        expected_headers.sort();
        assert_eq!(expected_headers, actual_headers);
    }

    #[test]
    #[should_panic(expected = "MethodNotAllowed")]
    fn response_allowed_method_errors_on_disallowed_method() {
        let allowed_methods = vec![
            Method::Get,
            Method::Head,
            Method::Post,
        ].into_iter()
            .collect();

        let method = "DELETE";

        let response = Response::new();
        let response = response.origin("https://www.example.com", false);
        let _ = response
            .allowed_methods(
                &FromStr::from_str(method).expect("not to fail"),
                &allowed_methods,
            )
            .unwrap();
    }

    /// `Response::allowed_headers` should check that headers are allowed, and only
    /// echoes back the list that is actually requested for and not the whole list
    #[test]
    fn response_allowed_headers_validates_and_echoes_requested_headers() {
        let allowed_headers = vec!["Bar", "Baz", "Foo"];
        let requested_headers = vec!["Bar", "Foo"];

        let response = Response::new();
        let response = response.origin("https://www.example.com", false);
        let response = response
            .allowed_headers(
                &FromStr::from_str(&requested_headers.join(",")).unwrap(),
                &AllOrSome::Some(
                    allowed_headers
                        .iter()
                        .map(|s| FromStr::from_str(*s).unwrap())
                        .collect(),
                ),
            )
            .expect("to not fail");

        // Build response and check built response header
        let response = response.build();
        let actual_header: Vec<_> = response
            .headers()
            .get("Access-Control-Allow-Headers")
            .collect();

        assert_eq!(1, actual_header.len());
        let mut actual_headers: Vec<String> = actual_header[0]
            .split(',')
            .map(|header| header.trim().to_string())
            .collect();
        actual_headers.sort();
        assert_eq!(requested_headers, actual_headers);
    }

    #[test]
    #[should_panic(expected = "HeadersNotAllowed")]
    fn response_allowed_headers_errors_on_non_subset() {
        let allowed_headers = vec!["Bar", "Baz", "Foo"];
        let requested_headers = vec!["Bar", "Foo", "Unknown"];

        let response = Response::new();
        let response = response.origin("https://www.example.com", false);
        let _ = response
            .allowed_headers(
                &FromStr::from_str(&requested_headers.join(",")).unwrap(),
                &AllOrSome::Some(
                    allowed_headers
                        .iter()
                        .map(|s| FromStr::from_str(*s).unwrap())
                        .collect(),
                ),
            )
            .unwrap();

    }

    #[test]
    fn response_does_not_build_if_origin_is_not_set() {
        let response = Response::new();
        let response = response.build();

        let headers: Vec<_> = response.headers().iter().collect();
        assert_eq!(headers.len(), 0);
    }

    // Note: Correct operation of Response::build is tested in the tests above for each of the
    // individual headers

    // Responder tests below

    #[test]
    fn responder_merges_correctly() {
        use std::io::Cursor;
        use rocket::http::Status;

        let wrapped = response::Response::build()
            .status(Status::ImATeapot)
            .raw_header("X-Teapot-Make", "Rocket")
            .sized_body(Cursor::new("Brewing the best coffee!"))
            .finalize();

        let response = Response::new();
        let response = response.origin("https://www.acme.com", false);

        let mut response = Responder::<String>::merge(wrapped, response.build());
        assert_eq!(response.status(), Status::ImATeapot);
        assert_eq!(response.body_string(), Some("Brewing the best coffee!".to_string()));

        // Check CORS header
        let expected_header = vec!["https://www.acme.com"];
        let actual_header: Vec<_> = response
            .headers()
            .get("Access-Control-Allow-Origin")
            .collect();
        assert_eq!(expected_header, actual_header);

        // Check other header
        let expected_header = vec!["Rocket"];
        let actual_header: Vec<_> = response.headers().get("X-Teapot-Make").collect();
        assert_eq!(expected_header, actual_header);
    }

    #[test]
    fn responder_does_not_merge_existing_cors() {
        let wrapped = response::Response::build()
            .raw_header("Access-Control-Allow-Origin", "https://www.example.com")
            .finalize();

        let response = Response::new();
        let response = response.origin("https://www.acme.com", false);

        let response = Responder::<()>::merge(wrapped, response.build());
        let expected_header = vec!["https://www.example.com"];
        let actual_header: Vec<_> = response
            .headers()
            .get("Access-Control-Allow-Origin")
            .collect();
        assert_eq!(expected_header, actual_header);
    }

    // The following tests check that preflight checks are done properly

    // fn make_cors_options() -> Options {
    //     let (allowed_origins, failed_origins) =
    //         AllOrSome::new_from_str_list(&["https://www.acme.com"]);
    //     assert!(failed_origins.is_empty());

    //     Options {
    //         allowed_origins: allowed_origins,
    //         allowed_methods: [Method::Get].iter().cloned().collect(),
    //         allowed_headers: AllOrSome::Some(
    //             ["Authorization"]
    //                 .into_iter()
    //                 .map(|s| s.to_string().into())
    //                 .collect(),
    //         ),
    //         allow_credentials: true,
    //         ..Default::default()
    //     }
    // }

    // /// Tests that non CORS preflight are let through without modification
    // #[test]
    // fn preflight_missing_origins_are_let_through() {
    //     let options = make_cors_options();
    //     let client = make_client();
    //     let request = client.get("/");

    //     let response = options.preflight((), None, None, None).expect("not to fail");

    //     let headers: Vec<_> = response.headers().iter().collect();
    //     assert_eq!(headers.len(), 0);
    // }
}
