//! CORS specific Request Headers

use std::collections::HashSet;
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{self, Outcome};
#[cfg(feature = "serialization")]
use serde_derive::{Deserialize, Serialize};
use unicase::UniCase;

#[cfg(feature = "serialization")]
use unicase_serde;

/// A case insensitive header name
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct HeaderFieldName(
    #[cfg_attr(feature = "serialization", serde(with = "unicase_serde::unicase"))] UniCase<String>,
);

impl Deref for HeaderFieldName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl fmt::Display for HeaderFieldName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> From<&'a str> for HeaderFieldName {
    fn from(s: &'a str) -> Self {
        HeaderFieldName(From::from(s))
    }
}

impl<'a> From<String> for HeaderFieldName {
    fn from(s: String) -> Self {
        HeaderFieldName(From::from(s))
    }
}

impl FromStr for HeaderFieldName {
    type Err = <String as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(HeaderFieldName(FromStr::from_str(s)?))
    }
}

/// A set of case insensitive header names
pub type HeaderFieldNamesSet = HashSet<HeaderFieldName>;

/// The `Origin` request header used in CORS
///
/// You can use this as a rocket [Request Guard](https://rocket.rs/guide/requests/#request-guards)
/// to ensure that `Origin` is passed in correctly.
///
/// Reference: [Mozilla](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Origin)
#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub enum Origin {
    /// A `null` Origin
    Null,
    /// A well-formed origin that was parsed by [`url::Url::origin`]
    Parsed(url::Origin),
    /// An unknown "opaque" origin that could not be parsed
    Opaque(String),
}

impl Origin {
    /// Perform an
    /// [ASCII serialization](https://html.spec.whatwg.org/multipage/#ascii-serialisation-of-an-origin)
    /// of this origin.
    pub fn ascii_serialization(&self) -> String {
        self.to_string()
    }

    /// Returns whether the origin was parsed as non-opaque
    pub fn is_tuple(&self) -> bool {
        match self {
            Origin::Null => false,
            Origin::Parsed(ref parsed) => parsed.is_tuple(),
            Origin::Opaque(_) => false,
        }
    }
}

impl FromStr for Origin {
    type Err = crate::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.to_lowercase() == "null" {
            Ok(Origin::Null)
        } else {
            match crate::to_origin(input)? {
                url::Origin::Opaque(_) => Ok(Origin::Opaque(input.to_string())),
                parsed @ url::Origin::Tuple(..) => Ok(Origin::Parsed(parsed)),
            }
        }
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Origin::Null => write!(f, "null"),
            Origin::Parsed(ref parsed) => write!(f, "{}", parsed.ascii_serialization()),
            Origin::Opaque(ref opaque) => write!(f, "{}", opaque),
        }
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Origin {
    type Error = crate::Error;

    fn from_request(request: &'a rocket::Request<'r>) -> request::Outcome<Self, crate::Error> {
        match request.headers().get_one("Origin") {
            Some(origin) => match Self::from_str(origin) {
                Ok(origin) => Outcome::Success(origin),
                Err(e) => Outcome::Failure((Status::BadRequest, e)),
            },
            None => Outcome::Forward(()),
        }
    }
}
/// The `Access-Control-Request-Method` request header
///
/// You can use this as a rocket [Request Guard](https://rocket.rs/guide/requests/#request-guards)
/// to ensure that the header is passed in correctly.
#[derive(Debug)]
pub struct AccessControlRequestMethod(pub crate::Method);

impl FromStr for AccessControlRequestMethod {
    type Err = ();

