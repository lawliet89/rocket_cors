/*!
[![Continuous integration](https://github.com/lawliet89/rocket_cors/actions/workflows/rust.yml/badge.svg)](https://github.com/lawliet89/rocket_cors/actions/workflows/rust.yml)
[![Repository](https://img.shields.io/github/tag/lawliet89/rocket_cors.svg)](https://github.com/lawliet89/rocket_cors)
[![Crates.io](https://img.shields.io/crates/v/rocket_cors.svg)](https://crates.io/crates/rocket_cors)

- Documentation: [master branch](https://lawliet89.github.io/rocket_cors) | [stable](https://docs.rs/rocket_cors)

Cross-origin resource sharing (CORS) for [Rocket](https://rocket.rs/) applications

## Requirements

- Rocket >= 0.4

If you are using Rocket 0.3, use the `0.3.0` version of this crate.

## Installation

Add the following to Cargo.toml:

```toml
rocket_cors = "0.6.0"
```

To use the latest `master` branch, for example:

```toml
rocket_cors = { git = "https://github.com/lawliet89/rocket_cors", branch = "master" }
```

## Features

By default, a `serialization` feature is enabled in this crate that allows you to (de)serialize
the [`CorsOptions`] struct that is described below. If you would like to disable this, simply
change your `Cargo.toml` to:

```toml
rocket_cors = { version = "0.6.0", default-features = false }
```

## Usage

Before you can add CORS responses to your application, you need to create a [`CorsOptions`]
struct that will hold the settings. Then, you need to create a [`Cors`] struct using
[`CorsOptions::to_cors`] which will validate and optimise the settings for Rocket to use.

Each of the examples can be run off the repository via `cargo run --example xxx` where `xxx` is

- `fairing`
- `guard`
- `manual`

### `CorsOptions` Struct

The [`CorsOptions`] struct contains the settings for CORS requests to be validated
and for responses to be generated. Defaults are defined for every field in the struct, and
are documented on the [`CorsOptions`] page. You can also deserialize
the struct from some format like JSON, YAML or TOML when the default `serialization` feature
is enabled.

### `Cors` Struct

The [`Cors`] struct is what will be used with Rocket. After creating or deserializing a
[`CorsOptions`] struct, use [`CorsOptions::to_cors`] to create a [`Cors`] struct.

### Three modes of operation

You can add CORS to your routes via one of three ways, in descending order of ease and in
ascending order of flexibility.

- Fairing (should only be used exclusively)
- Request Guard
- Truly Manual

Unfortunately, you cannot mix and match Fairing with any other of the methods, due to the
limitation of Rocket's fairing API. That is, the checks for Fairing will always happen first,
and if they fail, the route is never executed and so your guard or manual checks will never
get executed.

You can, however, mix and match guards and manual checks.

In summary:

|                                         | Fairing | Request Guard | Manual |
|:---------------------------------------:|:-------:|:-------------:|:------:|
|         Must apply to all routes        |    ✔    |       ✗       |    ✗   |
| Different settings for different routes |    ✗    |       ✗       |    ✔   |
|     May define custom OPTIONS routes    |    ✗    |       ✔       |    ✔   |

### Fairing

Fairing is the easiest to use and also the most inflexible. You don't have to define `OPTIONS`
routes for your application, and the checks are done transparently.

However, you can only have one set of settings that must apply to all routes. You cannot opt
any route out of CORS checks.

To use this, simply create a [`Cors`] from [`CorsOptions::to_cors`] and then
[`attach`](https://api.rocket.rs/rocket/struct.Rocket.html#method.attach) it to Rocket.

Refer to the [example](https://github.com/lawliet89/rocket_cors/blob/master/examples/fairing.rs).

#### Injected Route

The fairing implementation will inject a route during attachment to Rocket. This route is used
to handle errors during CORS validation.

This is due to the limitation in Rocket's Fairing
[lifecycle](https://rocket.rs/guide/fairings/). Ideally, we want to validate the CORS request
during `on_request`, and if the validation fails, we want to stop the route from even executing
to

1) prevent side effects
1) prevent resource usage from unnecessary computation

The only way to do this is to hijack the request and route it to our own injected route to
handle errors. Rocket does not allow Fairings to stop the processing of a route.

You can configure the behaviour of the injected route through a couple of fields in the
[`CorsOptions`].

### Request Guard

Using request guard requires you to sacrifice the convenience of Fairings for being able to
opt some routes out of CORS checks and enforcement. _BUT_ you are still restricted to only
one set of CORS settings and you have to mount additional routes to catch and process OPTIONS
requests. The `OPTIONS` routes are used for CORS preflight checks.

You will have to do the following:

- Create a [`Cors`] from [`CorsOptions`] and during Rocket's ignite, add the struct to
Rocket's [managed state](https://rocket.rs/guide/state/#managed-state).
- For all the routes that you want to enforce CORS on, you can mount either some
[catch all route](catch_all_options_routes) or define your own route for the OPTIONS
verb.
- Then in all the routes you want to enforce CORS on, add a
[Request Guard](https://rocket.rs/guide/requests/#request-guards) for the
[`Guard`] struct in the route arguments. You should not wrap this in an
`Option` or `Result` because the guard will let non-CORS requests through and will take over
error handling in case of errors.
- In your routes, to add CORS headers to your responses, use the appropriate functions on the
[`Guard`] for a `Response` or a `Responder`.

Refer to the [example](https://github.com/lawliet89/rocket_cors/blob/master/examples/guard.rs).

## Truly Manual

This mode is the most difficult to use but offers the most amount of flexibility.
You might have to understand how the library works internally to know how to use this mode.
In exchange, you can selectively choose which routes to offer CORS protection to, and you
can mix and match CORS settings for the routes. You can combine usage of this mode with
"guard" to offer a mix of ease of use and flexibility.

You really do not need to use this unless you have a truly ad-hoc need to respond to CORS
differently in a route. For example, you have a `ping` endpoint that allows all origins but
the rest of your routes do not.

### Handler

This mode requires that you pass in a closure that will be lazily evaluated once a CORS request
has been validated. If validation fails, the closure will not be run. You should put any code
that has any side effects or with an appreciable computation cost inside this handler.

### Steps to perform:
- You will first need to have a [`Cors`] struct ready. This struct can be borrowed with a lifetime
at least as long as `'r` which is the lifetime of a Rocket request. `'static` works too.
In this case, you might as well use the `Guard` method above and place the `Cors` struct in
Rocket's [state](https://rocket.rs/guide/state/).
Alternatively, you can create a [`Cors`] struct directly in the route.
- Your routes _might_ need to have a `'r` lifetime and return `impl Responder<'r>`. See below.
- Using the [`Cors`] struct, use either the
[`Cors::respond_owned`] or
[`Cors::respond_borrowed`] function and pass in a handler
that will be executed once CORS validation is successful.
- Your handler will be passed a [`Guard`] which you will have to use to
add CORS headers into your own response.
- You will have to manually define your own `OPTIONS` routes.

### Notes about route lifetime
You might have to specify a `'r` lifetime in your routes and then return `impl Responder<'r>`.
If you are not sure what to do, you can try to leave the lifetime out and then add it in
when the compiler complains.

Generally, you will need to manually annotate the lifetime for the following cases where
the compiler is unable to [elide](https://doc.rust-lang.org/beta/nomicon/lifetime-elision.html)
the lifetime:

- Your function arguments do not borrow anything.
- Your function arguments borrow from more than one lifetime.
- Your function arguments borrow from a lifetime that is shorter than the `'r` lifetime
required.

You can see examples when the lifetime annotation is required (or not) in `examples/manual.rs`.

See the [example](https://github.com/lawliet89/rocket_cors/blob/master/examples/manual.rs).

## Mixing Guard and Manual

You can mix `Guard` and `Truly Manual` modes together for your application. For example, your
application might restrict the Origins that can access it, except for one `ping` route that
allows all access.

See the [example](https://github.com/lawliet89/rocket_cors/blob/master/examples/guard.rs).

## Reference
- [Fetch CORS Specification](https://fetch.spec.whatwg.org/#cors-protocol)
- [Supplanted W3C CORS Specification](https://www.w3.org/TR/cors/)
- [Resource Advice](https://w3c.github.io/webappsec-cors-for-developers/#resources)
*/

#![deny(
    dead_code,
    deprecated,
    arithmetic_overflow,
    improper_ctypes,
    missing_docs,
    mutable_transmutes,
    no_mangle_const_items,
    non_camel_case_types,
    non_shorthand_field_patterns,
    non_upper_case_globals,
    overflowing_literals,
    path_statements,
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
    variant_size_differences,
    warnings,
    while_true
)]
#![allow(
    missing_copy_implementations,
    missing_debug_implementations,
    unknown_lints,
    unsafe_code,
    rustdoc::broken_intra_doc_links
)]
#![doc(test(attr(allow(unused_variables), deny(warnings))))]

