//! [![Build Status](https://travis-ci.org/lawliet89/rocket_cors.svg)](https://travis-ci.org/lawliet89/rocket_cors)
//! [![Dependency Status](https://dependencyci.com/github/lawliet89/rocket_cors/badge)](https://dependencyci.com/github/lawliet89/rocket_cors)
//! [![Repository](https://img.shields.io/github/tag/lawliet89/rocket_cors.svg)](https://github.com/lawliet89/rocket_cors)
//! [![Crates.io](https://img.shields.io/crates/v/rocket_cors.svg)](https://crates.io/crates/rocket_cors)
//!
//! - Documentation:   [master branch](https://lawliet89.github.io/rocket_cors)
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
//! In particular, `rocket_cors` is currently targetted for `nightly-2017-07-21`. Newer nightlies
//! might work, but it's not guaranteed.
//!
//! ## Installation
//!
//! Add the following to Cargo.toml:
//!
//! ```toml
//! rocket_cors = "0.2.0"
//! ```
//!
//! To use the latest `master` branch, for example:
//!
//! ```toml
//! rocket_cors = { git = "https://github.com/lawliet89/rocket_cors", branch = "master" }
//! ```
//!
//! ## Features
//!
//! By default, a `serialization` feature is enabled in this crate that allows you to (de)serialize
//! the `Cors` struct that is described below. If you would like to disable this, simply change
//! your `Cargo.toml` to:
//!
//! ```toml
//! rocket_cors = { version = "0.2.0", default-features = false }
//! ```
//!
//! ## Usage
//!
//! Before you can add CORS responses to your application, you need to create a `Cors` struct that
//! will hold the settings.
//!
//! Each of the examples can be run off the repository via `cargo run --example xxx` where `xxx` is
//!
//! - `fairing`
//! - `guard`
//! - `manual`
//!
//! ### `Cors` Struct
//!
//! The [`Cors` struct](struct.Cors.html) contains the settings for CORS requests to be validated
//! and for responses to be generated. Defaults are defined for every field in the struct, and
//! are documented on the [`Cors` struct](struct.Cors.html) page. You can also deserialize
//! the struct from some format like JSON, YAML or TOML when the default `serialization` feature
//! is enabled.
//!
//! ### Three modes of operation
//!
//! You can add CORS to your routes via one of three ways, in descending order of ease and in
//! ascending order of flexibility.
//!
//! - Fairing (should only used exclusively)
//! - Request Guard
//! - Truly Manual
//!
//! Unfortunately, you cannot mix and match Fairing with any other of the methods, due to the
//! limitation of Rocket's fairing API. That is, the checks for Fairing will always happen first,
//! and if they fail, the route is never executed and so your guard or manual checks will never
//! get executed.
//!
//! You can, however, mix and match guards and manual checks.
//!
//! In summary:
//!
//! |                                         | Fairing | Request Guard | Manual |
//! |:---------------------------------------:|:-------:|:-------------:|:------:|
//! |         Must apply to all routes        |    ✔    |       ✗       |    ✗   |
//! | Different settings for different routes |    ✗    |       ✗       |    ✔   |
//! |     May define custom OPTIONS routes    |    ✗    |       ✔       |    ✔   |
//!
//! ### Fairing
//!
//! Fairing is the easiest to use and also the most inflexible. You don't have to define `OPTIONS`
//! routes for your application, and the checks are done transparently.
//!
//! However, you can only have one set of settings that must apply to all routes. You cannot opt
//! any route out of CORS checks.
//!
//! To use this, simply create a [`Cors` struct](struct.Cors.html) and then
//! [`attach`](https://api.rocket.rs/rocket/struct.Rocket.html#method.attach) it to Rocket.
//!
//! ```rust,no_run
//! #![feature(plugin, custom_derive)]
//! #![plugin(rocket_codegen)]
//! extern crate rocket;
//! extern crate rocket_cors;
//!
//! use rocket::http::Method;
//! use rocket_cors::{AllowedOrigins, AllowedHeaders};
//!
//! #[get("/")]
//! fn cors<'a>() -> &'a str {
//!     "Hello CORS"
//! }
//!
//! fn main() {
//!     let (allowed_origins, failed_origins) = AllowedOrigins::some(&["https://www.acme.com"]);
//!     assert!(failed_origins.is_empty());
//!
//!     // You can also deserialize this
//!     let options = rocket_cors::Cors {
//!         allowed_origins: allowed_origins,
//!         allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
//!         allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
//!         allow_credentials: true,
//!         ..Default::default()
//!     };
//!
//!     rocket::ignite()
//!         .mount("/", routes![cors])
//!         .attach(options)
//!         .launch();
//! }
//!
//! ```
//!
//! ### Request Guard
//!
//! Using request guard requires you to sacrifice the convenience of Fairings for being able to
//! opt some routes out of CORS checks and enforcement. _BUT_ you are still restricted to only
//! one set of CORS settings and you have to mount additional routes to catch and process OPTIONS
//! requests. The `OPTIONS` routes are used for CORS preflight checks.
//!
//! You will have to do the following:
//!
//! - Create a [`Cors` struct](struct.Cors.html) and during Rocket's ignite, add the struct to
//! Rocket's [managed state](https://rocket.rs/guide/state/#managed-state).
//! - For all the routes that you want to enforce CORS on, you can mount either some
//! [catch all route](fn.catch_all_options_routes.html) or define your own route for the OPTIONS
//! verb.
//! - Then in all the routes you want to enforce CORS on, add a
//! [Request Guard](https://rocket.rs/guide/requests/#request-guards) for the
//! [`Guard`](struct.Guard.html) struct in the route arguments. You should not wrap this in an
//! `Option` or `Result` because the guard will let non-CORS requests through and will take over
//! error handling in case of errors.
//! - In your routes, to add CORS headers to your responses, use the appropriate functions on the
//! [`Guard`](struct.Guard.html) for a `Response` or a `Responder`.
//!
//! ```rust,no_run
//! #![feature(plugin)]
//! #![plugin(rocket_codegen)]
//! extern crate rocket;
//! extern crate rocket_cors;
//!
//! use std::io::Cursor;
//!
//! use rocket::Response;
//! use rocket::http::Method;
//! use rocket_cors::{Guard, AllowedOrigins, AllowedHeaders, Responder};
//!
//! /// Using a `Responder` -- the usual way you would use this
//! #[get("/")]
//! fn responder(cors: Guard) -> Responder<&str> {
//!     cors.responder("Hello CORS!")
//! }
//!
//! /// Using a `Response` instead of a `Responder`. You generally won't have to do this.
//! #[get("/response")]
//! fn response(cors: Guard) -> Response {
//!     let mut response = Response::new();
//!     response.set_sized_body(Cursor::new("Hello CORS!"));
//!     cors.response(response)
//! }
//!
//! /// Manually mount an OPTIONS route for your own handling
//! #[options("/manual")]
//! fn manual_options(cors: Guard) -> Responder<&str> {
//!     cors.responder("Manual OPTIONS preflight handling")
//! }
//!
//! /// Manually mount an OPTIONS route for your own handling
//! #[get("/manual")]
//! fn manual(cors: Guard) -> Responder<&str> {
//!     cors.responder("Manual OPTIONS preflight handling")
//! }
//!
//! fn main() {
//!     let (allowed_origins, failed_origins) = AllowedOrigins::some(&["https://www.acme.com"]);
//!     assert!(failed_origins.is_empty());
//!
//!     // You can also deserialize this
//!     let options = rocket_cors::Cors {
//!         allowed_origins: allowed_origins,
//!         allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
//!         allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
//!         allow_credentials: true,
//!         ..Default::default()
//!     };
//!
//!     rocket::ignite()
//!         .mount(
//!             "/",
//!             routes![responder, response],
//!         )
//!         // Mount the routes to catch all the OPTIONS pre-flight requests
//!         .mount("/", rocket_cors::catch_all_options_routes())
//!         // You can also manually mount an OPTIONS route that will be used instead
//!         .mount("/", routes![manual, manual_options])
//!         .manage(options)
//!         .launch();
//! }
//! ```
//!
//! ## Truly Manual
//!
//! This mode is the most difficult to use but offers the most amount of flexibility.
//! You might have to understand how the library works internally to know how to use this mode.
//! In exchange, you can selectively choose which routes to offer CORS protection to, and you
//! can mix and match CORS settings for the routes. You can combine usage of this mode with
//! "guard" to offer a mix of ease of use and flexibility.
//!
//! You really do not need to use this unless you have a truly ad-hoc need to respond to CORS
//! differently in a route. For example, you have a `ping` endpoint that allows all origins but
//! the rest of your routes do not.
//!
//! ### Handler
//!
//! This mode requires that you pass in a closure that will be lazily evaluated once a CORS request
//! has been validated. If validation fails, the closure will not be run. You should put any code
//! that has any side effects or with an appreciable computation cost inside this handler.
//!
//! ### Steps to perform:
//! - Your crate will need to enable the
//! [`conservative_impl_trait`](https://github.com/rust-lang/rfcs/blob/master/text/1522-conservative-impl-trait.md)
//! feature. You can use `#![feature(conservative_impl_trait)]` at your crate root.
//! Otherwise, the return type of your routes will be unspecifiable.
//! - You will first need to have a `Cors` struct ready. This struct can be borrowed with a lifetime
//! at least as long as `'r` which is the lifetime of a Rocket request. `'static` works too.
//! In this case, you might as well use the `Guard` method above and place the `Cors` struct in
//! Rocket's [state](https://rocket.rs/guide/state/).
//! Alternatively, you can create a `Cors` struct directly in the route.
//! - Your routes will need to have a `'r` lifetime and return `impl Responder<'r>`.
//! - Using the `Cors` struct, use either the
//! [`respond_owned`](struct.Cors.html#method.respond_owned) or
//! [`respond_borrowed`](struct.Cors.html#method.respond_borrowed) function and pass in a handler
//! that will be executed once CORS validation is successful.
//! - Your handler will be passed a [`Guard`](struct.Guard.html) which you will have to use to
//! add CORS headers into your own response.
//! - You will have to manually define your own `OPTIONS` routes.
//!
//! ### Notes about route lifetime
//! It is unfortunate that you have to manually specify the `'r` lifetimes in your routes.
//! Leaving out the lifetime will result in a
//! [compiler panic](https://github.com/rust-lang/rust/issues/43380). Even if the panic is fixed,
//! it is not known if we can exclude the lifetime because lifetimes are _elided_ in Rust,
//! not inferred.
//!
//! ### Owned example
//! This is the most likely scenario when you want to have manual CORS validation. You can use this
//! when the settings you want to use for a route is not the same as the rest of the application
//! (which you might have put in Rocket's state).
//!
//! ```rust,no_run
//! #![feature(plugin, conservative_impl_trait)]
//! #![plugin(rocket_codegen)]
//! extern crate rocket;
//! extern crate rocket_cors;
//!
//! use rocket::http::Method;
//! use rocket::response::Responder;
//! use rocket_cors::{Cors, AllowedOrigins, AllowedHeaders};
//!
//! /// Create and use an ad-hoc Cors
//! #[get("/owned")]
//! fn owned<'r>() -> impl Responder<'r> {
//!     let options = cors_options();
//!     options.respond_owned(|guard| guard.responder("Hello CORS"))
//! }
//!
//! /// You need to define an OPTIONS route for preflight checks.
//! /// These routes can just return the unit type `()`
//! #[options("/owned")]
//! fn owned_options<'r>() -> impl Responder<'r> {
//!     let options = cors_options();
//!     options.respond_owned(|guard| guard.responder(()))
//! }
//!
//! fn cors_options() -> Cors {
//!     let (allowed_origins, failed_origins) = AllowedOrigins::some(&["https://www.acme.com"]);
//!     assert!(failed_origins.is_empty());
//!
//!     // You can also deserialize this
//!     rocket_cors::Cors {
//!         allowed_origins: allowed_origins,
//!         allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
//!         allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
//!         allow_credentials: true,
//!         ..Default::default()
//!     }
//! }
//!
//! fn main() {
//!     rocket::ignite()
//!         .mount(
//!             "/",
//!             routes![
//!                 owned,
//!                 owned_options,
//!             ],
//!         )
//!         .manage(cors_options())
//!         .launch();
//! }
//! ```
//!
//! ### Borrowed Example
//! You might want to borrow the `Cors` struct from Rocket's state, for example. Unless you have
//! special handling, you might want to use the Guard method instead which has less hassle.
//!
//! ```rust,no_run
//! #![feature(plugin, conservative_impl_trait)]
//! #![plugin(rocket_codegen)]
//! extern crate rocket;
//! extern crate rocket_cors;
//!
//! use std::io::Cursor;
//!
//! use rocket::{State, Response};
//! use rocket::http::Method;
//! use rocket::response::Responder;
//! use rocket_cors::{Cors, AllowedOrigins, AllowedHeaders};
//!
//! /// Using a borrowed Cors
//! #[get("/")]
//! fn borrowed<'r>(options: State<'r, Cors>) -> impl Responder<'r> {
//!     options.inner().respond_borrowed(
//!         |guard| guard.responder("Hello CORS"),
//!     )
//! }
//!
//! /// Using a `Response` instead of a `Responder`. You generally won't have to do this.
//! #[get("/response")]
//! fn response<'r>(options: State<'r, Cors>) -> impl Responder<'r> {
//!     let mut response = Response::new();
//!     response.set_sized_body(Cursor::new("Hello CORS!"));
//!
//!     options.inner().respond_borrowed(
//!         move |guard| guard.response(response),
//!     )
//! }
//!
//! fn cors_options() -> Cors {
//!     let (allowed_origins, failed_origins) = AllowedOrigins::some(&["https://www.acme.com"]);
//!     assert!(failed_origins.is_empty());
//!
//!     // You can also deserialize this
//!     rocket_cors::Cors {
//!         allowed_origins: allowed_origins,
//!         allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
//!         allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
//!         allow_credentials: true,
//!         ..Default::default()
//!     }
//! }
//!
//! fn main() {
//!     rocket::ignite()
//!         .mount(
//!             "/",
//!             routes![
//!                 borrowed,
//!                 response,
//!             ],
//!         )
//!         .mount("/", rocket_cors::catch_all_options_routes()) // mount the catch all routes
//!         .manage(cors_options())
//!         .launch();
//! }
//! ```
//!
//! ## Mixing Guard and Manual
//!
//! You can mix `Guard` and `Truly Manual` modes together for your application. For example, your
//! application might restrict the Origins that can access it, except for one `ping` route that
//! allows all access.
//!
//! You can run the example code below with `cargo run --example mix`.
//!
//! ```rust,no_run
//! #![feature(plugin, conservative_impl_trait)]
//! #![plugin(rocket_codegen)]
//! extern crate rocket;
//! extern crate rocket_cors;
//!
//! use rocket::http::Method;
//! use rocket::response::Responder;
//! use rocket_cors::{Cors, Guard, AllowedOrigins, AllowedHeaders};
//!
//! /// The "usual" app route
//! #[get("/")]
//! fn app(cors: Guard) -> rocket_cors::Responder<&str> {
//!     cors.responder("Hello CORS!")
//! }
//!
//! /// The special "ping" route
//! #[get("/ping")]
//! fn ping<'r>() -> impl Responder<'r> {
//!     let options = cors_options_all();
//!     options.respond_owned(|guard| guard.responder("Pong!"))
//! }
//!
//! /// You need to define an OPTIONS route for preflight checks if you want to use `Cors` struct
//! /// that is not in Rocket's managed state.
//! /// These routes can just return the unit type `()`
//! #[options("/ping")]
//! fn ping_options<'r>() -> impl Responder<'r> {
//!     let options = cors_options_all();
//!     options.respond_owned(|guard| guard.responder(()))
//! }
//!
//! /// Returns the "application wide" Cors struct
//! fn cors_options() -> Cors {
//!     let (allowed_origins, failed_origins) = AllowedOrigins::some(&["https://www.acme.com"]);
//!     assert!(failed_origins.is_empty());
//!
//!     // You can also deserialize this
//!     rocket_cors::Cors {
//!         allowed_origins: allowed_origins,
//!         allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
//!         allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
//!         allow_credentials: true,
//!         ..Default::default()
//!     }
//! }
//!
//! /// A special struct that allows all origins
//! ///
//! /// Note: In your real application, you might want to use something like `lazy_static` to generate
//! /// a `&'static` reference to this instead of creating a new struct on every request.
//! fn cors_options_all() -> Cors {
//!     // You can also deserialize this
//!     rocket_cors::Cors {
//!         allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
//!         ..Default::default()
//!     }
//! }
//!
//! fn main() {
//!     rocket::ignite()
//!         .mount(
//!             "/",
//!             routes![
//!                 app,
//!                 ping,
//!                 ping_options,
//!             ],
//!         )
//!         .mount("/", rocket_cors::catch_all_options_routes()) // mount the catch all routes
//!         .manage(cors_options())
//!         .launch();
//! }
//!
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

