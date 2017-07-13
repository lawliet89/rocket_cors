//! Cross-origin resource sharing (CORS) for Rocket.rs applications
//!
//! Rocket (as of v0.2) does not have middleware support. Support for it is (supposedly)
//! on the way. In the mean time, we adopt an
//! [example implementation](https://github.com/SergioBenitez/Rocket/pull/141) to nest
//! `Responders` to acheive the same effect in the short run.
//!
//! # Examples
//! ```
//! #![feature(plugin, custom_derive)]
//! #![plugin(rocket_codegen)]
//! extern crate hyper;
//! extern crate rocket;
//! extern crate rocket_cors;
//!
//! use std::str::FromStr;
//!
//! use rocket::State;
//! use rocket::http::Method::*;
//! use rocket::http::{Header, Status};
//! use rocket::local::Client;
//! use rocket_cors::*;
//!
//! #[options("/")]
//! fn cors_options(origin: Option<Origin>,
//!                 method: AccessControlRequestMethod,
//!                 headers: AccessControlRequestHeaders,
//!                 options: State<rocket_cors::Options>)
//!                 -> Result<Response<()>, Error> {
//!     options.preflight(origin, &method, Some(&headers))
//! }
//!
//! #[get("/")]
//! fn cors(origin: Option<Origin>, options: State<rocket_cors::Options>)
//!         -> Result<Response<&'static str>, Error>
//! {
//!     options.respond("Hello CORS", origin)
//! }
//!
//! # fn main() {
//! let (allowed_origins, failed_origins) =
//!     AllowedOrigins::new_from_str_list(&["https://www.acme.com"]);
//! assert!(failed_origins.is_empty());
//! let cors_options = rocket_cors::Options {
//!     allowed_origins: allowed_origins,
//!     allowed_methods: [Get].iter().cloned().collect(),
//!     allowed_headers: ["Authorization"].iter().map(|s| s.to_string().into()).collect(),
//!     allow_credentials: true,
//!     ..Default::default()
//! };
//! let rocket = rocket::ignite().mount("/", routes![cors, cors_options]).manage(cors_options);
//! let client = Client::new(rocket).unwrap();
//!
//! // `Options` pre-flight checks
//! let origin_header =
//!     Header::from(hyper::header::Origin::from_str("https://www.acme.com").unwrap());
//! let method_header =
//!     Header::from(hyper::header::AccessControlRequestMethod(hyper::method::Method::Get));
//! let request_headers =
//!     hyper::header::AccessControlRequestHeaders(
//!         vec![FromStr::from_str("Authorization").unwrap()]);
//! let request_headers = Header::from(request_headers);
//! let req =
//!     client.options("/").header(origin_header).header(method_header).header(request_headers);
//!
//! let response = req.dispatch();
//! assert_eq!(response.status(), Status::Ok);
//!
//! // "Actual" request
//! let origin_header =
//!     Header::from(hyper::header::Origin::from_str("https://www.acme.com").unwrap());
//! let authorization = Header::new("Authorization", "let me in");
//! let req = client.get("/").header(origin_header).header(authorization);
//!
//! let mut response = req.dispatch();
//! assert_eq!(response.status(), Status::Ok);
//! let body_str = response.body().and_then(|body| body.into_string());
//! assert_eq!(body_str, Some("Hello CORS".to_string()));
//! # }
//! ```


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
use rocket::response::{self, Responder};
use rocket::http::{Method, Status};
use rocket::Outcome;
use unicase::UniCase;

#[cfg(test)]
#[macro_use]
mod test_macros;

/// CORS related error
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

impl<'r> Responder<'r> for Error {
    fn respond_to(self, _: &Request) -> Result<response::Response<'r>, Status> {
        error_!("CORS Error: {:?}", self);
        Err(match self {
            Error::MissingOrigin | Error::OriginNotAllowed | Error::MethodNotAllowed |
            Error::HeadersNotAllowed => Status::Forbidden,
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
            None => Outcome::Failure((Status::BadRequest, Error::MissingRequestMethod)),
        }
    }
}

type HeaderFieldNamesSet = HashSet<UniCase<String>>;

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
            None => Outcome::Failure((Status::BadRequest, Error::MissingRequestHeaders)),
        }
    }
}

