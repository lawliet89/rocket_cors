//! CORS specific Request Headers

use std::collections::HashSet;
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use rocket::{self, Outcome};
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use unicase::UniCase;
use url;

#[cfg(feature = "serialization")]
use unicase_serde;
#[cfg(feature = "serialization")]
use url_serde;

/// A case insensitive header name
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct HeaderFieldName(
    #[cfg_attr(feature = "serialization", serde(with = "unicase_serde::unicase"))]
    UniCase<String>
);

impl Deref for HeaderFieldName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl fmt::Display for HeaderFieldName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

/// A wrapped `url::Url` to allow for deserialization
#[derive(Eq, PartialEq, Clone, Hash, Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Url(
    #[cfg_attr(feature = "serialization", serde(with = "url_serde"))]
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
    type Error = ::Error;

    fn from_request(request: &'a rocket::Request<'r>) -> request::Outcome<Self, ::Error> {
        match request.headers().get_one("Origin") {
            Some(origin) => {
                match Self::from_str(origin) {
                    Ok(origin) => Outcome::Success(origin),
                    Err(e) => Outcome::Failure((Status::BadRequest, ::Error::BadOrigin(e))),
                }
            }
            None => Outcome::Forward(()),
        }
    }
}

/// The `Origin` request header used in CORS
///
/// You can use this as a rocket [Request Guard](https://rocket.rs/guide/requests/#request-guards)
/// to ensure that `Origin` is passed in correctly.
pub type Origin = Url;

/// The `Access-Control-Request-Method` request header
///
/// You can use this as a rocket [Request Guard](https://rocket.rs/guide/requests/#request-guards)
/// to ensure that the header is passed in correctly.
#[derive(Debug)]
pub struct AccessControlRequestMethod(pub ::Method);

impl FromStr for AccessControlRequestMethod {
    type Err = rocket::Error;

    fn from_str(method: &str) -> Result<Self, Self::Err> {
        Ok(AccessControlRequestMethod(::Method::from_str(method)?))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for AccessControlRequestMethod {
    type Error = ::Error;

    fn from_request(request: &'a rocket::Request<'r>) -> request::Outcome<Self, ::Error> {
        match request.headers().get_one("Access-Control-Request-Method") {
            Some(request_method) => {
                match Self::from_str(request_method) {
                    Ok(request_method) => Outcome::Success(request_method),
                    Err(e) => Outcome::Failure((Status::BadRequest, ::Error::BadRequestMethod(e))),
                }
            }
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
    type Error = ::Error;

    fn from_request(request: &'a rocket::Request<'r>) -> request::Outcome<Self, ::Error> {
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

        let outcome: request::Outcome<Origin, ::Error> = FromRequest::from_request(request.inner());
        let parsed_header = assert_matches!(outcome, Outcome::Success(s), s);
        assert_eq!("https://www.example.com/", parsed_header.as_str());
    }

    #[test]
    fn request_method_conversion() {
        let method = "POST";
        let parsed_method = not_err!(AccessControlRequestMethod::from_str(method));
        assert_matches!(
            parsed_method,
            AccessControlRequestMethod(::Method(rocket::http::Method::Post))
        );

        let method = "options";
        let parsed_method = not_err!(AccessControlRequestMethod::from_str(method));
        assert_matches!(
            parsed_method,
            AccessControlRequestMethod(::Method(rocket::http::Method::Options))
        );

        let method = "INVALID";
        let _ = is_err!(AccessControlRequestMethod::from_str(method));
    }

    #[test]
    fn request_method_parsing() {
        let client = make_client();
        let mut request = client.get("/");
        let method = hyper::header::AccessControlRequestMethod(hyper::method::Method::Get);
        request.add_header(method);
        let outcome: request::Outcome<AccessControlRequestMethod, ::Error> =
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
        let outcome: request::Outcome<AccessControlRequestHeaders, ::Error> =
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
}