#[cfg(test)]
#[macro_use]
mod test_macros;
mod fairing;

pub mod headers;

use std::borrow::Cow;
use std::collections::HashSet;
use std::error;
use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;
use std::str::FromStr;

#[allow(unused_imports)]
use ::log::{debug, error, info};
use regex::RegexSet;
use rocket::http::{self, Status};
use rocket::request::{FromRequest, Request};
use rocket::response;
use rocket::{debug_, error_, info_, outcome::Outcome, State};
#[cfg(feature = "serialization")]
use serde_derive::{Deserialize, Serialize};

use crate::headers::{
    AccessControlRequestHeaders, AccessControlRequestMethod, HeaderFieldName, HeaderFieldNamesSet,
    Origin,
};

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
    /// The configured Allowed Origins are Opaque origins. Use a Regex instead.
    OpaqueAllowedOrigin(Vec<String>),
    /// The request header `Access-Control-Request-Method` is required but is missing
    MissingRequestMethod,
    /// The request header `Access-Control-Request-Method` has an invalid value
    BadRequestMethod,
    /// The request header `Access-Control-Request-Headers`  is required but is missing.
    MissingRequestHeaders,
    /// Origin is not allowed to make this request
    OriginNotAllowed(String),
    /// Requested method is not allowed
    MethodNotAllowed(String),
    /// A regular expression compilation error
    RegexError(regex::Error),
    /// One or more headers requested are not allowed
    HeadersNotAllowed,
    /// Credentials are allowed, but the Origin is set to "*". This is not allowed by W3C
    ///
    /// This is a misconfiguration. Check the documentation for `Cors`.
    CredentialsWithWildcardOrigin,
    /// A CORS Request Guard was used, but no CORS Options was available in Rocket's state
    ///
    /// This is a misconfiguration. Use `Rocket::manage` to add a CORS options to managed state.
    MissingCorsInRocketState,
    /// The `on_response` handler of Fairing could not find the injected header from the Request.
    /// Either some other fairing has removed it, or this is a bug.
    MissingInjectedHeader,
}

impl Error {
    fn status(&self) -> Status {
        match *self {
            Error::MissingOrigin
            | Error::OriginNotAllowed(_)
            | Error::MethodNotAllowed(_)
            | Error::HeadersNotAllowed => Status::Forbidden,
            Error::CredentialsWithWildcardOrigin
            | Error::MissingCorsInRocketState
            | Error::MissingInjectedHeader => Status::InternalServerError,
            _ => Status::BadRequest,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingOrigin => write!(
                f,
                "The request header `Origin` is \
                 required but is missing"
            ),
            Error::BadOrigin(_) => write!(f, "The request header `Origin` contains an invalid URL"),
            Error::MissingRequestMethod => write!(
                f,
                "The request header `Access-Control-Request-Method` \
                 is required but is missing"
            ),
            Error::BadRequestMethod => write!(
                f,
                "The request header `Access-Control-Request-Method` has an invalid value"
            ),
            Error::MissingRequestHeaders => write!(
                f,
                "The request header `Access-Control-Request-Headers` \
                 is required but is missing"
            ),
            Error::OriginNotAllowed(origin) => write!(
                f,
                "Origin '{}' is \
                 not allowed to request",
                origin
            ),
            Error::MethodNotAllowed(method) => write!(f, "Method '{}' is not allowed", &method),
            Error::HeadersNotAllowed => write!(f, "Headers are not allowed"),
            Error::CredentialsWithWildcardOrigin => write!(
                f,
                "Credentials are allowed, but the Origin is set to \"*\". \
                 This is not allowed by W3C"
            ),
            Error::MissingCorsInRocketState => write!(
                f,
                "A CORS Request Guard was used, but no CORS Options \
                 was available in Rocket's state"
            ),
            Error::MissingInjectedHeader => {
                write!(f,
                "The `on_response` handler of Fairing could not find the injected header from the \
                 Request. Either some other fairing has removed it, or this is a bug.")
            }
            Error::OpaqueAllowedOrigin(ref origins) => write!(
                f,
                "The configured Origins '{}' are Opaque Origins. \
                 Use regex instead.",
                origins.join("; ")
            ),
            Error::RegexError(ref e) => write!(f, "{}", e),
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::BadOrigin(ref e) => Some(e),
            _ => Some(self),
        }
    }
}

impl<'r, 'o: 'r> response::Responder<'r, 'o> for Error {
    fn respond_to(self, _: &Request<'_>) -> Result<response::Response<'o>, Status> {
        error_!("CORS Error: {}", self);
        Err(self.status())
    }
}

impl From<url::ParseError> for Error {
    fn from(error: url::ParseError) -> Self {
        Error::BadOrigin(error)
    }
}