/// Origins that are allowed to issue CORS request. This is needed for browser
/// access to the authentication server, but tools like `curl`
/// do not obey nor enforce the CORS convention.
///
/// This enum (de)serialized as an [untagged](https://serde.rs/enum-representations.html)
/// enum variant.
///
/// # Examples
/// ## Allow all origins
/// ```json
/// { "allowed_origins": null }
/// ```
/// ```
/// extern crate rocket_cors;
/// #[macro_use]
/// extern crate serde_derive;
/// extern crate serde_json;
///
/// use rocket_cors::*;
///
/// # fn main() {
/// #[derive(Serialize, Deserialize)]
/// struct Test {
///     allowed_origins: AllowedOrigins
/// }
///
/// let json = r#"{ "allowed_origins": null }"#;
/// let deserialized: Test = serde_json::from_str(json).unwrap();
/// # }
/// ```
/// ## Allow specific origins
///
/// ```json
/// { "allowed_origins": ["http://127.0.0.1:8000/","https://foobar.com/"] }
/// ```
///
/// ```
/// extern crate rocket_cors;
/// #[macro_use]
/// extern crate serde_derive;
/// extern crate serde_json;
///
/// use rocket_cors::*;
///
/// # fn main() {
/// #[derive(Serialize, Deserialize)]
/// struct Test {
///     allowed_origins: AllowedOrigins
/// }
///
/// let json = r#"{ "allowed_origins": ["http://127.0.0.1:8000/","https://foobar.com/"] }"#;
/// let deserialized: Test = serde_json::from_str(json).unwrap();
/// # }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllowedOrigins {
    /// All origins are allowed. Equivalent to the "*" value.
    All,
    /// Only origins listed are allowed.
    Some(HashSet<Url>),
}

impl Default for AllowedOrigins {
    fn default() -> Self {
        AllowedOrigins::All
    }
}

impl AllowedOrigins {
    /// New `AllowedOrigins` from a list of URL strings.
    /// Returns a tuple where the first element is the struct `AllowedOrigins`,
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

        (AllowedOrigins::Some(ok_set), error_map)
    }
}

/// Options to aid in the building of a CORS response during pre-flight or after.
/// See module level documentation for usage examples.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Origins that are allowed to make requests.
    /// Will be verified against the `Origin` request header.
    pub allowed_origins: AllowedOrigins,
    /// Methods that the clients are allowed to request in.
    /// Will be verified against the `Access-Control-Request-Method` request header
    /// during pre-flight only.
    pub allowed_methods: HashSet<Method>,
    /// Headers that the clients are allowed to request in.
    /// Will be verified against the `Access-Control-Request-Headers` request header
    /// during pre-flight only.
    pub allowed_headers: HeaderFieldNamesSet,
    /// The `Access-Control-Allow-Credentials` response header
    pub allow_credentials: bool,
    /// The `Access-Control-Expose-Headers` responde header
    pub expose_headers: HashSet<String>,
    /// The `Access-Control-Max-Age` response header
    pub max_age: Option<usize>,
}

impl Options {
    /// Construct a preflight response based on the options. Will return an `Err`
    /// if any of the preflight checks
    /// fail.
    pub fn preflight(
        &self,
        origin: Option<Origin>,
        method: &AccessControlRequestMethod,
        headers: Option<&AccessControlRequestHeaders>,
    ) -> Result<Response<()>, Error> {


        match origin {
            None => Err(Error::MissingOrigin),
            Some(origin) => {
                let response = Response::<()>::allowed_origin((), &origin, &self.allowed_origins)?
                    .allowed_methods(method, self.allowed_methods.clone())?;

                match headers {
                    Some(headers) => {
                        self.append(response.allowed_headers(headers, &self.allowed_headers))
                    }
                    None => Ok(response),
                }
            }
        }
    }

    /// Respond to a request based on the settings.
    /// If the `Origin` is not provided, then this request was not made by a browser and there is no
    /// CORS enforcement.
    pub fn respond<'r, R: Responder<'r>>(
        &self,
        responder: R,
        origin: Option<Origin>,
    ) -> Result<Response<R>, Error> {
        match origin {
            None => Ok(Response::<R>::any(responder)),
            Some(origin) => {
                self.append(Response::<R>::allowed_origin(
                    responder,
                    &origin,
                    &self.allowed_origins,
                ))
            }
        }
    }

    fn append<'r, R: Responder<'r>>(
        &self,
        response: Result<Response<R>, Error>,
    ) -> Result<Response<R>, Error> {
        Ok(
            response?
                .credentials(self.allow_credentials)
                .exposed_headers(
                    self.expose_headers
                        .iter()
                        .map(|s| &**s)
                        .collect::<Vec<&str>>()
                        .as_slice(),
                )
                .max_age(self.max_age),
        )
    }
}