#![cfg_attr(test, feature(plugin))]
#![cfg_attr(test, plugin(rocket_codegen))]
#![doc(test(attr(allow(unused_variables), deny(warnings))))]

#[macro_use]
extern crate log;
#[macro_use]
extern crate rocket;
extern crate unicase;
extern crate url;

#[cfg(feature = "serialization")]
extern crate serde;
#[cfg(feature = "serialization")]
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "serialization")]
extern crate unicase_serde;
#[cfg(feature = "serialization")]
extern crate url_serde;

#[cfg(test)]
extern crate hyper;
#[cfg(feature = "serialization")]
#[cfg(test)]
extern crate serde_test;
#[cfg(feature = "serialization")]
#[cfg(test)]
extern crate serde_json;

#[cfg(test)]
#[macro_use]
mod test_macros;
mod fairing;

pub mod headers;

use std::borrow::Cow;
use std::collections::{HashSet, HashMap};
use std::error;
use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;
use std::str::FromStr;

use rocket::{Outcome, State};
use rocket::http::{self, Status};
use rocket::request::{Request, FromRequest};
use rocket::response;

use headers::{HeaderFieldName, HeaderFieldNamesSet, Origin, AccessControlRequestHeaders,
              AccessControlRequestMethod, Url};

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
    /// A CORS Request Guard was used, but no CORS Options was available in Rocket's state
    ///
    /// This is a misconfiguration. Use `Rocket::manage` to add a CORS options to managed state.
    MissingCorsInRocketState,
    /// The `on_response` handler of Fairing could not find the injected header from the Request.
    /// Either some other fairing has removed it, or this is a bug.
    MissingInjectedHeader,
    /// The `on_response` handler of Fairing found an unknown injected header value from the
    /// Request. Either some other fairing has modified it, or this is a bug.
    UnknownInjectedHeader,
}