impl From<regex::Error> for Error {
    fn from(error: regex::Error) -> Self {
        Error::RegexError(error)
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
#[derive(Default)]
pub enum AllOrSome<T> {
    /// Everything is allowed. Usually equivalent to the "*" value.
    #[default]
    All,
    /// Only some of `T` is allowed
    Some(T),
}

impl<T> AllOrSome<T> {
    /// Returns whether this is an `All` variant
    pub fn is_all(&self) -> bool {
        match self {
            AllOrSome::All => true,
            AllOrSome::Some(_) => false,
        }
    }

    /// Returns whether this is a `Some` variant
    pub fn is_some(&self) -> bool {
        !self.is_all()
    }

    /// Unwrap a `Some` variant and get its inner value
    ///
    /// # Panics
    /// Panics if the variant is `All`
    pub fn unwrap(self) -> T {
        match self {
            AllOrSome::All => panic!("Attempting to unwrap an `All`"),
            AllOrSome::Some(inner) => inner,
        }
    }
}

/// A wrapper type around `rocket::http::Method` to support serialization and deserialization
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Method(http::Method);

impl FromStr for Method {
    type Err = ();

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(feature = "serialization")]
mod method_serde {
    use std::fmt;
    use std::str::FromStr;

    use serde::{self, Deserialize, Serialize};

    use crate::Method;

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

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
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
/// Exact matches are matched exactly with the
/// [ASCII Serialization](https://html.spec.whatwg.org/multipage/origin.html#ascii-serialisation-of-an-origin)
/// of the origin.
///
/// Regular expressions are tested for matches against the
/// [ASCII Serialization](https://html.spec.whatwg.org/multipage/origin.html#ascii-serialisation-of-an-origin)
/// of the origin.
///
/// # Opaque Origins
/// The [specification](https://html.spec.whatwg.org/multipage/origin.html) defines an Opaque Origin
/// as one that cannot be recreated. You can refer to the source code for the [`url::Url::origin`]
/// method to see how an Opaque Origin is determined. Examples of Opaque origins might include
/// schemes like `file://` or Browser specific schemes like `"moz-extension://` or
/// `chrome-extension://`.
///
/// Opaque Origins cannot be matched exactly. You must use Regex to match Opaque Origins. If you
/// attempt to create [`Cors`] from [`CorsOptions`], you will get an error.
/// # Warning about Regex expressions
/// By default, regex expressions are
/// [unanchored](https://docs.rs/regex/1.1.2/regex/struct.RegexSet.html#method.is_match).
///
/// This means that if the regex does not start with `^` or `\A`, or end with `$` or `\z`,
/// then it is permitted to match anywhere in the text. You are encouraged to use the anchors when
/// crafting your Regex expressions.
///
/// # Examples
/// ```rust
/// use rocket_cors::AllowedOrigins;
///
/// let exact = ["https://www.acme.com"];
/// let regex = ["^https://(.+).acme.com$"];
///
/// let all_origins = AllowedOrigins::all();
/// let some_origins = AllowedOrigins::some_exact(&exact);
/// let null_origins = AllowedOrigins::some_null();
/// let regex_origins = AllowedOrigins::some_regex(&regex);
/// let mixed_origins = AllowedOrigins::some(&exact, &regex);
/// ```
///
pub type AllowedOrigins = AllOrSome<Origins>;

impl AllowedOrigins {
    /// Allows some origins, with a mix of exact matches or regex matches
    ///
    /// Validation is not performed at this stage, but at a later stage.
    ///
    /// Exact matches are matched exactly with the
    /// [ASCII Serialization](https://html.spec.whatwg.org/multipage/origin.html#ascii-serialisation-of-an-origin)
    /// of the origin.
    ///
    /// Regular expressions are tested for matches against the
    /// [ASCII Serialization](https://html.spec.whatwg.org/multipage/origin.html#ascii-serialisation-of-an-origin)
    /// of the origin.
    ///
    /// # Opaque Origins
    /// The [specification](https://html.spec.whatwg.org/multipage/origin.html) defines an Opaque Origin
    /// as one that cannot be recreated. You can refer to the source code for the [`url::Url::origin`]
    /// method to see how an Opaque Origin is determined. Examples of Opaque origins might include
    /// schemes like `file://` or Browser specific schemes like `"moz-extension://` or
    /// `chrome-extension://`.
    ///
    /// Opaque Origins cannot be matched exactly. You must use Regex to match Opaque Origins. If you
    /// attempt to create [`Cors`] from [`CorsOptions`], you will get an error.
    /// # Warning about Regex expressions
    /// By default, regex expressions are
    /// [unanchored](https://docs.rs/regex/1.1.2/regex/struct.RegexSet.html#method.is_match).
    ///
    /// This means that if the regex does not start with `^` or `\A`, or end with `$` or `\z`,
    /// then it is permitted to match anywhere in the text. You are encouraged to use the anchors when
    /// crafting your Regex expressions.
    #[allow(clippy::needless_lifetimes)]
    pub fn some<'a, 'b, S1: AsRef<str>, S2: AsRef<str>>(exact: &'a [S1], regex: &'b [S2]) -> Self {
        AllOrSome::Some(Origins {
            exact: Some(exact.iter().map(|s| s.as_ref().to_string()).collect()),
            regex: Some(regex.iter().map(|s| s.as_ref().to_string()).collect()),
            ..Default::default()
        })
    }

    /// Allows some _exact_ origins
    ///
    /// Validation is not performed at this stage, but at a later stage.
    ///
    /// Exact matches are matched exactly with the
    /// [ASCII Serialization](https://html.spec.whatwg.org/multipage/origin.html#ascii-serialisation-of-an-origin)
    /// of the origin.
    /// # Opaque Origins
    /// The [specification](https://html.spec.whatwg.org/multipage/origin.html) defines an Opaque Origin
    /// as one that cannot be recreated. You can refer to the source code for the [`url::Url::origin`]
    /// method to see how an Opaque Origin is determined. Examples of Opaque origins might include
    /// schemes like `file://` or Browser specific schemes like `"moz-extension://` or
    /// `chrome-extension://`.
    pub fn some_exact<S: AsRef<str>>(exact: &[S]) -> Self {
        AllOrSome::Some(Origins {
            exact: Some(exact.iter().map(|s| s.as_ref().to_string()).collect()),
            ..Default::default()
        })
    }

    /// Allow some regular expression origins
    ///
    /// Validation is not performed at this stage, but at a later stage.
    ///
    /// Regular expressions are tested for matches against the
    /// [ASCII Serialization](https://html.spec.whatwg.org/multipage/origin.html#ascii-serialisation-of-an-origin)
    /// of the origin.
    ///
    /// # Warning about Regex expressions
    /// By default, regex expressions are
    /// [unanchored](https://docs.rs/regex/1.1.2/regex/struct.RegexSet.html#method.is_match).
    ///
    /// This means that if the regex does not start with `^` or `\A`, or end with `$` or `\z`,
    /// then it is permitted to match anywhere in the text. You are encouraged to use the anchors when
    /// crafting your Regex expressions.
    pub fn some_regex<S: AsRef<str>>(regex: &[S]) -> Self {
        AllOrSome::Some(Origins {
            regex: Some(regex.iter().map(|s| s.as_ref().to_string()).collect()),
            ..Default::default()
        })
    }

    /// Allow some `null` origins
    pub fn some_null() -> Self {
        AllOrSome::Some(Origins {
            allow_null: true,
            ..Default::default()
        })
    }

    /// Allows all origins
    pub fn all() -> Self {
        AllOrSome::All
    }
}

/// Origins that are allowed to make CORS requests.
///
/// An origin is defined according to the defined
/// [syntax](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Origin).
///
/// Origins can be specified as an exact match or using regex.
///
/// These Origins are specified as logical `ORs`. That is, if any of the origins match, the entire
/// request is considered to be valid.
///
/// Exact matches are matched exactly with the
/// [ASCII Serialization](https://html.spec.whatwg.org/multipage/origin.html#ascii-serialisation-of-an-origin)
/// of the origin.
///
/// Regular expressions are tested for matches against the
/// [ASCII Serialization](https://html.spec.whatwg.org/multipage/origin.html#ascii-serialisation-of-an-origin)
/// of the origin.
///
/// # Opaque Origins
/// The [specification](https://html.spec.whatwg.org/multipage/origin.html) defines an Opaque Origin
/// as one that cannot be recreated. You can refer to the source code for the [`url::Url::origin`]
/// method to see how an Opaque Origin is determined. Examples of Opaque origins might include
/// schemes like `file://` or Browser specific schemes like `"moz-extension://` or
/// `chrome-extension://`.
///
/// Opaque Origins cannot be matched exactly. You must use Regex to match Opaque Origins. If you
/// attempt to create [`Cors`] from [`CorsOptions`], you will get an error.
///
/// # Warning about Regex expressions
/// By default, regex expressions are
/// [unanchored](https://docs.rs/regex/1.1.2/regex/struct.RegexSet.html#method.is_match).
///
/// This means that if the regex does not start with `^` or `\A`, or end with `$` or `\z`,
/// then it is permitted to match anywhere in the text. You are encouraged to use the anchors when
/// crafting your Regex expressions.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serialization", serde(default))]
pub struct Origins {
    /// Whether null origins are accepted
    #[cfg_attr(feature = "serialization", serde(default))]
    pub allow_null: bool,
    /// Origins that must be matched exactly as provided.
    ///
    /// These __must__ be valid URL strings that will be parsed and validated when
    /// creating [`Cors`].
    ///
    /// Exact matches are matched exactly with the
    /// [ASCII Serialization](https://html.spec.whatwg.org/multipage/origin.html#ascii-serialisation-of-an-origin)
    /// of the origin.
    ///
    /// # Opaque Origins
    /// The [specification](https://html.spec.whatwg.org/multipage/origin.html) defines an Opaque Origin
    /// as one that cannot be recreated. You can refer to the source code for the [`url::Url::origin`]
    /// method to see how an Opaque Origin is determined. Examples of Opaque origins might include
    /// schemes like `file://` or Browser specific schemes like `"moz-extension://` or
    /// `chrome-extension://`.
    ///
    /// Opaque Origins cannot be matched exactly. You must use Regex to match Opaque Origins. If you
    /// attempt to create [`Cors`] from [`CorsOptions`], you will get an error.
    #[cfg_attr(feature = "serialization", serde(default))]
    pub exact: Option<HashSet<String>>,
    /// Origins that will be matched via __any__ regex in this list.
    ///
    /// These __must__ be valid Regex that will be parsed and validated when creating [`Cors`].
    ///
    /// The regex will be matched according to the
    /// [ASCII serialization](https://html.spec.whatwg.org/multipage/#ascii-serialisation-of-an-origin)
    /// of the incoming Origin.
    ///
    /// For more information on the syntax of Regex in Rust, see the
    /// [documentation](https://docs.rs/regex).
    ///
    /// Regular expressions are tested for matches against the
    /// [ASCII Serialization](https://html.spec.whatwg.org/multipage/origin.html#ascii-serialisation-of-an-origin)
    /// of the origin.
    ///
    /// # Warning about Regex expressions
    /// By default, regex expressions are
    /// [unanchored](https://docs.rs/regex/1.1.2/regex/struct.RegexSet.html#method.is_match).
    #[cfg_attr(feature = "serialization", serde(default))]
    pub regex: Option<HashSet<String>>,
}

/// Parsed set of configured allowed origins
#[derive(Clone, Debug)]
pub(crate) struct ParsedAllowedOrigins {
    pub allow_null: bool,
    pub exact: HashSet<url::Origin>,
    pub regex: Option<RegexSet>,
}

impl ParsedAllowedOrigins {
    fn parse(origins: &Origins) -> Result<Self, Error> {
        let exact: Result<Vec<(&str, url::Origin)>, Error> = match &origins.exact {
            Some(exact) => exact
                .iter()
                .map(|url| Ok((url.as_str(), to_origin(url.as_str())?)))
                .collect(),
            None => Ok(Default::default()),
        };
        let exact = exact?;

        // Let's check if they are Opaque
        let (tuple, opaque): (Vec<_>, Vec<_>) =
            exact.into_iter().partition(|(_, url)| url.is_tuple());

        if !opaque.is_empty() {
            return Err(Error::OpaqueAllowedOrigin(
                opaque
                    .into_iter()
                    .map(|(original, _)| original.to_string())
                    .collect(),
            ));
        }

        let exact = tuple.into_iter().map(|(_, url)| url).collect();

        let regex = match &origins.regex {
            None => None,
            Some(ref regex) => Some(RegexSet::new(regex)?),
        };

        Ok(Self {
            allow_null: origins.allow_null,
            exact,
            regex,
        })
    }

    fn verify(&self, origin: &Origin) -> bool {
        info_!("Verifying origin: {}", origin);
        match origin {
            Origin::Null => {
                info_!("Origin is null. Allowing? {}", self.allow_null);
                self.allow_null
            }
            Origin::Parsed(ref parsed) => {
                assert!(
                    parsed.is_tuple(),
                    "Parsed Origin is not tuple. This is a bug. Please report"
                );
                // Verify by exact, then regex
                if self.exact.get(parsed).is_some() {
                    info_!("Origin has an exact match");
                    return true;
                }
                if let Some(regex_set) = &self.regex {
                    let regex_match = regex_set.is_match(&parsed.ascii_serialization());
                    debug_!("Matching against regex set {:#?}", regex_set);
                    info_!("Origin has a regex match? {}", regex_match);
                    return regex_match;
                }

                info!("Origin does not match anything");
                false
            }
            Origin::Opaque(ref opaque) => {
                if let Some(regex_set) = &self.regex {
                    let regex_match = regex_set.is_match(opaque);
                    debug_!("Matching against regex set {:#?}", regex_set);
                    info_!("Origin has a regex match? {}", regex_match);
                    return regex_match;
                }

                info!("Origin does not match anything");
                false
            }
        }
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
        AllOrSome::Some(headers.iter().map(|s| (*s).to_string().into()).collect())
    }

    /// Allows all headers
    pub fn all() -> Self {
        AllOrSome::All
    }
}

/// Configuration options for CORS request handling.
///
/// You create a new copy of this struct by defining the configurations in the fields below.
/// This struct can also be deserialized by serde with the `serialization` feature which is
/// enabled by default.
///
/// [`Default`](https://doc.rust-lang.org/std/default/trait.Default.html) is implemented for this
/// struct. The default for each field is described in the documentation for the field.
///
/// Before you can use this with Rocket, you will need to call the [`CorsOptions::to_cors`] method.
///
/// # Examples
///
/// You can run an example from the repository to demonstrate the JSON serialization with
/// `cargo run --example json`.
///
/// ## Pure default
/// ```rust
/// let default = rocket_cors::CorsOptions::default();
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
///   "fairing_route_base": "/cors",
///   "fairing_route_rank": 0
/// }
/// ```
/// ### Defined
/// ```json
/// {
///   "allowed_origins": {
///     "Some": {
///         "exact": ["https://www.acme.com"],
///         "regex": ["^https://www.example-[A-z0-9]*.com$"]
///     }
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
pub struct CorsOptions {
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
    #[cfg_attr(feature = "serialization", serde(default))]
    pub allowed_origins: AllowedOrigins,
    /// The list of methods which the allowed origins are allowed to access for
    /// non-simple requests.
    ///
    /// This is the `list of methods` in the
    /// [Resource Processing Model](https://www.w3.org/TR/cors/#resource-processing-model).
    ///
    /// Defaults to `[GET, HEAD, POST, OPTIONS, PUT, PATCH, DELETE]`
    #[cfg_attr(
        feature = "serialization",
        serde(default = "CorsOptions::default_allowed_methods")
    )]
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
    pub allowed_headers: AllowedHeaders,
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
    /// When used as Fairing, Cors will need to redirect failed CORS checks to a custom route
    /// mounted by the fairing. Specify the base of the route so that it doesn't clash with any
    /// of your existing routes.
    ///
    /// Defaults to "/cors"
    #[cfg_attr(
        feature = "serialization",
        serde(default = "CorsOptions::default_fairing_route_base")
    )]
    pub fairing_route_base: String,
    /// When used as Fairing, Cors will need to redirect failed CORS checks to a custom route
    /// mounted by the fairing. Specify the rank of the route so that it doesn't clash with any
    /// of your existing routes. Remember that a higher ranked route has lower priority.
    ///
    /// Defaults to 0
    #[cfg_attr(
        feature = "serialization",
        serde(default = "CorsOptions::default_fairing_route_rank")
    )]
    pub fairing_route_rank: isize,
}