/// A CORS Response which wraps another struct which implements `Responder`. You will typically
/// use [`Options`] instead to verify and build the response instead of this directly.
/// See module level documentation for usage examples.
pub struct Response<R> {
    responder: R,
    allow_origin: String,
    allow_methods: HashSet<Method>,
    allow_headers: HeaderFieldNamesSet,
    allow_credentials: bool,
    expose_headers: HeaderFieldNamesSet,
    max_age: Option<usize>,
}

impl<'r, R: Responder<'r>> Response<R> {
    /// Consumes the responder and origin and returns basic CORS
    fn origin(responder: R, origin: &str) -> Self {
        Self {
            allow_origin: origin.to_string(),
            allow_headers: HashSet::new(),
            allow_methods: HashSet::new(),
            responder: responder,
            allow_credentials: false,
            expose_headers: HashSet::new(),
            max_age: None,
        }
    }
    /// Consumes the responder and based on the provided list of allowed origins,
    /// check if the requested origin is allowed.
    /// Useful for pre-flight and during requests
    pub fn allowed_origin(
        responder: R,
        origin: &Origin,
        allowed_origins: &AllowedOrigins,
    ) -> Result<Self, Error> {
        match *allowed_origins {
            AllowedOrigins::All => Ok(Self::any(responder)),
            AllowedOrigins::Some(ref allowed_origins) => {
                let origin = origin.origin().unicode_serialization();

                let allowed_origins: HashSet<_> = allowed_origins
                    .iter()
                    .map(|o| o.origin().unicode_serialization())
                    .collect();
                let _ = allowed_origins.get(&origin).ok_or_else(
                    || Error::OriginNotAllowed,
                )?;
                Ok(Self::origin(responder, &origin))
            }
        }
    }

    /// Consumes responder and returns CORS with any origin
    pub fn any(responder: R) -> Self {
        Self::origin(responder, "*")
    }

    /// Consumes the CORS, set allow_credentials to
    /// new value and returns changed CORS
    pub fn credentials(mut self, value: bool) -> Self {
        self.allow_credentials = value;
        self
    }

    /// Consumes the CORS, set expose_headers to
    /// passed headers and returns changed CORS
    pub fn exposed_headers(mut self, headers: &[&str]) -> Self {
        self.expose_headers = headers.into_iter().map(|s| s.to_string().into()).collect();
        self
    }

    /// Consumes the CORS, set max_age to
    /// passed value and returns changed CORS
    pub fn max_age(mut self, value: Option<usize>) -> Self {
        self.max_age = value;
        self
    }

    /// Consumes the CORS, set allow_methods to
    /// passed methods and returns changed CORS
    fn methods(mut self, methods: HashSet<Method>) -> Self {
        self.allow_methods = methods;
        self
    }

    /// Consumes the CORS, check if requested method is allowed.
    /// Useful for pre-flight checks
    pub fn allowed_methods(
        self,
        method: &AccessControlRequestMethod,
        allowed_methods: HashSet<Method>,
    ) -> Result<Self, Error> {
        let &AccessControlRequestMethod(ref request_method) = method;
        if !allowed_methods.iter().any(|m| m == request_method) {
            Err(Error::MethodNotAllowed)?
        }
        Ok(self.methods(allowed_methods))
    }

    /// Consumes the CORS, set allow_headers to
    /// passed headers and returns changed CORS
    fn headers(mut self, headers: &[&str]) -> Self {
        self.allow_headers = headers.into_iter().map(|s| s.to_string().into()).collect();
        self
    }

    /// Consumes the CORS, check if requested headersa are allowed.
    /// Useful for pre-flight checks
    pub fn allowed_headers(
        self,
        headers: &AccessControlRequestHeaders,
        allowed_headers: &HeaderFieldNamesSet,
    ) -> Result<Self, Error> {
        let &AccessControlRequestHeaders(ref headers) = headers;
        if !headers.is_empty() && !headers.is_subset(allowed_headers) {
            Err(Error::HeadersNotAllowed)?
        }
        Ok(
            self.headers(
                allowed_headers
                    .iter()
                    .map(|s| &**s.deref())
                    .collect::<Vec<&str>>()
                    .as_slice(),
            ),
        )
    }
}