impl Error {
    fn status(&self) -> Status {
        match *self {
            Error::MissingOrigin | Error::OriginNotAllowed | Error::MethodNotAllowed |
            Error::HeadersNotAllowed => Status::Forbidden,
            Error::CredentialsWithWildcardOrigin |
            Error::MissingCorsInRocketState |
            Error::MissingInjectedHeader |
            Error::UnknownInjectedHeader => Status::InternalServerError,
            _ => Status::BadRequest,
        }
    }
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
            Error::MissingCorsInRocketState => {
                "A CORS Request Guard was used, but no CORS Options was available in Rocket's state"
            }
            Error::MissingInjectedHeader => {
                "The `on_response` handler of Fairing could not find the injected header from the \
                 Request. Either some other fairing has removed it, or this is a bug."
            }
            Error::UnknownInjectedHeader => {
                "The `on_response` handler of Fairing found an unknown injected header value from \
                 the Request. Either some other fairing has modified it, or this is a bug."
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
        error_!("CORS Error: {}", self);
        Err(self.status())
    }
}

/// An enum signifying that some of type T is allowed, or `All` (everything is allowed).
///
/// `Default` is implemented for this enum and is `All`.
///
/// This enum is serialized and deserialized
/// ["Externally tagged"](https://serde.rs/enum-representations.html)
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
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
    #[deprecated(since = "0.1.3", note = "please use `AllowedOrigins::Some` instead")]
    /// New `AllOrSome` from a list of URL strings.
    /// Returns a tuple where the first element is the struct `AllOrSome`,
    /// and the second element
    /// is a map of strings which failed to parse into URLs and their associated parse errors.
    pub fn new_from_str_list(urls: &[&str]) -> (Self, HashMap<String, url::ParseError>) {
        AllowedOrigins::some(urls)
    }
}

/// A wrapper type around `rocket::http::Method` to support serialization and deserialization
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Method(http::Method);

impl FromStr for Method {
    type Err = rocket::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let method = http::Method::from_str(s)?;
        Ok(Method(method))
    }
}

impl Deref for Method {
    type Target = http::Method;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<http::Method> for Method {
    fn from(method: http::Method) -> Self {
        Method(method)
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(feature = "serialization")]
mod method_serde {
    use std::fmt;
    use std::str::FromStr;

    use serde::{self, Serialize, Deserialize};

    use Method;

    impl Serialize for Method {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(self.as_str())
        }
    }

    impl<'de> Deserialize<'de> for Method {
        fn deserialize<D>(deserializer: D) -> Result<Method, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use serde::de::{self, Visitor};

            struct MethodVisitor;
            impl<'de> Visitor<'de> for MethodVisitor {
                type Value = Method;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("a string containing a HTTP Verb")
                }

                fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    match Self::Value::from_str(s) {
                        Ok(value) => Ok(value),
                        Err(e) => Err(de::Error::custom(format!("{:?}", e))),
                    }
                }
            }

            deserializer.deserialize_string(MethodVisitor)
        }
    }
}

/// A list of allowed origins. Either Some origins are allowed, or all origins are allowed.
///
/// # Examples
/// ```rust
/// use rocket_cors::AllowedOrigins;
///
/// let all_origins = AllowedOrigins::all();
/// let (some_origins, failed_origins) = AllowedOrigins::some(&["https://www.acme.com"]);
/// assert!(failed_origins.is_empty());
/// ```
pub type AllowedOrigins = AllOrSome<HashSet<Url>>;

impl AllowedOrigins {
    /// Allows some origins
    ///
    /// Returns a tuple where the first element is the struct `AllowedOrigins`,
    /// and the second element
    /// is a map of strings which failed to parse into URLs and their associated parse errors.
    pub fn some(urls: &[&str]) -> (Self, HashMap<String, url::ParseError>) {
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

    /// Allows all origins
    pub fn all() -> Self {
        AllOrSome::All
    }
}

/// A list of allowed methods
///
/// The [list](https://api.rocket.rs/rocket/http/enum.Method.html)
/// of methods is whatever is supported by Rocket.
///
/// # Example
/// ```rust
/// use std::str::FromStr;
/// use rocket_cors::AllowedMethods;
///
/// let allowed_methods: AllowedMethods = ["Get", "Post", "Delete"]
///    .iter()
///    .map(|s| FromStr::from_str(s).unwrap())
///    .collect();
/// ```
pub type AllowedMethods = HashSet<Method>;

/// A list of allowed headers
///
/// # Examples
/// ```rust
/// use rocket_cors::AllowedHeaders;
///
/// let all_headers = AllowedHeaders::all();
/// let some_headers = AllowedHeaders::some(&["Authorization", "Accept"]);
/// ```
pub type AllowedHeaders = AllOrSome<HashSet<HeaderFieldName>>;

impl AllowedHeaders {
    /// Allow some headers
    pub fn some(headers: &[&str]) -> Self {
        AllOrSome::Some(headers.iter().map(|s| s.to_string().into()).collect())
    }