    fn from_str(method: &str) -> Result<Self, Self::Err> {
        Ok(AccessControlRequestMethod(crate::Method::from_str(method)?))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for AccessControlRequestMethod {
    type Error = crate::Error;

    fn from_request(request: &'a rocket::Request<'r>) -> request::Outcome<Self, crate::Error> {
        match request.headers().get_one("Access-Control-Request-Method") {
            Some(request_method) => match Self::from_str(request_method) {
                Ok(request_method) => Outcome::Success(request_method),
                Err(_) => Outcome::Failure((Status::BadRequest, crate::Error::BadRequestMethod)),
            },
            None => Outcome::Forward(()),
        }
    }
}

/// The `Access-Control-Request-Headers` request header
///
/// You can use this as a rocket [Request Guard](https://rocket.rs/guide/requests/#request-guards)
/// to ensure that the header is passed in correctly.
#[derive(Eq, PartialEq, Debug)]
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
            .map(|header| From::from(header.trim().to_string()))
            .collect();
        Ok(AccessControlRequestHeaders(set))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for AccessControlRequestHeaders {
    type Error = crate::Error;

    fn from_request(request: &'a rocket::Request<'r>) -> request::Outcome<Self, crate::Error> {
        match request.headers().get_one("Access-Control-Request-Headers") {
            Some(request_headers) => match Self::from_str(request_headers) {
                Ok(request_headers) => Outcome::Success(request_headers),
                Err(()) => {
                    unreachable!("`AccessControlRequestHeaders::from_str` should never fail")
                }
            },
            None => Outcome::Forward(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use hyper;
    use rocket;
    use rocket::local::Client;

    use super::*;

    /// Make a client with no routes for unit testing
    fn make_client() -> Client {
        let rocket = rocket::ignite();
        Client::new(rocket).expect("valid rocket instance")
    }

    // `Origin::from_str` tests

    #[test]
    fn origin_is_parsed_properly() {
        let url = "https://foo.bar.xyz";
        let parsed = not_err!(Origin::from_str(url));
        assert_eq!(parsed.ascii_serialization(), url);
    }

    #[test]
    fn origin_parsing_strips_paths() {
        // this should never really be sent by a compliant user agent
        let url = "https://foo.bar.xyz/path/somewhere";
        let parsed = not_err!(Origin::from_str(url));
        let expected = "https://foo.bar.xyz";
        assert_eq!(parsed.ascii_serialization(), expected);
    }

    #[test]
    #[should_panic(expected = "BadOrigin")]
    fn origin_parsing_disallows_invalid_origins() {
        let url = "invalid_url";
        let _ = Origin::from_str(url).unwrap();
    }

    #[test]
    fn origin_parses_opaque_origins() {
        let url = "blob://foobar";
        let parsed = not_err!(Origin::from_str(url));

        assert!(!parsed.is_tuple());
    }

    // The following tests check that CORS Request headers are parsed correctly

    #[test]
    fn origin_header_conversion() {
        let url = "https://foo.bar.xyz";
        let parsed = not_err!(Origin::from_str(url));
        assert_eq!(parsed.ascii_serialization(), url);

        let url = "https://foo.bar.xyz:1234";
        let parsed = not_err!(Origin::from_str(url));
        assert_eq!(parsed.ascii_serialization(), url);

        // this should never really be sent by a compliant user agent
        let url = "https://foo.bar.xyz/path/somewhere";
        let parsed = not_err!(Origin::from_str(url));
        let expected = "https://foo.bar.xyz";
        assert_eq!(parsed.ascii_serialization(), expected);

        let url = "invalid_url";
        let _ = is_err!(Origin::from_str(url));
    }

    #[test]
    fn origin_header_parsing() {
        let client = make_client();
        let mut request = client.get("/");

        let origin = hyper::header::Origin::new("https", "www.example.com", None);
        request.add_header(origin);

        let outcome: request::Outcome<Origin, crate::Error> =
            FromRequest::from_request(request.inner());
        let parsed_header = assert_matches!(outcome, Outcome::Success(s), s);
        assert_eq!(
            "https://www.example.com",
            parsed_header.ascii_serialization()
        );
    }

    #[test]
    fn request_method_conversion() {
        let method = "POST";
        let parsed_method = not_err!(AccessControlRequestMethod::from_str(method));
        assert_matches!(
            parsed_method,
            AccessControlRequestMethod(crate::Method(rocket::http::Method::Post))
        );

        let method = "options";
        let parsed_method = not_err!(AccessControlRequestMethod::from_str(method));
        assert_matches!(
            parsed_method,
            AccessControlRequestMethod(crate::Method(rocket::http::Method::Options))
        );

        let method = "INVALID";
        is_err!(AccessControlRequestMethod::from_str(method));
    }

    #[test]
    fn request_method_parsing() {
        let client = make_client();
        let mut request = client.get("/");
        let method = hyper::header::AccessControlRequestMethod(hyper::method::Method::Get);
        request.add_header(method);
        let outcome: request::Outcome<AccessControlRequestMethod, crate::Error> =
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
            headers.iter().map(|s| (*s).to_string().into()).collect();
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
        let outcome: request::Outcome<AccessControlRequestHeaders, crate::Error> =
            FromRequest::from_request(request.inner());

        let parsed_header = assert_matches!(outcome, Outcome::Success(s), s);
        let AccessControlRequestHeaders(parsed_headers) = parsed_header;
        let mut parsed_headers: Vec<String> =
            parsed_headers.iter().map(ToString::to_string).collect();
        parsed_headers.sort();
        assert_eq!(
            vec!["accept-language".to_string(), "date".to_string()],
            parsed_headers
        );
    }
}