impl<'r, R: Responder<'r>> Responder<'r> for Response<R> {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        let mut response = response::Response::build_from(self.responder.respond_to(request)?)
            .raw_header("Access-Control-Allow-Origin", self.allow_origin)
            .finalize();

        if self.allow_credentials {
            response.set_raw_header("Access-Control-Allow-Credentials", "true");
        } else {
            response.set_raw_header("Access-Control-Allow-Credentials", "false");
        }

        if !self.expose_headers.is_empty() {
            let headers: Vec<String> = self.expose_headers
                .into_iter()
                .map(|s| s.deref().to_string())
                .collect();
            let headers = headers.join(", ");

            response.set_raw_header("Access-Control-Expose-Headers", headers);
        }

        if !self.allow_headers.is_empty() {
            let headers: Vec<String> = self.allow_headers
                .into_iter()
                .map(|s| s.deref().to_string())
                .collect();
            let headers = headers.join(", ");

            response.set_raw_header("Access-Control-Allow-Headers", headers);
        }


        if !self.allow_methods.is_empty() {
            let methods: Vec<_> = self.allow_methods.into_iter().map(|m| m.as_str()).collect();
            let methods = methods.join(", ");

            response.set_raw_header("Access-Control-Allow-Methods", methods);
        }

        if self.max_age.is_some() {
            let max_age = self.max_age.unwrap();
            response.set_raw_header("Access-Control-Max-Age", max_age.to_string());
        }

        Ok(response)
    }
}

#[cfg(test)]
#[allow(unmounted_route)]
mod tests {
    use std::str::FromStr;

    use hyper;
    use rocket;
    use rocket::local::Client;
    use rocket::http::Method;
    use rocket::http::{Header, Status};
    use rocket::State;

    use super::*;

    #[test]
    fn origin_header_conversion() {
        let url = "https://foo.bar.xyz";
        let _ = not_err!(Origin::from_str(url));

        let url = "https://foo.bar.xyz/path/somewhere"; // this should never really be used
        let _ = not_err!(Origin::from_str(url));

        let url = "invalid_url";
        let _ = is_err!(Origin::from_str(url));
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
    fn request_headers_conversion() {
        let headers = ["foo", "bar", "baz"];
        let parsed_headers = not_err!(AccessControlRequestHeaders::from_str(&headers.join(", ")));
        let expected_headers: HeaderFieldNamesSet =
            headers.iter().map(|s| s.to_string().into()).collect();
        let AccessControlRequestHeaders(actual_headers) = parsed_headers;
        assert_eq!(actual_headers, expected_headers);
    }

    #[get("/request_headers")]
    #[allow(needless_pass_by_value)]
    fn request_headers(
        origin: Origin,
        method: AccessControlRequestMethod,
        headers: AccessControlRequestHeaders,
    ) -> String {
        let AccessControlRequestMethod(method) = method;
        let AccessControlRequestHeaders(headers) = headers;
        let mut headers = headers
            .iter()
            .map(|s| s.deref().to_string())
            .collect::<Vec<String>>();
        headers.sort();
        format!("{}\n{}\n{}", origin, method, headers.join(", "))
    }

    /// Tests that all the headers are parsed correcly in a HTTP request
    #[test]
    fn request_headers_round_trip_smoke_test() {
        let rocket = rocket::ignite().mount("/", routes![request_headers]);
        let client = not_err!(Client::new(rocket));

        let origin_header = Header::from(not_err!(
            hyper::header::Origin::from_str("https://foo.bar.xyz")
        ));
        let method_header = Header::from(hyper::header::AccessControlRequestMethod(
            hyper::method::Method::Get,
        ));
        let request_headers = hyper::header::AccessControlRequestHeaders(vec![
            FromStr::from_str("accept-language").unwrap(),
            FromStr::from_str("X-Ping").unwrap(),
        ]);
        let request_headers = Header::from(request_headers);
        let req = client
            .get("/request_headers")
            .header(origin_header)
            .header(method_header)
            .header(request_headers);
        let mut response = req.dispatch();

        assert_eq!(Status::Ok, response.status());
        let body_str = not_none!(response.body().and_then(|body| body.into_string()));
        let expected_body = r#"https://foo.bar.xyz/
GET
X-Ping, accept-language"#;
        assert_eq!(expected_body, body_str);
    }

    #[get("/any")]
    #[cfg_attr(feature = "clippy_lints", allow(needless_pass_by_value))]
    fn any() -> Response<&'static str> {
        Response::any("Hello, world!")
    }