    /// Allows all headers
    pub fn all() -> Self {
        AllOrSome::All
    }
}

/// Response generator and [Fairing](https://rocket.rs/guide/fairings/) for CORS
///
/// This struct can be as Fairing or in an ad-hoc manner to generate CORS response. See the
/// documentation at the [crate root](index.html) for usage information.
///
/// You create a new copy of this struct by defining the configurations in the fields below.
/// This struct can also be deserialized by serde with the `serialization` feature which is
/// enabled by default.
///
/// [`Default`](https://doc.rust-lang.org/std/default/trait.Default.html) is implemented for this
/// struct. The default for each field is described in the docuementation for the field.
///
/// # Examples
///
/// You can run an example from the repository to demonstrate the JSON serialization with
/// `cargo run --example json`.
///
/// ## Pure default
/// ```rust
/// let default = rocket_cors::Cors::default();
/// ```
///
/// ## JSON Examples
/// ### Default
///
/// ```json
/// {
///   "allowed_origins": "All",
///   "allowed_methods": [
///     "POST",
///     "PATCH",
///     "PUT",
///     "DELETE",
///     "HEAD",
///     "OPTIONS",
///     "GET"
///   ],
///   "allowed_headers": "All",
///   "allow_credentials": false,
///   "expose_headers": [],
///   "max_age": null,
///   "send_wildcard": false,
///   "fairing_route_base": "/cors"
/// }
/// ```
/// ### Defined
/// ```json
/// {
///   "allowed_origins": {
///     "Some": [
///       "https://www.acme.com/"
///     ]
///   },
///   "allowed_methods": [
///     "POST",
///     "DELETE",
///     "GET"
///   ],
///   "allowed_headers": {
///     "Some": [
///       "Accept",
///       "Authorization"
///     ]
///   },
///   "allow_credentials": true,
///   "expose_headers": [
///     "Content-Type",
///     "X-Custom"
///   ],
///   "max_age": 42,
///   "send_wildcard": false,
///   "fairing_route_base": "/mycors"
/// }
///
/// ```
#[derive(Eq, PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
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
    #[cfg_attr(feature = "serialization", serde(default))]
    pub allowed_origins: AllowedOrigins,
    /// The list of methods which the allowed origins are allowed to access for
    /// non-simple requests.
    ///
    /// This is the `list of methods` in the
    /// [Resource Processing Model](https://www.w3.org/TR/cors/#resource-processing-model).
    ///
    /// Defaults to `[GET, HEAD, POST, OPTIONS, PUT, PATCH, DELETE]`
    #[cfg_attr(feature = "serialization", serde(default = "Cors::default_allowed_methods"))]
    pub allowed_methods: AllowedMethods,
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
    #[cfg_attr(feature = "serialization", serde(default))]
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
    #[cfg_attr(feature = "serialization", serde(default))]
    pub allow_credentials: bool,
    /// The list of headers which are safe to expose to the API of a CORS API specification.
    /// This corresponds to the `Access-Control-Expose-Headers` responde header.
    ///
    /// This is the `list of exposed headers` in the
    /// [Resource Processing Model](https://www.w3.org/TR/cors/#resource-processing-model).
    ///
    /// This defaults to an empty set.
    #[cfg_attr(feature = "serialization", serde(default))]
    pub expose_headers: HashSet<String>,
    /// The maximum time for which this CORS request maybe cached. This value is set as the
    /// `Access-Control-Max-Age` header.
    ///
    /// This defaults to `None` (unset).
    #[cfg_attr(feature = "serialization", serde(default))]
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
    #[cfg_attr(feature = "serialization", serde(default))]
    pub send_wildcard: bool,
    /// When used as Fairing, Cors will need to redirect failed CORS checks to a custom route to
    /// be mounted by the fairing. Specify the base the route so that it doesn't clash with any
    /// of your existing routes.
    ///
    /// Defaults to "/cors"
    #[cfg_attr(feature = "serialization", serde(default = "Cors::default_fairing_route_base"))]
    pub fairing_route_base: String,
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
            fairing_route_base: Self::default_fairing_route_base(),
        }
    }
}

impl Cors {
    fn default_allowed_methods() -> HashSet<Method> {
        use rocket::http::Method;

        vec![
            Method::Get,
            Method::Head,
            Method::Post,
            Method::Options,
            Method::Put,
            Method::Patch,
            Method::Delete,
        ].into_iter()
            .map(From::from)
            .collect()
    }

