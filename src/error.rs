use std::convert::From;
use std::{fmt, io, num, result};

use reqwest;
use serde::de::{Deserialize, Deserializer};
use serde_json;

/// An alias for `sunk`'s error result type.
pub type Result<T> = result::Result<T, self::Error>;

/// Possible errors that may be returned by a function.
#[derive(Debug, Fail)]
pub enum Error {
    /// Unable to connect to the Subsonic server.
    #[fail(display = "Unable to connect to server: received {}", _0)]
    Connection(reqwest::StatusCode),

    /// Unable to recognize the URL provided in `Client` setup.
    #[fail(display = "Invalid URL: {}", _0)]
    Url(UrlError),
    /// The Subsonic server returned an error.
    #[fail(display = "{}", _0)]
    Api(#[cause] ApiError),

    /// A number conversion failed.
    #[fail(display = "Failed to parse int: {}", _0)]
    Parse(#[cause] num::ParseIntError),
    /// An IO issue occurred.
    #[fail(display = "IO error: {}", _0)]
    Io(#[cause] io::Error),
    /// An error in the web framework occurred.
    #[fail(display = "Connection error: {}", _0)]
    Reqwest(#[cause] reqwest::Error),
    /// An error occurred in serialization.
    #[fail(display = "Error serialising: {}", _0)]
    Serde(#[cause] serde_json::Error),

    /// For general, one-off errors.
    #[fail(display = "{}", _0)]
    Other(&'static str),
}

/// Possible errors when initializing a `Client`.
#[derive(Debug, Fail)]
pub enum UrlError {
    /// Unable to parse the URL.
    #[fail(display = "{}", _0)]
    Reqwest(#[cause] reqwest::UrlError),
    /// Unable to determine the scheme of the address.
    ///
    /// The provider for the `Client` does not automatically add the HTTP
    /// scheme like other Rust frameworks. If you encounter this error,
    /// you probably need to add `http://` or `https://` to your server address.
    #[fail(display = "Unable to determine scheme")]
    Scheme,
    /// The server address was not provided.
    #[fail(display = "Missing server address")]
    Address,
}

/// The possible errors a Subsonic server may return.
#[derive(Debug, Fail, Clone)]
pub enum ApiError {
    /// A generic error.
    Generic(String),
    /// A required parameter is missing.
    MissingParameter,
    /// Incompatible REST protocol version. Client must upgrade.
    ClientMustUpgrade,
    /// Incompatible REST protocol version. Server must upgrade.
    ServerMustUpgrade,
    /// Wrong username or password.
    WrongAuth,
    /// Token authentication is not supported for LDAP users.
    Ldap,
    /// The user is not authorized for the given operation.
    NotAuthorized(String),
    /// The trial period for the Subsonic server is over.
    ///
    /// Subsonic has a thirty day trial to use the software, including the REST
    /// API. Forks of Subsonic typically do not offer this support and should
    /// never return this error.
    TrialExpired,
    /// The requested data was not found.
    NotFound,
}

impl ApiError {
    /// Returns the code number of the error.
    pub fn as_u16(&self) -> u16 {
        use self::ApiError::*;
        match *self {
            Generic(_) => 0,
            MissingParameter => 10,
            ClientMustUpgrade => 20,
            ServerMustUpgrade => 30,
            WrongAuth => 40,
            Ldap => 41,
            NotAuthorized(_) => 50,
            TrialExpired => 60,
            NotFound => 70,
        }
    }
}

/// Deserializes a `serde_json::Value` into an `ApiError`.
///
/// Expects a Subsonic `error` response; for example:
///
/// ```ignore
/// "error": {
///   "code": 50,
///   "message": "Permission denied for resource"
///  }
/// ```
impl<'de> Deserialize<'de> for ApiError {
    fn deserialize<D>(de: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct _Error {
            code: usize,
            message: String,
        }

        let raw = _Error::deserialize(de)?;

        use self::ApiError::*;

        match raw.code {
            10 => Ok(Generic(raw.message)),
            20 => Ok(ClientMustUpgrade),
            30 => Ok(ServerMustUpgrade),
            40 => Ok(WrongAuth),
            41 => Ok(Ldap),
            50 => Ok(NotAuthorized(raw.message)),
            60 => Ok(TrialExpired),
            70 => Ok(NotFound),
            _ => unimplemented!(),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ApiError::*;
        match *self {
            Generic(ref s) => write!(f, "Generic error: {}", s),
            MissingParameter => write!(f, "Missing a required parameter"),
            ClientMustUpgrade => write!(f, "Incompatible protocol; client must upgrade"),
            ServerMustUpgrade => write!(f, "Incompatible protocol; server must upgrade"),
            WrongAuth => write!(f, "Wrong username or password"),
            Ldap => write!(f, "Token authentication not supported for LDAP users"),
            NotAuthorized(ref s) => write!(f, "Not authorized: {}", s),
            TrialExpired => write!(f, "Subsonic trial period has expired"),
            NotFound => write!(f, "Requested data not found"),
        }
    }
}
macro_rules! box_err {
    ($err:ty, $to:ident) => {
        impl From<$err> for Error {
            fn from(err: $err) -> Error {
                Error::$to(err)
            }
        }
    };
}

box_err!(reqwest::Error, Reqwest);
box_err!(io::Error, Io);
box_err!(num::ParseIntError, Parse);
box_err!(serde_json::Error, Serde);
box_err!(UrlError, Url);
box_err!(ApiError, Api);

impl From<reqwest::UrlError> for UrlError {
    fn from(err: reqwest::UrlError) -> UrlError {
        UrlError::Reqwest(err)
    }
}

impl From<reqwest::UrlError> for Error {
    fn from(err: reqwest::UrlError) -> Error {
        Error::Url(err.into())
    }
}