impl Default for CorsOptions {
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
            fairing_route_rank: Self::default_fairing_route_rank(),
        }
    }
}

impl CorsOptions {
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
        ]
        .into_iter()
        .map(From::from)
        .collect()
    }

    fn default_fairing_route_base() -> String {
        "/cors".to_string()
    }

    fn default_fairing_route_rank() -> isize {
        0
    }

    /// Validates if any of the settings are disallowed, incorrect, or illegal
    pub fn validate(&self) -> Result<(), Error> {
        if self.allowed_origins.is_all() && self.send_wildcard && self.allow_credentials {
            return Err(Error::CredentialsWithWildcardOrigin);
        }

        Ok(())
    }

    /// Creates a [`Cors`] struct that can be used to respond to requests or as a Rocket Fairing
    pub fn to_cors(&self) -> Result<Cors, Error> {
        Cors::from_options(self)
    }

    /// Sets the allowed origins
    #[must_use]
    pub fn allowed_origins(mut self, allowed_origins: AllowedOrigins) -> Self {
        self.allowed_origins = allowed_origins;
        self
    }

    /// Sets the allowed methods
    #[must_use]
    pub fn allowed_methods(mut self, allowed_methods: AllowedMethods) -> Self {
        self.allowed_methods = allowed_methods;
        self
    }

    /// Sets the allowed headers
    #[must_use]
    pub fn allowed_headers(mut self, allowed_headers: AllowedHeaders) -> Self {
        self.allowed_headers = allowed_headers;
        self
    }

    /// Marks if credentials are allowed
    #[must_use]
    pub fn allow_credentials(mut self, allow_credentials: bool) -> Self {
        self.allow_credentials = allow_credentials;
        self
    }

    /// Sets the expose headers
    #[must_use]
    pub fn expose_headers(mut self, expose_headers: HashSet<String>) -> Self {
        self.expose_headers = expose_headers;
        self
    }

    /// Sets the max age
    #[must_use]
    pub fn max_age(mut self, max_age: Option<usize>) -> Self {
        self.max_age = max_age;
        self
    }

    /// Marks if wildcards are send
    #[must_use]
    pub fn send_wildcard(mut self, send_wildcard: bool) -> Self {
        self.send_wildcard = send_wildcard;
        self
    }

    /// Sets the base of the fairing route
    #[must_use]
    pub fn fairing_route_base<S: Into<String>>(mut self, fairing_route_base: S) -> Self {
        self.fairing_route_base = fairing_route_base.into();
        self
    }

    /// Sets the rank of the fairing route
    #[must_use]
    pub fn fairing_route_rank(mut self, fairing_route_rank: isize) -> Self {
        self.fairing_route_rank = fairing_route_rank;
        self
    }
}

/// Response generator and [Fairing](https://rocket.rs/guide/fairings/) for CORS
///
/// This struct can be as Fairing or in an ad-hoc manner to generate CORS response. See the
/// documentation at the [crate root](index.html) for usage information.
///
/// This struct can be created by using [`CorsOptions::to_cors`] or [`Cors::from_options`].
#[derive(Clone, Debug)]
pub struct Cors {
    pub(crate) allowed_origins: AllOrSome<ParsedAllowedOrigins>,
    pub(crate) allowed_methods: AllowedMethods,
    pub(crate) allowed_headers: AllOrSome<HashSet<HeaderFieldName>>,
    pub(crate) allow_credentials: bool,
    pub(crate) expose_headers: HashSet<String>,
    pub(crate) max_age: Option<usize>,
    pub(crate) send_wildcard: bool,
    pub(crate) fairing_route_base: String,
    pub(crate) fairing_route_rank: isize,
}