    #[test]
    fn response_any_origin_smoke_test() {
        let rocket = rocket::ignite().mount("/", routes![any]);
        let client = not_err!(Client::new(rocket));

        let req = client.get("/any");
        let mut response = req.dispatch();

        assert_eq!(Status::Ok, response.status());
        let body_str = response.body().and_then(|body| body.into_string());
        let values: Vec<_> = response
            .headers()
            .get("Access-Control-Allow-Origin")
            .collect();
        assert_eq!(values, vec!["*"]);
        assert_eq!(body_str, Some("Hello, world!".to_string()));
    }

    #[options("/")]
    #[allow(needless_pass_by_value)]
    fn cors_options(
        origin: Option<Origin>,
        method: AccessControlRequestMethod,
        headers: AccessControlRequestHeaders,
        options: State<Options>,
    ) -> Result<Response<()>, Error> {
        options.preflight(origin, &method, Some(&headers))
    }

    #[get("/")]
    #[allow(needless_pass_by_value)]
    fn cors(
        origin: Option<Origin>,
        options: State<Options>,
    ) -> Result<Response<&'static str>, Error> {
        options.respond("Hello CORS", origin)
    }

    fn make_cors_options() -> Options {
        let (allowed_origins, failed_origins) =
            AllowedOrigins::new_from_str_list(&["https://www.acme.com"]);
        assert!(failed_origins.is_empty());

        Options {
            allowed_origins: allowed_origins,
            allowed_methods: [Method::Get].iter().cloned().collect(),
            allowed_headers: ["Authorization"]
                .iter()
                .map(|s| s.to_string().into())
                .collect(),
            allow_credentials: true,
            ..Default::default()
        }
    }

    #[test]
    fn cors_options_check() {
        let rocket = rocket::ignite()
            .mount("/", routes![cors, cors_options])
            .manage(make_cors_options());
        let client = not_err!(Client::new(rocket));

        let origin_header = Header::from(not_err!(
            hyper::header::Origin::from_str("https://www.acme.com")
        ));
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
        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    fn cors_get_check() {
        let rocket = rocket::ignite()
            .mount("/", routes![cors, cors_options])
            .manage(make_cors_options());
        let client = not_err!(Client::new(rocket));

        let origin_header = Header::from(not_err!(
            hyper::header::Origin::from_str("https://www.acme.com")
        ));
        let authorization = Header::new("Authorization", "let me in");
        let req = client.get("/").header(origin_header).header(authorization);

        let mut response = req.dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body_str = response.body().and_then(|body| body.into_string());
        assert_eq!(body_str, Some("Hello CORS".to_string()));
    }

    /// This test is to check that non CORS compliant requests to GET should still work. (i.e. curl)
    #[test]
    fn cors_get_no_origin() {
        let rocket = rocket::ignite()
            .mount("/", routes![cors, cors_options])
            .manage(make_cors_options());
        let client = not_err!(Client::new(rocket));

        let authorization = Header::new("Authorization", "let me in");
        let req = client.get("/").header(authorization);

        let mut response = req.dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body_str = response.body().and_then(|body| body.into_string());
        assert_eq!(body_str, Some("Hello CORS".to_string()));
    }

    #[test]
    fn cors_options_bad_origin() {
        let rocket = rocket::ignite()
            .mount("/", routes![cors, cors_options])
            .manage(make_cors_options());
        let client = not_err!(Client::new(rocket));

        let origin_header = Header::from(not_err!(hyper::header::Origin::from_str(
            "https://www.bad-origin.com",
        )));
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
        let client = not_err!(Client::new(rocket));

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
        assert_eq!(response.status(), Status::Forbidden);
    }

    #[test]
    fn cors_options_bad_request_method() {
        let rocket = rocket::ignite()
            .mount("/", routes![cors, cors_options])
            .manage(make_cors_options());
        let client = not_err!(Client::new(rocket));

        let origin_header = Header::from(not_err!(
            hyper::header::Origin::from_str("https://www.acme.com")
        ));
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
        let client = not_err!(Client::new(rocket));

        let origin_header = Header::from(not_err!(
            hyper::header::Origin::from_str("https://www.acme.com")
        ));
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
        let client = not_err!(Client::new(rocket));

        let origin_header = Header::from(not_err!(hyper::header::Origin::from_str(
            "https://www.bad-origin.com",
        )));
        let authorization = Header::new("Authorization", "let me in");
        let req = client.get("/").header(origin_header).header(authorization);

        let response = req.dispatch();
        assert_eq!(response.status(), Status::Forbidden);
    }
}
