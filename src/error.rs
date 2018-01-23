use hyper;
use json;
use std::{fmt, io, num, result};
use std::convert::From;

pub type Result<T> = result::Result<T, self::Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Invalid URL: {}", _0)] Uri(UriError),
    #[fail(display = "Unable to connect to server: received {}", _0)]
    ConnectionError(hyper::StatusCode),
    #[fail(display = "{}", _0)] Other(&'static str),

    #[fail(display = "{}", _0)] Api(#[cause] ApiError),

    #[fail(display = "Failed to parse int: {}", _0)]
    ParError(#[cause] num::ParseIntError),
    #[fail(display = "IO error: {}", _0)] Io(#[cause] io::Error),
    #[fail(display = "Connection error: {}", _0)]
    HyperError(#[cause] hyper::Error),
    #[fail(display = "Error serialising: {}", _0)]
    SerdeError(#[cause] json::Error),
}

#[derive(Debug, Fail)]
pub enum UriError {
    #[fail(display = "{}", _0)] Hyper(#[cause] hyper::error::UriError),
    #[fail(display = "Unable to determine scheme")] Scheme,
    #[fail(display = "Missing server address")] Address,
}

#[derive(Debug, Fail, Clone)]
pub enum ApiError {
    Generic(String),
    MissingParameter,
    ClientMustUpgrade,
    ServerMustUpgrade,
    WrongAuth,
    Ldap,
    NotAuthorized(String),
    TrialExpired,
    NotFound,
}

impl ApiError {
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

    pub fn try_from(json: &json::Value) -> Result<ApiError> {
        use self::ApiError::*;
        let code = json["code"].as_u64().unwrap();
        let message = json["message"].as_str().unwrap().to_string();
        match code {
            10 => Ok(Generic(message)),
            20 => Ok(ClientMustUpgrade),
            30 => Ok(ServerMustUpgrade),
            40 => Ok(WrongAuth),
            41 => Ok(Ldap),
            50 => Ok(NotAuthorized(message)),
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
            ClientMustUpgrade => {
                write!(f, "Incompatible protocol; client must upgrade")
            }
            ServerMustUpgrade => {
                write!(f, "Incompatible protocol; server must upgrade")
            }
            WrongAuth => write!(f, "Wrong username or password"),
            Ldap => {
                write!(f, "Token authentication not supported for LDAP users")
            }
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
    }
}

box_err!(hyper::Error, HyperError);
box_err!(io::Error, Io);
box_err!(num::ParseIntError, ParError);
box_err!(json::Error, SerdeError);

impl From<hyper::error::UriError> for UriError {
    fn from(err: hyper::error::UriError) -> UriError { UriError::Hyper(err) }
}