    fn default_fairing_route_base() -> String {
        "/cors".to_string()
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

    /// Manually respond to a request with CORS checks and headers using an Owned `Cors`.
    ///
    /// Use this variant when your `Cors` struct will not live at least as long as the whole `'r`
    /// lifetime of the request.
    ///
    /// After the CORS checks are done, the passed in handler closure will be run to generate a
    /// final response. You will have to merge your response with the `Guard` that you have been
    /// passed in to include the CORS headers.
    ///
    /// See the documentation at the [crate root](index.html) for usage information.
    pub fn respond_owned<'r, F, R>(self, handler: F) -> Result<ManualResponder<'r, F, R>, Error>
    where
        F: FnOnce(Guard<'r>) -> R + 'r,
        R: response::Responder<'r>,
    {
        self.validate()?;
        Ok(ManualResponder::new(Cow::Owned(self), handler))
    }

    /// Manually respond to a request with CORS checks and headers using a borrowed `Cors`.
    ///
    /// Use this variant when your `Cors` struct will live at least as long as the whole `'r`
    /// lifetime of the request. If you are getting your `Cors` from Rocket's state, you will have
    /// to use the [`inner` function](https://api.rocket.rs/rocket/struct.State.html#method.inner)
    /// to get a longer borrowed lifetime.
    ///
    /// After the CORS checks are done, the passed in handler closure will be run to generate a
    /// final response. You will have to merge your response with the `Guard` that you have been
    /// passed in to include the CORS headers.
    ///
    /// See the documentation at the [crate root](index.html) for usage information.
    pub fn respond_borrowed<'r, F, R>(
        &'r self,
        handler: F,
    ) -> Result<ManualResponder<'r, F, R>, Error>
    where
        F: FnOnce(Guard<'r>) -> R + 'r,
        R: response::Responder<'r>,
    {
        self.validate()?;
        Ok(ManualResponder::new(Cow::Borrowed(self), handler))
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
///
/// You can get this struct by using `Cors::validate_request` in an ad-hoc manner.
#[derive(Eq, PartialEq, Debug)]
pub(crate) struct Response {
    allow_origin: Option<AllOrSome<Url>>,
    allow_methods: HashSet<Method>,
    allow_headers: HeaderFieldNamesSet,
    allow_credentials: bool,
    expose_headers: HeaderFieldNamesSet,
    max_age: Option<usize>,
    vary_origin: bool,
}

impl Response {
    /// Create an empty `Response`
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
    fn origin(mut self, origin: &Url, vary_origin: bool) -> Self {
        self.allow_origin = Some(AllOrSome::Some(origin.clone()));
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

    /// Consumes the `Response` and return  a `Responder` that wraps a
    /// provided `rocket:response::Responder` with CORS headers
    pub fn responder<'r, R: response::Responder<'r>>(self, responder: R) -> Responder<'r, R> {
        Responder::new(responder, self)
    }

    /// Merge a `rocket::Response` with this CORS response. This is usually used in the final step
    /// of a route to return a value for the route.
    ///
    /// This will overwrite any existing CORS headers
    pub fn response<'r>(&self, base: response::Response<'r>) -> response::Response<'r> {
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
            AllOrSome::Some(ref origin) => origin.origin().unicode_serialization(),
        };

        let _ = response.set_raw_header("Access-Control-Allow-Origin", origin);

        if self.allow_credentials {
            let _ = response.set_raw_header("Access-Control-Allow-Credentials", "true");
        } else {
            response.remove_header("Access-Control-Allow-Credentials");
        }

        if !self.expose_headers.is_empty() {
            let headers: Vec<String> = self.expose_headers
                .iter()
                .map(|s| s.deref().to_string())
                .collect();
            let headers = headers.join(", ");

            let _ = response.set_raw_header("Access-Control-Expose-Headers", headers);
        } else {
            response.remove_header("Access-Control-Expose-Headers");
        }

        if !self.allow_headers.is_empty() {
            let headers: Vec<String> = self.allow_headers
                .iter()
                .map(|s| s.deref().to_string())
                .collect();
            let headers = headers.join(", ");

            let _ = response.set_raw_header("Access-Control-Allow-Headers", headers);
        } else {
            response.remove_header("Access-Control-Allow-Headers");
        }

        if !self.allow_methods.is_empty() {
            let methods: Vec<_> = self.allow_methods.iter().map(|m| m.as_str()).collect();
            let methods = methods.join(", ");

            let _ = response.set_raw_header("Access-Control-Allow-Methods", methods);
        } else {
            response.remove_header("Access-Control-Allow-Methods");
        }

        if self.max_age.is_some() {
            let max_age = self.max_age.unwrap();
            let _ = response.set_raw_header("Access-Control-Max-Age", max_age.to_string());
        } else {
            response.remove_header("Access-Control-Max-Age");
        }

        if self.vary_origin {
            let _ = response.set_raw_header("Vary", "Origin");
        } else {
            response.remove_header("Vary");
        }
    }

    /// Validate and create a new CORS Response from a request and settings
    pub fn validate_and_build<'a, 'r>(
        options: &'a Cors,
        request: &'a Request<'r>,
    ) -> Result<Self, Error> {
        validate_and_build(options, request)
    }
}


/// A [request guard](https://rocket.rs/guide/requests/#request-guards) to check CORS headers
/// before a route is run. Will not execute the route if checks fail.
///
/// See the documentation at the [crate root](index.html) for usage information.
///
/// You should not wrap this in an
/// `Option` or `Result` because the guard will let non-CORS requests through and will take over
/// error handling in case of errors.
/// In essence, this is just a wrapper around `Response` with a `'r` borrowed lifetime so users
/// don't have to keep specifying the lifetimes in their routes
pub struct Guard<'r> {
    response: Response,
    marker: PhantomData<&'r Response>,
}

impl<'r> Guard<'r> {
    fn new(response: Response) -> Self {
        Self {
            response,
            marker: PhantomData,
        }
    }

    /// Consumes the Guard and return  a `Responder` that wraps a
    /// provided `rocket:response::Responder` with CORS headers
    pub fn responder<R: response::Responder<'r>>(self, responder: R) -> Responder<'r, R> {
        self.response.responder(responder)
    }

    /// Merge a `rocket::Response` with this CORS Guard. This is usually used in the final step
    /// of a route to return a value for the route.
    ///
    /// This will overwrite any existing CORS headers
    pub fn response(&self, base: response::Response<'r>) -> response::Response<'r> {
        self.response.response(base)
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Guard<'r> {
    type Error = Error;

    fn from_request(request: &'a Request<'r>) -> rocket::request::Outcome<Self, Self::Error> {
        let options = match request.guard::<State<Cors>>() {
            Outcome::Success(options) => options,
            _ => {
                let error = Error::MissingCorsInRocketState;
                return Outcome::Failure((error.status(), error));
            }
        };

        match Response::validate_and_build(&options, request) {
            Ok(response) => Outcome::Success(Self::new(response)),
            Err(error) => Outcome::Failure((error.status(), error)),
        }
    }
}

/// A [`Responder`](https://rocket.rs/guide/responses/#responder) which will simply wraps another
/// `Responder` with CORS headers.
///
/// The following CORS headers will be overwritten:
///
/// - `Access-Control-Allow-Origin`
/// - `Access-Control-Expose-Headers`
/// - `Access-Control-Max-Age`
/// - `Access-Control-Allow-Credentials`
/// - `Access-Control-Allow-Methods`
/// - `Access-Control-Allow-Headers`
/// - `Vary`
///
/// See the documentation at the [crate root](index.html) for usage information.
#[derive(Debug)]
pub struct Responder<'r, R> {
    responder: R,
    cors_response: Response,
    marker: PhantomData<response::Responder<'r>>,
}

impl<'r, R: response::Responder<'r>> Responder<'r, R> {
    fn new(responder: R, cors_response: Response) -> Self {
        Self {
            responder,
            cors_response,
            marker: PhantomData,
        }
    }

    /// Respond to a request
    fn respond(self, request: &Request) -> response::Result<'r> {
        let mut response = self.responder.respond_to(request)?; // handle status errors?
        self.cors_response.merge(&mut response);
        Ok(response)
    }
}

impl<'r, R: response::Responder<'r>> response::Responder<'r> for Responder<'r, R> {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        self.respond(request)
    }
}

/// A Manual Responder used in the "truly manual" mode of operation.
///
/// See the documentation at the [crate root](index.html) for usage information.
pub struct ManualResponder<'r, F, R> {
    options: Cow<'r, Cors>,
    handler: F,
    marker: PhantomData<R>,
}

impl<'r, F, R> ManualResponder<'r, F, R>
where
    F: FnOnce(Guard<'r>) -> R + 'r,
    R: response::Responder<'r>,
{
    /// Create a new manual responder by passing in either a borrowed or owned `Cors` option.
    ///
    /// A borrowed `Cors` option must live for the entirety of the `'r` lifetime which is the
    /// lifetime of the entire Rocket request.
    fn new(options: Cow<'r, Cors>, handler: F) -> Self {
        let marker = PhantomData;
        Self {
            options,
            handler,
            marker,
        }
    }

    fn build_guard(&self, request: &Request) -> Result<Guard<'r>, Error> {
        let response = Response::validate_and_build(&self.options, request)?;
        Ok(Guard::new(response))
    }
}

impl<'r, F, R> response::Responder<'r> for ManualResponder<'r, F, R>
where
    F: FnOnce(Guard<'r>) -> R + 'r,
    R: response::Responder<'r>,
{
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        let guard = match self.build_guard(request) {
            Ok(guard) => guard,
            Err(err) => {
                error_!("CORS error: {}", err);
                return Err(err.status());
            }
        };
        (self.handler)(guard).respond_to(request)
    }
}

/// Result of CORS validation.
///
/// The variants hold enough information to build a response to the validation result
#[derive(Debug, Eq, PartialEq)]
enum ValidationResult {
    /// Not a CORS request
    None,
    /// Successful preflight request
    Preflight {
        origin: Origin,
        headers: Option<AccessControlRequestHeaders>,
    },
    /// Successful actual request
    Request { origin: Origin },
}

/// Validates a request for CORS and returns a CORS Response
fn validate_and_build(options: &Cors, request: &Request) -> Result<Response, Error> {
    let result = validate(options, request)?;

    Ok(match result {
        ValidationResult::None => Response::new(),
        ValidationResult::Preflight { origin, headers } => {
            preflight_response(options, &origin, headers.as_ref())
        }
        ValidationResult::Request { origin } => actual_request_response(options, &origin),
    })
}