impl Cors {
    /// Create a `Cors` struct from a [`CorsOptions`]
    pub fn from_options(options: &CorsOptions) -> Result<Self, Error> {
        options.validate()?;

        let allowed_origins = parse_allowed_origins(&options.allowed_origins)?;

        Ok(Cors {
            allowed_origins,
            allowed_methods: options.allowed_methods.clone(),
            allowed_headers: options.allowed_headers.clone(),
            allow_credentials: options.allow_credentials,
            expose_headers: options.expose_headers.clone(),
            max_age: options.max_age,
            send_wildcard: options.send_wildcard,
            fairing_route_base: options.fairing_route_base.clone(),
            fairing_route_rank: options.fairing_route_rank,
        })
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
    pub fn respond_owned<'r, 'o: 'r, F, R>(
        self,
        handler: F,
    ) -> Result<ManualResponder<'r, F, R>, Error>
    where
        F: FnOnce(Guard<'r>) -> R + 'r,
        R: response::Responder<'r, 'o>,
    {
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
    pub fn respond_borrowed<'r, 'o: 'r, F, R>(
        &'r self,
        handler: F,
    ) -> Result<ManualResponder<'r, F, R>, Error>
    where
        F: FnOnce(Guard<'r>) -> R + 'r,
        R: response::Responder<'r, 'o>,
    {
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
///
/// The following headers will be merged:
/// - `Vary`
///
/// You can get this struct by using `Cors::validate_request` in an ad-hoc manner.
#[derive(Eq, PartialEq, Debug)]
pub(crate) struct Response {
    allow_origin: Option<AllOrSome<String>>,
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
        self.expose_headers = headers.iter().map(|s| (*s).to_string().into()).collect();
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
        self.allow_headers = headers.iter().map(|s| (*s).to_string().into()).collect();
        self
    }

    /// Consumes the `Response` and return  a `Responder` that wraps a
    /// provided `rocket:response::Responder` with CORS headers
    pub fn responder<'r, 'o: 'r, R: response::Responder<'r, 'o>>(
        self,
        responder: R,
    ) -> Responder<R> {
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
    fn merge(&self, response: &mut response::Response<'_>) {
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

        let _ = response.set_raw_header("Access-Control-Allow-Origin", origin);

        if self.allow_credentials {
            let _ = response.set_raw_header("Access-Control-Allow-Credentials", "true");
        } else {
            response.remove_header("Access-Control-Allow-Credentials");
        }

        if !self.expose_headers.is_empty() {
            let headers: Vec<String> = self
                .expose_headers
                .iter()
                .map(|s| s.deref().to_string())
                .collect();
            let headers = headers.join(", ");

            let _ = response.set_raw_header("Access-Control-Expose-Headers", headers);
        } else {
            response.remove_header("Access-Control-Expose-Headers");
        }

        if !self.allow_headers.is_empty() {
            let headers: Vec<String> = self
                .allow_headers
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
            response.adjoin_raw_header("Vary", "Origin");
        }
    }

    /// Validate and create a new CORS Response from a request and settings
    pub fn validate_and_build<'a>(options: &'a Cors, request: &'a Request) -> Result<Self, Error> {
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

impl<'r, 'o: 'r> Guard<'r> {
    fn new(response: Response) -> Self {
        Self {
            response,
            marker: PhantomData,
        }
    }

    /// Consumes the Guard and return  a `Responder` that wraps a
    /// provided `rocket:response::Responder` with CORS headers
    pub fn responder<R: response::Responder<'r, 'o>>(self, responder: R) -> Responder<R> {
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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Guard<'r> {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let options = match request.guard::<&State<Cors>>().await {
            Outcome::Success(options) => options,
            _ => {
                let error = Error::MissingCorsInRocketState;
                return Outcome::Error((error.status(), error));
            }
        };

        match Response::validate_and_build(options, request) {
            Ok(response) => Outcome::Success(Self::new(response)),
            Err(error) => Outcome::Error((error.status(), error)),
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
///
/// The following headers will be merged:
/// - `Vary`
///
/// See the documentation at the [crate root](index.html) for usage information.
#[derive(Debug)]
pub struct Responder<R> {
    responder: R,
    cors_response: Response,
}

impl<'r, 'o: 'r, R: response::Responder<'r, 'o>> Responder<R> {
    fn new(responder: R, cors_response: Response) -> Self {
        Self {
            responder,
            cors_response,
            // marker: PhantomData,
        }
    }

    /// Respond to a request
    fn respond(self, request: &'r Request<'_>) -> response::Result<'o> {
        let mut response = self.responder.respond_to(request)?; // handle status errors?
        self.cors_response.merge(&mut response);
        Ok(response)
    }
}

impl<'r, 'o: 'r, R: response::Responder<'r, 'o>> response::Responder<'r, 'o> for Responder<R> {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'o> {
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

impl<'r, 'o: 'r, F, R> ManualResponder<'r, F, R>
where
    F: FnOnce(Guard<'r>) -> R + 'r,
    R: response::Responder<'r, 'o>,
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

    fn build_guard(&self, request: &Request<'_>) -> Result<Guard<'r>, Error> {
        let response = Response::validate_and_build(&self.options, request)?;
        Ok(Guard::new(response))
    }
}

impl<'r, 'o: 'r, F, R> response::Responder<'r, 'o> for ManualResponder<'r, F, R>
where
    F: FnOnce(Guard<'r>) -> R + 'r,
    R: response::Responder<'r, 'o>,
{
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'o> {
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
#[allow(variant_size_differences)]
enum ValidationResult {
    /// Not a CORS request
    None,
    /// Successful preflight request
    Preflight {
        origin: String,
        headers: Option<AccessControlRequestHeaders>,
    },
    /// Successful actual request
    Request { origin: String },
}

/// Convert a str to a URL Origin
fn to_origin<S: AsRef<str>>(origin: S) -> Result<url::Origin, Error> {
    Ok(url::Url::parse(origin.as_ref())?.origin())
}

/// Parse and process allowed origins
fn parse_allowed_origins(
    origins: &AllowedOrigins,
) -> Result<AllOrSome<ParsedAllowedOrigins>, Error> {
    match origins {
        AllOrSome::All => Ok(AllOrSome::All),
        AllOrSome::Some(origins) => {
            let parsed = ParsedAllowedOrigins::parse(origins)?;
            Ok(AllOrSome::Some(parsed))
        }
    }
}

/// Validates a request for CORS and returns a CORS Response
fn validate_and_build(options: &Cors, request: &Request<'_>) -> Result<Response, Error> {
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
fn validate(options: &Cors, request: &Request<'_>) -> Result<ValidationResult, Error> {
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
            Ok(ValidationResult::Preflight {
                origin: origin.to_string(),
                headers,
            })
        }
        _ => {
            actual_request_validate(options, &origin)?;
            Ok(ValidationResult::Request {
                origin: origin.to_string(),
            })
        }
    }
}

/// Consumes the responder and based on the provided list of allowed origins,
/// check if the requested origin is allowed.
/// Useful for pre-flight and during requests
fn validate_origin(
    origin: &Origin,
    allowed_origins: &AllOrSome<ParsedAllowedOrigins>,
) -> Result<(), Error> {
    match *allowed_origins {
        // Always matching is acceptable since the list of origins can be unbounded.
        AllOrSome::All => Ok(()),
        AllOrSome::Some(ref allowed_origins) => {
            if allowed_origins.verify(origin) {
                Ok(())
            } else {
                Err(Error::OriginNotAllowed(origin.to_string()))
            }
        }
    }
}

/// Validate allowed methods
fn validate_allowed_method(
    method: &AccessControlRequestMethod,
    allowed_methods: &AllowedMethods,
) -> Result<(), Error> {
    let AccessControlRequestMethod(request_method) = method;
    if !allowed_methods.iter().any(|m| m == request_method) {
        return Err(Error::MethodNotAllowed(method.0.to_string()));
    }

    // TODO: Subset to route? Or just the method requested for?
    Ok(())
}

/// Validate allowed headers
fn validate_allowed_headers(
    headers: &AccessControlRequestHeaders,
    allowed_headers: &AllowedHeaders,
) -> Result<(), Error> {
    let AccessControlRequestHeaders(headers) = headers;

    match *allowed_headers {
        AllOrSome::All => Ok(()),
        AllOrSome::Some(ref allowed_headers) => {
            if !headers.is_empty() && !headers.is_subset(allowed_headers) {
                return Err(Error::HeadersNotAllowed);
            }
            Ok(())
        }
    }
}

/// Gets the `Origin` request header from the request
fn origin(request: &Request<'_>) -> Result<Option<Origin>, Error> {
    match Origin::from_request_sync(request) {
        Outcome::Forward(_) => Ok(None),
        Outcome::Success(origin) => Ok(Some(origin)),
        Outcome::Error((_, err)) => Err(err),
    }
}

/// Gets the `Access-Control-Request-Method` request header from the request
fn request_method(request: &Request<'_>) -> Result<Option<AccessControlRequestMethod>, Error> {
    match AccessControlRequestMethod::from_request_sync(request) {
        Outcome::Forward(_) => Ok(None),
        Outcome::Success(method) => Ok(Some(method)),
        Outcome::Error((_, err)) => Err(err),
    }
}

/// Gets the `Access-Control-Request-Headers` request header from the request
fn request_headers(request: &Request<'_>) -> Result<Option<AccessControlRequestHeaders>, Error> {
    match AccessControlRequestHeaders::from_request_sync(request) {
        Outcome::Forward(_) => Ok(None),
        Outcome::Success(geaders) => Ok(Some(geaders)),
        Outcome::Error((_, err)) => Err(err),
    }
}

/// Do pre-flight validation checks
///
/// This implementation references the
/// [W3C recommendation](https://www.w3.org/TR/cors/#resource-preflight-requests)
/// and [Fetch specification](https://fetch.spec.whatwg.org/#cors-preflight-fetch)
fn preflight_validate(
    options: &Cors,
    origin: &Origin,
    method: &Option<AccessControlRequestMethod>,
    headers: &Option<AccessControlRequestHeaders>,
) -> Result<(), Error> {
    // Note: All header parse failures are dealt with in the `FromRequest` trait implementation

    // 2. If the value of the Origin header is not a case-sensitive match for any of the values
    // in list of origins do not set any additional headers and terminate this set of steps.
    validate_origin(origin, &options.allowed_origins)?;

    // 3. Let `method` be the value as result of parsing the Access-Control-Request-Method
    // header.
    // If there is no Access-Control-Request-Method header or if parsing failed,
    // do not set any additional headers and terminate this set of steps.
    // The request is outside the scope of this specification.

    let method = method.as_ref().ok_or(Error::MissingRequestMethod)?;

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
/// [W3C recommendation](https://www.w3.org/TR/cors/#resource-preflight-requests)
/// and [Fetch specification](https://fetch.spec.whatwg.org/#cors-preflight-fetch).
fn preflight_response(
    options: &Cors,
    origin: &str,
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
        let AccessControlRequestHeaders(headers) = headers;
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
/// [W3C recommendation](https://www.w3.org/TR/cors/#resource-requests)
/// and [Fetch specification](https://fetch.spec.whatwg.org/#cors-preflight-fetch).
fn actual_request_validate(options: &Cors, origin: &Origin) -> Result<(), Error> {
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
/// [W3C recommendation](https://www.w3.org/TR/cors/#resource-requests)
/// and [Fetch specification](https://fetch.spec.whatwg.org/#cors-preflight-fetch)
fn actual_request_response(options: &Cors, origin: &str) -> Response {
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
    vec![rocket::Route::ranked(
        isize::MAX,
        http::Method::Options,
        "/<catch_all_options_route..>",
        CatchAllOptionsRouteHandler {},
    )]
}

/// Handler for the "catch all options route"
#[derive(Clone)]
struct CatchAllOptionsRouteHandler {}

#[rocket::async_trait]
impl rocket::route::Handler for CatchAllOptionsRouteHandler {
    async fn handle<'r>(
        &self,
        request: &'r Request<'_>,
        _: rocket::Data<'r>,
    ) -> rocket::route::Outcome<'r> {
        let guard: Guard<'_> = match request.guard().await {
            Outcome::Success(guard) => guard,
            Outcome::Error((status, _)) => return rocket::route::Outcome::Error(status),
            Outcome::Forward(_) => unreachable!("Should not be reachable"),
        };

        info_!(
            "\"Catch all\" handling of CORS `OPTIONS` preflight for request {}",
            request
        );

        rocket::route::Outcome::from(request, guard.responder(()))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rocket::http::hyper;
    use rocket::http::Header;
    use rocket::local::blocking::Client;

    use super::*;
    use crate::http::Method;

    static ORIGIN: ::http::header::HeaderName = hyper::header::ORIGIN;
    static ACCESS_CONTROL_REQUEST_METHOD: ::http::header::HeaderName =
        hyper::header::ACCESS_CONTROL_REQUEST_METHOD;
    static ACCESS_CONTROL_REQUEST_HEADERS: ::http::header::HeaderName =
        hyper::header::ACCESS_CONTROL_REQUEST_HEADERS;

    fn to_parsed_origin<S: AsRef<str>>(origin: S) -> Result<Origin, Error> {
        Origin::from_str(origin.as_ref())
    }

    fn make_cors_options() -> CorsOptions {
        let allowed_origins = AllowedOrigins::some_exact(&["https://www.acme.com"]);

        CorsOptions {
            allowed_origins,
            allowed_methods: vec![http::Method::Get]
                .into_iter()
                .map(From::from)
                .collect(),
            allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
            allow_credentials: true,
            expose_headers: ["Content-Type", "X-Custom"]
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            ..Default::default()
        }
    }

    fn make_invalid_options() -> CorsOptions {
        let mut cors = make_cors_options();
        cors.allow_credentials = true;
        cors.allowed_origins = AllOrSome::All;
        cors.send_wildcard = true;
        cors
    }

    /// Make a client with no routes for unit testing
    fn make_client() -> Client {
        let rocket = rocket::build();
        Client::tracked(rocket).expect("valid rocket instance")
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

    #[test]
    fn cors_options_from_builder_pattern() {
        let allowed_origins = AllowedOrigins::some_exact(&["https://www.acme.com"]);
        let cors_options_from_builder = CorsOptions::default()
            .allowed_origins(allowed_origins)
            .allowed_methods(
                vec![http::Method::Get]
                    .into_iter()
                    .map(From::from)
                    .collect(),
            )
            .allowed_headers(AllowedHeaders::some(&["Authorization", "Accept"]))
            .allow_credentials(true)
            .expose_headers(
                ["Content-Type", "X-Custom"]
                    .iter()
                    .map(|s| (*s).to_string())
                    .collect(),
            );
        assert_eq!(cors_options_from_builder, make_cors_options());
    }

    /// Check that the the default deserialization matches the one returned by `Default::default`
    #[cfg(feature = "serialization")]
    #[test]
    fn cors_default_deserialization_is_correct() {
        let deserialized: CorsOptions = serde_json::from_str("{}").expect("To not fail");
        assert_eq!(deserialized, CorsOptions::default());

        let expected_json = r#"
{
  "allowed_origins": "All",
  "allowed_methods": [
    "POST",
    "PATCH",
    "PUT",
    "DELETE",
    "HEAD",
    "OPTIONS",
    "GET"
  ],
  "allowed_headers": "All",
  "allow_credentials": false,
  "expose_headers": [],
  "max_age": null,
  "send_wildcard": false,
  "fairing_route_base": "/cors",
  "fairing_route_rank": 0
}
"#;
        let actual: CorsOptions = serde_json::from_str(expected_json).expect("to not fail");
        assert_eq!(actual, CorsOptions::default());
    }

    /// Checks that the example provided can actually be deserialized
    #[cfg(feature = "serialization")]
    #[test]
    fn cors_options_example_can_be_deserialized() {
        let json = r#"{
  "allowed_origins": {
    "Some": {
        "exact": ["https://www.acme.com"],
        "regex": ["^https://www.example-[A-z0-9]*.com$"]
    }
  },
  "allowed_methods": [
    "POST",
    "DELETE",
    "GET"
  ],
  "allowed_headers": {
    "Some": [
      "Accept",
      "Authorization"
    ]
  },
  "allow_credentials": true,
  "expose_headers": [
    "Content-Type",
    "X-Custom"
  ],
  "max_age": 42,
  "send_wildcard": false,
  "fairing_route_base": "/mycors"
}"#;
        let _: CorsOptions = serde_json::from_str(json).expect("to not fail");
    }

    #[test]
    fn allowed_some_origins_allows_different_lifetimes() {
        let static_exact = ["http://www.example.com"];

        let random_allocation = vec![1, 2, 3];
        let port: *const Vec<i32> = &random_allocation;
        let port = port as u16;

        let random_regex = vec![format!("https://(.+):{}", port)];

        // Should compile
        let _ = AllowedOrigins::some(&static_exact, &random_regex);
    }

    // `ParsedAllowedOrigins::parse` tests
    #[test]
    fn allowed_origins_are_parsed_correctly() {
        let allowed_origins = not_err!(parse_allowed_origins(&AllowedOrigins::some(
            &["https://www.acme.com"],
            &["^https://www.example-[A-z0-9]+.com$"]
        )));
        assert!(allowed_origins.is_some());

        let expected_exact: HashSet<url::Origin> = [url::Url::from_str("https://www.acme.com")
            .expect("not to fail")
            .origin()]
        .iter()
        .map(Clone::clone)
        .collect();
        let expected_regex = ["^https://www.example-[A-z0-9]+.com$"];

        let actual = allowed_origins.unwrap();
        assert_eq!(expected_exact, actual.exact);
        assert_eq!(expected_regex, actual.regex.expect("to be some").patterns());
    }

    #[test]
    fn allowed_origins_errors_on_opaque_exact() {
        let error = parse_allowed_origins(&AllowedOrigins::some::<_, &str>(
            &[
                "chrome-extension://something",
                "moz-extension://something",
                "https://valid.com",
            ],
            &[],
        ))
        .unwrap_err();

        match error {
            Error::OpaqueAllowedOrigin(mut origins) => {
                origins.sort();
                assert_eq!(
                    origins,
                    ["chrome-extension://something", "moz-extension://something"]
                );
            }
            others => {
                panic!("Unexpected error: {:#?}", others);
            }
        };
    }

    // The following tests check validation

    #[test]
    fn validate_origin_allows_all_origins() {
        let url = "https://www.example.com";
        let origin = not_err!(to_parsed_origin(url));
        let allowed_origins = AllOrSome::All;

        not_err!(validate_origin(&origin, &allowed_origins));
    }

    #[test]
    fn validate_origin_allows_origin() {
        let url = "https://www.example.com";
        let origin = not_err!(to_parsed_origin(url));
        let allowed_origins = not_err!(parse_allowed_origins(&AllowedOrigins::some_exact(&[
            "https://www.example.com"
        ])));

        not_err!(validate_origin(&origin, &allowed_origins));
    }

    #[test]
    fn validate_origin_handles_punycode_properly() {
        // Test a variety of scenarios where the Origin and settings are in punycode, or not
        let cases = vec![
            ("https://аpple.com", "https://аpple.com"),
            ("https://аpple.com", "https://xn--pple-43d.com"),
            ("https://xn--pple-43d.com", "https://аpple.com"),
            ("https://xn--pple-43d.com", "https://xn--pple-43d.com"),
        ];

        for (url, allowed_origin) in cases {
            let origin = not_err!(to_parsed_origin(url));
            let allowed_origins = not_err!(parse_allowed_origins(&AllowedOrigins::some_exact(&[
                allowed_origin
            ])));

            not_err!(validate_origin(&origin, &allowed_origins));
        }
    }

    #[test]
    fn validate_origin_validates_regex() {
        let allowed_origins = not_err!(parse_allowed_origins(&AllowedOrigins::some_regex(&[
            "^https://www.example-[A-z0-9]+.com$",
            "^https://(.+).acme.com$",
        ])));

        let url = "https://www.example-something.com";
        let origin = not_err!(to_parsed_origin(url));
        not_err!(validate_origin(&origin, &allowed_origins));

        let url = "https://subdomain.acme.com";
        let origin = not_err!(to_parsed_origin(url));
        not_err!(validate_origin(&origin, &allowed_origins));
    }

    #[test]
    fn validate_origin_validates_opaque_origins() {
        let url = "moz-extension://8c7c4444-e29f-…cb8-1ade813dbd12/js/content.js:505";
        let origin = not_err!(to_parsed_origin(url));
        let allowed_origins = not_err!(parse_allowed_origins(&AllowedOrigins::some_regex(&[
            "moz-extension://.*"
        ])));

        not_err!(validate_origin(&origin, &allowed_origins));
    }

    #[test]
    fn validate_origin_validates_mixed_settings() {
        let allowed_origins = not_err!(parse_allowed_origins(&AllowedOrigins::some(
            &["https://www.acme.com"],
            &["^https://www.example-[A-z0-9]+.com$"]
        )));

        let url = "https://www.example-something123.com";
        let origin = not_err!(to_parsed_origin(url));
        not_err!(validate_origin(&origin, &allowed_origins));

        let url = "https://www.acme.com";
        let origin = not_err!(to_parsed_origin(url));
        not_err!(validate_origin(&origin, &allowed_origins));
    }

    #[test]
    #[should_panic(expected = "OriginNotAllowed")]
    fn validate_origin_rejects_invalid_origin() {
        let url = "https://www.acme.com";
        let origin = not_err!(to_parsed_origin(url));
        let allowed_origins = not_err!(parse_allowed_origins(&AllowedOrigins::some_exact(&[
            "https://www.example.com"
        ])));

        validate_origin(&origin, &allowed_origins).unwrap();
    }

    #[test]
    fn response_sets_allow_origin_without_vary_correctly() {
        let response = Response::new();
        let response = response.origin("https://www.example.com", false);

        // Build response and check built response header
        let expected_header = vec!["https://www.example.com"];
        let response = response.response(response::Response::new());
        let actual_header: Vec<_> = response
            .headers()
            .get("Access-Control-Allow-Origin")
            .collect();
        assert_eq!(expected_header, actual_header);

        assert!(response.headers().get("Vary").next().is_none());
    }

    #[test]
    fn response_sets_allow_origin_with_vary_correctly() {
        let response = Response::new();
        let response = response.origin("https://www.example.com", true);

        // Build response and check built response header
        let expected_header = vec!["https://www.example.com"];
        let response = response.response(response::Response::new());
        let actual_header: Vec<_> = response
            .headers()
            .get("Access-Control-Allow-Origin")
            .collect();
        assert_eq!(expected_header, actual_header);
    }

    #[test]
    fn response_sets_any_origin_correctly() {
        let response = Response::new();
        let response = response.any();

        // Build response and check built response header
        let expected_header = vec!["*"];
        let response = response.response(response::Response::new());
        let actual_header: Vec<_> = response
            .headers()
            .get("Access-Control-Allow-Origin")
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
        let response = response.origin("https://www.example.com", false);

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
        let response = response.origin("https://www.example.com", false);

        let response = response.max_age(None);

        // Build response and check built response header
        let response = response.response(response::Response::new());
        assert!(response
            .headers()
            .get("Access-Control-Max-Age")
            .next()
            .is_none())
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
        )
        .unwrap()
    }

    #[test]
    fn all_allowed_headers_are_validated_correctly() {
        let allowed_headers = AllOrSome::All;
        let requested_headers = ["Bar", "Foo"];

        not_err!(validate_allowed_headers(
            &FromStr::from_str(&requested_headers.join(",")).unwrap(),
            &allowed_headers,
        ));
    }

    /// `Response::allowed_headers` should check that headers are allowed, and only
    /// echoes back the list that is actually requested for and not the whole list
    #[test]
    fn allowed_headers_are_validated_correctly() {
        let allowed_headers = ["Bar", "Baz", "Foo"];
        let requested_headers = ["Bar", "Foo"];

        not_err!(validate_allowed_headers(
            &FromStr::from_str(&requested_headers.join(",")).unwrap(),
            &AllOrSome::Some(
                allowed_headers
                    .iter()
                    .map(|s| FromStr::from_str(s).unwrap())
                    .collect(),
            ),
        ));
    }

    #[test]
    #[should_panic(expected = "HeadersNotAllowed")]
    fn allowed_headers_errors_on_non_subset() {
        let allowed_headers = ["Bar", "Baz", "Foo"];
        let requested_headers = ["Bar", "Foo", "Unknown"];

        validate_allowed_headers(
            &FromStr::from_str(&requested_headers.join(",")).unwrap(),
            &AllOrSome::Some(
                allowed_headers
                    .iter()
                    .map(|s| FromStr::from_str(s).unwrap())
                    .collect(),
            ),
        )
        .unwrap();
    }

    #[test]
    fn response_does_not_build_if_origin_is_not_set() {
        let response = Response::new();
        let response = response.response(response::Response::new());

        assert_eq!(response.headers().iter().count(), 0);
    }

    #[test]
    fn response_build_removes_existing_cors_headers_and_keeps_others() {
        use std::io::Cursor;

        let body = "Brewing the best coffee!";
        let original = response::Response::build()
            .status(Status::ImATeapot)
            .raw_header("X-Teapot-Make", "Rocket")
            .raw_header("Access-Control-Max-Age", "42")
            .sized_body(body.len(), Cursor::new(body))
            .finalize();

        let response = Response::new();
        let response = response.origin("https://www.example.com", false);
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
        assert!(response
            .headers()
            .get("Access-Control-Max-Age")
            .next()
            .is_none());
    }

    #[derive(Debug, Eq, PartialEq)]
    #[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
    struct MethodTest {
        method: crate::Method,
    }

    #[cfg(feature = "serialization")]
    #[test]
    fn method_serde_roundtrip() {
        use serde_test::{assert_tokens, Token};

        let test = MethodTest {
            method: From::from(http::Method::Get),
        };

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
        let cors = make_cors_options().to_cors().expect("To not fail");
        let client = make_client();

        let origin_header = Header::new(ORIGIN.as_str(), "https://www.acme.com");
        let method_header = Header::new(
            ACCESS_CONTROL_REQUEST_METHOD.as_str(),
            hyper::Method::GET.as_str(),
        );
        let request_headers = Header::new(ACCESS_CONTROL_REQUEST_HEADERS.as_str(), "Authorization");

        let request = client
            .options("/")
            .header(origin_header)
            .header(method_header)
            .header(request_headers);

        let result = validate(&cors, request.inner()).expect("to not fail");
        let expected_result = ValidationResult::Preflight {
            origin: "https://www.acme.com".to_string(),
            // Checks that only a subset of allowed headers are returned
            // -- i.e. whatever is requested for
            headers: Some(FromStr::from_str("Authorization").unwrap()),
        };

        assert_eq!(expected_result, result);
    }

    #[test]
    fn preflight_validation_allows_all_origin() {
        let mut options = make_cors_options();
        options.allowed_origins = AllOrSome::All;
        let cors = options.to_cors().expect("To not fail");
        let client = make_client();

        let origin_header = Header::new(ORIGIN.as_str(), "https://www.example.com");
        let method_header = Header::new(
            ACCESS_CONTROL_REQUEST_METHOD.as_str(),
            hyper::Method::GET.as_str(),
        );
        let request_headers = Header::new(ACCESS_CONTROL_REQUEST_HEADERS.as_str(), "Authorization");

        let request = client
            .options("/")
            .header(origin_header)
            .header(method_header)
            .header(request_headers);

        let result = validate(&cors, request.inner()).expect("to not fail");
        let expected_result = ValidationResult::Preflight {
            origin: "https://www.example.com".to_string(),
            headers: Some(FromStr::from_str("Authorization").unwrap()),
        };

        assert_eq!(expected_result, result);
    }

    #[test]
    #[should_panic(expected = "OriginNotAllowed")]
    fn preflight_validation_errors_on_invalid_origin() {
        let cors = make_cors_options().to_cors().expect("To not fail");
        let client = make_client();

        let origin_header = Header::new(ORIGIN.as_str(), "https://www.example.com");
        let method_header = Header::new(
            ACCESS_CONTROL_REQUEST_METHOD.as_str(),
            hyper::Method::GET.as_str(),
        );
        let request_headers = Header::new(ACCESS_CONTROL_REQUEST_HEADERS.as_str(), "Authorization");

        let request = client
            .options("/")
            .header(origin_header)
            .header(method_header)
            .header(request_headers);

        let _ = validate(&cors, request.inner()).unwrap();
    }

    #[test]
    #[should_panic(expected = "MissingRequestMethod")]
    fn preflight_validation_errors_on_missing_request_method() {
        let cors = make_cors_options().to_cors().expect("To not fail");
        let client = make_client();

        let origin_header = Header::new(ORIGIN.as_str(), "https://www.acme.com");
        let request_headers = Header::new(ACCESS_CONTROL_REQUEST_HEADERS.as_str(), "Authorization");

        let request = client
            .options("/")
            .header(origin_header)
            .header(request_headers);

        let _ = validate(&cors, request.inner()).unwrap();
    }

    #[test]
    #[should_panic(expected = "MethodNotAllowed")]
    fn preflight_validation_errors_on_disallowed_method() {
        let cors = make_cors_options().to_cors().expect("To not fail");
        let client = make_client();

        let origin_header = Header::new(ORIGIN.as_str(), "https://www.acme.com");
        let method_header = Header::new(
            ACCESS_CONTROL_REQUEST_METHOD.as_str(),
            hyper::Method::POST.as_str(),
        );
        let request_headers = Header::new(ACCESS_CONTROL_REQUEST_HEADERS.as_str(), "Authorization");

        let request = client
            .options("/")
            .header(origin_header)
            .header(method_header)
            .header(request_headers);

        let _ = validate(&cors, request.inner()).unwrap();
    }

    #[test]
    #[should_panic(expected = "HeadersNotAllowed")]
    fn preflight_validation_errors_on_disallowed_headers() {
        let cors = make_cors_options().to_cors().expect("To not fail");
        let client = make_client();

        let origin_header = Header::new(ORIGIN.as_str(), "https://www.acme.com");
        let method_header = Header::new(
            ACCESS_CONTROL_REQUEST_METHOD.as_str(),
            hyper::Method::GET.as_str(),
        );
        let request_headers = Header::new(
            ACCESS_CONTROL_REQUEST_HEADERS.as_str(),
            "Authorization, X-NOT-ALLOWED",
        );

        let request = client
            .options("/")
            .header(origin_header)
            .header(method_header)
            .header(request_headers);

        let _ = validate(&cors, request.inner()).unwrap();
    }

    #[test]
    fn actual_request_validated_correctly() {
        let cors = make_cors_options().to_cors().expect("To not fail");
        let client = make_client();

        let origin_header = Header::new(ORIGIN.as_str(), "https://www.acme.com");
        let request = client.get("/").header(origin_header);

        let result = validate(&cors, request.inner()).expect("to not fail");
        let expected_result = ValidationResult::Request {
            origin: "https://www.acme.com".to_string(),
        };

        assert_eq!(expected_result, result);
    }

    #[test]
    fn actual_request_validation_allows_all_origin() {
        let mut options = make_cors_options();
        options.allowed_origins = AllOrSome::All;
        let cors = options.to_cors().expect("To not fail");
        let client = make_client();

        let origin_header = Header::new(ORIGIN.as_str(), "https://www.example.com");
        let request = client.get("/").header(origin_header);

        let result = validate(&cors, request.inner()).expect("to not fail");
        let expected_result = ValidationResult::Request {
            origin: "https://www.example.com".to_string(),
        };

        assert_eq!(expected_result, result);
    }

    #[test]
    #[should_panic(expected = "OriginNotAllowed")]
    fn actual_request_validation_errors_on_incorrect_origin() {
        let cors = make_cors_options().to_cors().expect("To not fail");
        let client = make_client();

        let origin_header = Header::new(ORIGIN.as_str(), "https://www.example.com");
        let request = client.get("/").header(origin_header);

        let _ = validate(&cors, request.inner()).unwrap();
    }

    #[test]
    fn non_cors_request_return_empty_response() {
        let cors = make_cors_options().to_cors().expect("To not fail");
        let client = make_client();

        let request = client.options("/");
        let response = validate_and_build(&cors, request.inner()).expect("to not fail");
        let expected_response = Response::new();
        assert_eq!(expected_response, response);
    }

    #[test]
    fn preflight_validated_and_built_correctly() {
        let options = make_cors_options();
        let cors = options.to_cors().expect("To not fail");
        let client = make_client();

        let origin_header = Header::new(ORIGIN.as_str(), "https://www.acme.com");
        let method_header = Header::new(
            ACCESS_CONTROL_REQUEST_METHOD.as_str(),
            hyper::Method::GET.as_str(),
        );
        let request_headers = Header::new(ACCESS_CONTROL_REQUEST_HEADERS.as_str(), "Authorization");

        let request = client
            .options("/")
            .header(origin_header)
            .header(method_header)
            .header(request_headers);

        let response = validate_and_build(&cors, request.inner()).expect("to not fail");

        let expected_response = Response::new()
            .origin("https://www.acme.com", false)
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
        let cors = options.to_cors().expect("To not fail");

        let client = make_client();

        let origin_header = Header::new(ORIGIN.as_str(), "https://www.acme.com");
        let method_header = Header::new(
            ACCESS_CONTROL_REQUEST_METHOD.as_str(),
            hyper::Method::GET.as_str(),
        );
        let request_headers = Header::new(ACCESS_CONTROL_REQUEST_HEADERS.as_str(), "Authorization");

        let request = client
            .options("/")
            .header(origin_header)
            .header(method_header)
            .header(request_headers);

        let response = validate_and_build(&cors, request.inner()).expect("to not fail");

        let expected_response = Response::new()
            .origin("https://www.acme.com", true)
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
        let cors = options.to_cors().expect("To not fail");

        let client = make_client();

        let origin_header = Header::new(ORIGIN.as_str(), "https://www.acme.com");
        let method_header = Header::new(
            ACCESS_CONTROL_REQUEST_METHOD.as_str(),
            hyper::Method::GET.as_str(),
        );
        let request_headers = Header::new(ACCESS_CONTROL_REQUEST_HEADERS.as_str(), "Authorization");

        let request = client
            .options("/")
            .header(origin_header)
            .header(method_header)
            .header(request_headers);

        let response = validate_and_build(&cors, request.inner()).expect("to not fail");

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
        let cors = options.to_cors().expect("To not fail");
        let client = make_client();

        let origin_header = Header::new(ORIGIN.as_str(), "https://www.acme.com");
        let request = client.get("/").header(origin_header);

        let response = validate_and_build(&cors, request.inner()).expect("to not fail");
        let expected_response = Response::new()
            .origin("https://www.acme.com", false)
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
        let cors = options.to_cors().expect("To not fail");

        let client = make_client();

        let origin_header = Header::new(ORIGIN.as_str(), "https://www.acme.com");
        let request = client.get("/").header(origin_header);

        let response = validate_and_build(&cors, request.inner()).expect("to not fail");
        let expected_response = Response::new()
            .origin("https://www.acme.com", true)
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
        let cors = options.to_cors().expect("To not fail");

        let client = make_client();

        let origin_header = Header::new(ORIGIN.as_str(), "https://www.acme.com");
        let request = client.get("/").header(origin_header);

        let response = validate_and_build(&cors, request.inner()).expect("to not fail");
        let expected_response = Response::new()
            .any()
            .credentials(options.allow_credentials)
            .exposed_headers(&["Content-Type", "X-Custom"]);

        assert_eq!(expected_response, response);
    }
}