/// Validate a CORS request
fn validate(options: &Cors, request: &Request) -> Result<ValidationResult, Error> {
    // 1. If the Origin header is not present terminate this set of steps.
    // The request is outside the scope of this specification.
    let origin = origin(request)?;
    let origin = match origin {
        None => {
            // Not a CORS request
            return Ok(ValidationResult::None);
        }
        Some(origin) => origin,
    };

    // Check if the request verb is an OPTION or something else
    match request.method() {
        http::Method::Options => {
            let method = request_method(request)?;
            let headers = request_headers(request)?;
            preflight_validate(options, &origin, &method, &headers)?;
            Ok(ValidationResult::Preflight { origin, headers })
        }
        _ => {
            actual_request_validate(options, &origin)?;
            Ok(ValidationResult::Request { origin })
        }
    }
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

/// Do pre-flight validation checks
///
/// This implementation references the
/// [W3C recommendation](https://www.w3.org/TR/cors/#resource-preflight-requests).
fn preflight_validate(
    options: &Cors,
    origin: &Origin,
    method: &Option<AccessControlRequestMethod>,
    headers: &Option<AccessControlRequestHeaders>,
) -> Result<(), Error> {

    options.validate()?; // Fast-forward check for #7

    // Note: All header parse failures are dealt with in the `FromRequest` trait implementation

    // 2. If the value of the Origin header is not a case-sensitive match for any of the values
    // in list of origins do not set any additional headers and terminate this set of steps.
    validate_origin(origin, &options.allowed_origins)?;

    // 3. Let `method` be the value as result of parsing the Access-Control-Request-Method
    // header.
    // If there is no Access-Control-Request-Method header or if parsing failed,
    // do not set any additional headers and terminate this set of steps.
    // The request is outside the scope of this specification.

    let method = method.as_ref().ok_or_else(|| Error::MissingRequestMethod)?;

    // 4. Let header field-names be the values as result of parsing the
    // Access-Control-Request-Headers headers.
    // If there are no Access-Control-Request-Headers headers
    // let header field-names be the empty list.
    // If parsing failed do not set any additional headers and terminate this set of steps.
    // The request is outside the scope of this specification.

    // 5. If method is not a case-sensitive match for any of the values in list of methods
    // do not set any additional headers and terminate this set of steps.

    validate_allowed_method(method, &options.allowed_methods)?;

    // 6. If any of the header field-names is not a ASCII case-insensitive match for any of the
    // values in list of headers do not set any additional headers and terminate this set of
    // steps.

    if let Some(ref headers) = *headers {
        validate_allowed_headers(headers, &options.allowed_headers)?;
    }

    Ok(())
}

/// Build a response for pre-flight checks
///
/// This implementation references the
/// [W3C recommendation](https://www.w3.org/TR/cors/#resource-preflight-requests).
fn preflight_response(
    options: &Cors,
    origin: &Origin,
    headers: Option<&AccessControlRequestHeaders>,
) -> Response {
    let response = Response::new();

    // 7. If the resource supports credentials add a single Access-Control-Allow-Origin header,
    // with the value of the Origin header as value, and add a
    // single Access-Control-Allow-Credentials header with the case-sensitive string "true" as
    // value.
    // Otherwise, add a single Access-Control-Allow-Origin header,
    // with either the value of the Origin header or the string "*" as value.
    // Note: The string "*" cannot be used for a resource that supports credentials.

    // Validation has been done in options.validate
    let response = match options.allowed_origins {
        AllOrSome::All => {
            if options.send_wildcard {
                response.any()
            } else {
                response.origin(origin, true)
            }
        }
        AllOrSome::Some(_) => response.origin(origin, false),
    };
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

    let response = response.methods(&options.allowed_methods);

    // 10. If each of the header field-names is a simple header and none is Content-Type,
    // this step may be skipped.
    // Add one or more Access-Control-Allow-Headers headers consisting of (a subset of)
    // the list of headers.
    // If a header field name is a simple header and is not Content-Type,
    // it is not required to be listed. Content-Type is to be listed as only a
    // subset of its values makes it qualify as simple header.
    // Since the list of headers can be unbounded, simply returning supported headers
    // from Access-Control-Allow-Headers can be enough.

    // We do not do anything special with simple headers
    if let Some(headers) = headers {
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
    }
}

/// Do checks for an actual request
///
/// This implementation references the
/// [W3C recommendation](https://www.w3.org/TR/cors/#resource-requests).
fn actual_request_validate(options: &Cors, origin: &Origin) -> Result<(), Error> {
    options.validate()?;

    // Note: All header parse failures are dealt with in the `FromRequest` trait implementation

    // 2. If the value of the Origin header is not a case-sensitive match for any of the values
    // in list of origins, do not set any additional headers and terminate this set of steps.
    // Always matching is acceptable since the list of origins can be unbounded.

    validate_origin(origin, &options.allowed_origins)?;

    Ok(())
}

/// Build the response for an actual request
///
/// This implementation references the
/// [W3C recommendation](https://www.w3.org/TR/cors/#resource-requests).
fn actual_request_response(options: &Cors, origin: &Origin) -> Response {
    let response = Response::new();

    // 3. If the resource supports credentials add a single Access-Control-Allow-Origin header,
    // with the value of the Origin header as value, and add a
    // single Access-Control-Allow-Credentials header with the case-sensitive string "true" as
    // value.
    // Otherwise, add a single Access-Control-Allow-Origin header,
    // with either the value of the Origin header or the string "*" as value.
    // Note: The string "*" cannot be used for a resource that supports credentials.

    // Validation has been done in options.validate

    let response = match options.allowed_origins {
        AllOrSome::All => {
            if options.send_wildcard {
                response.any()
            } else {
                response.origin(origin, true)
            }
        }
        AllOrSome::Some(_) => response.origin(origin, false),
    };

    let response = response.credentials(options.allow_credentials);

    // 4. If the list of exposed headers is not empty add one or more
    // Access-Control-Expose-Headers headers, with as values the header field names given in
    // the list of exposed headers.
    // By not adding the appropriate headers resource can also clear the preflight result cache
    // of all entries where origin is a case-sensitive match for the value of the Origin header
    // and url is a case-sensitive match for the URL of the resource.

    response.exposed_headers(
        options
            .expose_headers
            .iter()
            .map(|s| &**s)
            .collect::<Vec<&str>>()
            .as_slice(),
    )
}

/// Returns "catch all" OPTIONS routes that you can mount to catch all OPTIONS request. Only works
/// if you have put a `Cors` struct into Rocket's managed state.
///
/// This route has very high rank (and therefore low priority) of
/// [max value](https://doc.rust-lang.org/nightly/std/primitive.isize.html#method.max_value)
/// so you can define your own to override this route's behaviour.
///
/// See the documentation at the [crate root](index.html) for usage information.
pub fn catch_all_options_routes() -> Vec<rocket::Route> {
    vec![
        rocket::Route::ranked(
            isize::max_value(),
            http::Method::Options,
            "/",
            catch_all_options_route_handler
        ),
        rocket::Route::ranked(
            isize::max_value(),
            http::Method::Options,
            "/<catch_all_options_route..>",
            catch_all_options_route_handler
        ),
    ]
}

/// Handler for the "catch all options route"
fn catch_all_options_route_handler<'r>(
    request: &'r Request,
    _: rocket::Data,
) -> rocket::handler::Outcome<'r> {

    let guard: Guard = match request.guard() {
        Outcome::Success(guard) => guard,
        Outcome::Failure((status, _)) => return rocket::handler::Outcome::failure(status),
        Outcome::Forward(()) => unreachable!("Should not be reachable"),
    };

    info_!(
        "\"Catch all\" handling of CORS `OPTIONS` preflight for request {}",
        request
    );

    rocket::handler::Outcome::from(request, guard.responder(()))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rocket::local::Client;
    use rocket::http::Header;
    #[cfg(feature = "serialization")]
    use serde_json;

    use super::*;
    use http::Method;

    fn make_cors_options() -> Cors {
        let (allowed_origins, failed_origins) = AllowedOrigins::some(&["https://www.acme.com"]);
        assert!(failed_origins.is_empty());

        Cors {
            allowed_origins: allowed_origins,
            allowed_methods: vec![http::Method::Get]
                .into_iter()
                .map(From::from)
                .collect(),
            allowed_headers: AllowedHeaders::some(&[&"Authorization", "Accept"]),
            allow_credentials: true,
            expose_headers: ["Content-Type", "X-Custom"]
                .into_iter()
                .map(|s| s.to_string().into())
                .collect(),
            ..Default::default()
        }
    }

    fn make_invalid_options() -> Cors {
        let mut cors = make_cors_options();
        cors.allow_credentials = true;
        cors.allowed_origins = AllOrSome::All;
        cors.send_wildcard = true;
        cors
    }

    /// Make a client with no routes for unit testing
    fn make_client() -> Client {
        let rocket = rocket::ignite();
        Client::new(rocket).expect("valid rocket instance")
    }

    // CORS options test

    #[test]
    fn cors_is_validated() {
        assert!(make_cors_options().validate().is_ok())
    }

    #[test]
    #[should_panic(expected = "CredentialsWithWildcardOrigin")]
    fn cors_validates_illegal_allow_credentials() {
        let cors = make_invalid_options();

        cors.validate().unwrap();
    }

    /// Check that the the default deserialization matches the one returned by `Default::default`
    #[cfg(feature = "serialization")]
    #[test]
    fn cors_default_deserialization_is_correct() {
        let deserialized: Cors = serde_json::from_str("{}").expect("To not fail");
        assert_eq!(deserialized, Cors::default());
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
        let (allowed_origins, failed_origins) = AllowedOrigins::some(&["https://www.example.com"]);
        assert!(failed_origins.is_empty());

        not_err!(validate_origin(&origin, &allowed_origins));
    }

    #[test]
    #[should_panic(expected = "OriginNotAllowed")]
    fn response_rejects_invalid_origin() {
        let url = "https://www.acme.com";
        let origin = Origin::from_str(url).unwrap();
        let (allowed_origins, failed_origins) = AllowedOrigins::some(&["https://www.example.com"]);
        assert!(failed_origins.is_empty());

        validate_origin(&origin, &allowed_origins).unwrap();
    }

    #[test]
    fn response_sets_exposed_headers_correctly() {
        let headers = vec!["Bar", "Baz", "Foo"];
        let response = Response::new();
        let response = response.origin(
            &FromStr::from_str("https://www.example.com").unwrap(),
            false,
        );
        let response = response.exposed_headers(&headers);

        // Build response and check built response header
        let response = response.response(response::Response::new());
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
        let response = response.origin(
            &FromStr::from_str("https://www.example.com").unwrap(),
            false,
        );

        let response = response.max_age(Some(42));

        // Build response and check built response header
        let expected_header = vec!["42"];
        let response = response.response(response::Response::new());
        let actual_header: Vec<_> = response.headers().get("Access-Control-Max-Age").collect();
        assert_eq!(expected_header, actual_header);
    }

    #[test]
    fn response_does_not_set_max_age_when_none() {
        let response = Response::new();
        let response = response.origin(
            &FromStr::from_str("https://www.example.com").unwrap(),
            false,
        );

        let response = response.max_age(None);

        // Build response and check built response header
        let response = response.response(response::Response::new());
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
            .map(From::from)
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
            .map(From::from)
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
        let response = response.response(response::Response::new());

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
        let response = response.origin(
            &FromStr::from_str("https://www.example.com").unwrap(),
            false,
        );
        let response = response.response(original);
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

    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
    struct MethodTest {
        method: ::Method,
    }

    #[cfg(feature = "serialization")]
    #[test]
    fn method_serde_roundtrip() {
        use serde_test::{Token, assert_tokens};

        let test = MethodTest { method: From::from(http::Method::Get) };

        assert_tokens(
            &test,
            &[
                Token::Struct {
                    name: "MethodTest",
                    len: 1,
                },
                Token::Str("method"),
                Token::Str("GET"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn preflight_validated_correctly() {
        let options = make_cors_options();
        let client = make_client();

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

        let request = client
            .options("/")
            .header(origin_header)
            .header(method_header)
            .header(request_headers);

        let result = validate(&options, request.inner()).expect("to not fail");
        let expected_result = ValidationResult::Preflight {
            origin: FromStr::from_str("https://www.acme.com").unwrap(),
            // Checks that only a subset of allowed headers are returned
            // -- i.e. whatever is requested for
            headers: Some(FromStr::from_str("Authorization").unwrap()),
        };

        assert_eq!(expected_result, result);
    }

    #[test]
    #[should_panic(expected = "CredentialsWithWildcardOrigin")]
    fn preflight_validation_errors_on_invalid_options() {
        let options = make_invalid_options();
        let client = make_client();

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

        let request = client
            .options("/")
            .header(origin_header)
            .header(method_header)
            .header(request_headers);

        let _ = validate(&options, request.inner()).unwrap();
    }

    #[test]
    fn preflight_validation_allows_all_origin() {
        let mut options = make_cors_options();
        options.allowed_origins = AllOrSome::All;
        let client = make_client();

        let origin_header = Header::from(
            hyper::header::Origin::from_str("https://www.example.com").unwrap(),
        );
        let method_header = Header::from(hyper::header::AccessControlRequestMethod(
            hyper::method::Method::Get,
        ));
        let request_headers = hyper::header::AccessControlRequestHeaders(
            vec![FromStr::from_str("Authorization").unwrap()],
        );
        let request_headers = Header::from(request_headers);

        let request = client
            .options("/")
            .header(origin_header)
            .header(method_header)
            .header(request_headers);

        let result = validate(&options, request.inner()).expect("to not fail");
        let expected_result = ValidationResult::Preflight {
            origin: FromStr::from_str("https://www.example.com").unwrap(),
            headers: Some(FromStr::from_str("Authorization").unwrap()),
        };

        assert_eq!(expected_result, result);
    }

    #[test]
    #[should_panic(expected = "OriginNotAllowed")]
    fn preflight_validation_errors_on_invalid_origin() {
        let options = make_cors_options();
        let client = make_client();

        let origin_header = Header::from(
            hyper::header::Origin::from_str("https://www.example.com").unwrap(),
        );
        let method_header = Header::from(hyper::header::AccessControlRequestMethod(
            hyper::method::Method::Get,
        ));
        let request_headers = hyper::header::AccessControlRequestHeaders(
            vec![FromStr::from_str("Authorization").unwrap()],
        );
        let request_headers = Header::from(request_headers);

        let request = client
            .options("/")
            .header(origin_header)
            .header(method_header)
            .header(request_headers);

        let _ = validate(&options, request.inner()).unwrap();
    }

    #[test]
    #[should_panic(expected = "MissingRequestMethod")]
    fn preflight_validation_errors_on_missing_request_method() {
        let options = make_cors_options();
        let client = make_client();

        let origin_header = Header::from(
            hyper::header::Origin::from_str("https://www.acme.com").unwrap(),
        );
        let request_headers = hyper::header::AccessControlRequestHeaders(
            vec![FromStr::from_str("Authorization").unwrap()],
        );
        let request_headers = Header::from(request_headers);

        let request = client.options("/").header(origin_header).header(
            request_headers,
        );

        let _ = validate(&options, request.inner()).unwrap();
    }

    #[test]
    #[should_panic(expected = "MethodNotAllowed")]
    fn preflight_validation_errors_on_disallowed_method() {
        let options = make_cors_options();
        let client = make_client();

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

        let request = client
            .options("/")
            .header(origin_header)
            .header(method_header)
            .header(request_headers);

        let _ = validate(&options, request.inner()).unwrap();
    }

    #[test]
    #[should_panic(expected = "HeadersNotAllowed")]
    fn preflight_validation_errors_on_disallowed_headers() {
        let options = make_cors_options();
        let client = make_client();

        let origin_header = Header::from(
            hyper::header::Origin::from_str("https://www.acme.com").unwrap(),
        );
        let method_header = Header::from(hyper::header::AccessControlRequestMethod(
            hyper::method::Method::Get,
        ));
        let request_headers = hyper::header::AccessControlRequestHeaders(vec![
            FromStr::from_str("Authorization").unwrap(),
            FromStr::from_str("X-NOT-ALLOWED").unwrap(),
        ]);
        let request_headers = Header::from(request_headers);

        let request = client
            .options("/")
            .header(origin_header)
            .header(method_header)
            .header(request_headers);

        let _ = validate(&options, request.inner()).unwrap();
    }

    #[test]
    fn actual_request_validated_correctly() {
        let options = make_cors_options();
        let client = make_client();

        let origin_header = Header::from(
            hyper::header::Origin::from_str("https://www.acme.com").unwrap(),
        );
        let request = client.get("/").header(origin_header);

        let result = validate(&options, request.inner()).expect("to not fail");
        let expected_result = ValidationResult::Request {
            origin: FromStr::from_str("https://www.acme.com").unwrap(),
        };

        assert_eq!(expected_result, result);
    }

    #[test]
    #[should_panic(expected = "CredentialsWithWildcardOrigin")]
    fn actual_request_validation_errors_on_invalid_options() {
        let options = make_invalid_options();
        let client = make_client();

        let origin_header = Header::from(
            hyper::header::Origin::from_str("https://www.acme.com").unwrap(),
        );
        let request = client.get("/").header(origin_header);

        let _ = validate(&options, request.inner()).unwrap();
    }

    #[test]
    fn actual_request_validation_allows_all_origin() {
        let mut options = make_cors_options();
        options.allowed_origins = AllOrSome::All;
        let client = make_client();

        let origin_header = Header::from(
            hyper::header::Origin::from_str("https://www.example.com").unwrap(),
        );
        let request = client.get("/").header(origin_header);

        let result = validate(&options, request.inner()).expect("to not fail");
        let expected_result = ValidationResult::Request {
            origin: FromStr::from_str("https://www.example.com").unwrap(),
        };

        assert_eq!(expected_result, result);
    }

    #[test]
    #[should_panic(expected = "OriginNotAllowed")]
    fn actual_request_validation_errors_on_incorrect_origin() {
        let options = make_cors_options();
        let client = make_client();

        let origin_header = Header::from(
            hyper::header::Origin::from_str("https://www.example.com").unwrap(),
        );
        let request = client.get("/").header(origin_header);

        let _ = validate(&options, request.inner()).unwrap();
    }

    #[test]
    fn non_cors_request_return_empty_response() {
        let options = make_cors_options();
        let client = make_client();

        let request = client.options("/");
        let response = validate_and_build(&options, request.inner()).expect("to not fail");
        let expected_response = Response::new();
        assert_eq!(expected_response, response);
    }

    #[test]
    fn preflight_validated_and_built_correctly() {
        let options = make_cors_options();
        let client = make_client();

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

        let request = client
            .options("/")
            .header(origin_header)
            .header(method_header)
            .header(request_headers);

        let response = validate_and_build(&options, request.inner()).expect("to not fail");

        let expected_response = Response::new()
            .origin(&FromStr::from_str("https://www.acme.com/").unwrap(), false)
            .headers(&["Authorization"])
            .methods(&options.allowed_methods)
            .credentials(options.allow_credentials)
            .max_age(options.max_age);

        assert_eq!(expected_response, response);
    }

    /// Tests that when All origins are allowed and send_wildcard disabled, the vary header is set
    /// in the response and the requested origin is echoed
    #[test]
    fn preflight_all_origins_with_vary() {
        let mut options = make_cors_options();
        options.allowed_origins = AllOrSome::All;
        options.send_wildcard = false;

        let client = make_client();

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

        let request = client
            .options("/")
            .header(origin_header)
            .header(method_header)
            .header(request_headers);

        let response = validate_and_build(&options, request.inner()).expect("to not fail");

        let expected_response = Response::new()
            .origin(&FromStr::from_str("https://www.acme.com/").unwrap(), true)
            .headers(&["Authorization"])
            .methods(&options.allowed_methods)
            .credentials(options.allow_credentials)
            .max_age(options.max_age);

        assert_eq!(expected_response, response);
    }

    /// Tests that when All origins are allowed and send_wildcard enabled, the origin is set to "*"
    #[test]
    fn preflight_all_origins_with_wildcard() {
        let mut options = make_cors_options();
        options.allowed_origins = AllOrSome::All;
        options.send_wildcard = true;
        options.allow_credentials = false;

        let client = make_client();

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

        let request = client
            .options("/")
            .header(origin_header)
            .header(method_header)
            .header(request_headers);

        let response = validate_and_build(&options, request.inner()).expect("to not fail");

        let expected_response = Response::new()
            .any()
            .headers(&["Authorization"])
            .methods(&options.allowed_methods)
            .credentials(options.allow_credentials)
            .max_age(options.max_age);

        assert_eq!(expected_response, response);
    }

    #[test]
    fn actual_request_validated_and_built_correctly() {
        let options = make_cors_options();
        let client = make_client();

        let origin_header = Header::from(
            hyper::header::Origin::from_str("https://www.acme.com").unwrap(),
        );
        let request = client.get("/").header(origin_header);

        let response = validate_and_build(&options, request.inner()).expect("to not fail");
        let expected_response = Response::new()
            .origin(&FromStr::from_str("https://www.acme.com/").unwrap(), false)
            .credentials(options.allow_credentials)
            .exposed_headers(&["Content-Type", "X-Custom"]);

        assert_eq!(expected_response, response);
    }

    #[test]
    fn actual_request_all_origins_with_vary() {
        let mut options = make_cors_options();
        options.allowed_origins = AllOrSome::All;
        options.send_wildcard = false;
        options.allow_credentials = false;

        let client = make_client();

        let origin_header = Header::from(
            hyper::header::Origin::from_str("https://www.acme.com").unwrap(),
        );
        let request = client.get("/").header(origin_header);

        let response = validate_and_build(&options, request.inner()).expect("to not fail");
        let expected_response = Response::new()
            .origin(&FromStr::from_str("https://www.acme.com/").unwrap(), true)
            .credentials(options.allow_credentials)
            .exposed_headers(&["Content-Type", "X-Custom"]);

        assert_eq!(expected_response, response);
    }

    #[test]
    fn actual_request_all_origins_with_wildcard() {
        let mut options = make_cors_options();
        options.allowed_origins = AllOrSome::All;
        options.send_wildcard = true;
        options.allow_credentials = false;

        let client = make_client();

        let origin_header = Header::from(
            hyper::header::Origin::from_str("https://www.acme.com").unwrap(),
        );
        let request = client.get("/").header(origin_header);

        let response = validate_and_build(&options, request.inner()).expect("to not fail");
        let expected_response = Response::new()
            .any()
            .credentials(options.allow_credentials)
            .exposed_headers(&["Content-Type", "X-Custom"]);

        assert_eq!(expected_response, response);
    }
}
