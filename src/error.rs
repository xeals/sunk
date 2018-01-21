use hyper;
use json;
use std::{fmt, io, result};
use std::convert::From;

use api::Api;
use util::*;

// pub type ApiResult<T> = result::Result<T, self::SubsonicError>;
pub type Result<T> = result::Result<T, self::Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Unknown error: {}", _0)] UnknownError(&'static str),
    #[fail(display = "Invalid URL: {}", _0)]
    InvalidUrl(#[cause] hyper::error::UriError),
    #[fail(display = "IO error: {}", _0)] Io(#[cause] io::Error),
    #[fail(display = "API error: {}", _0)] ServerError(String), /* InvalidUrl(&'static str) */
    #[fail(display = "Connection error: {}", _0)]
    HyperError(#[cause] hyper::Error),
    #[fail(display = "Bad field: {}", _0)] ParseError(&'static str),
    #[fail(display = "{}", _0)] Api(#[cause] SubsonicError),
}

#[derive(Debug, Fail, Clone)]
pub enum SubsonicError {
    Generic(String),
    MissingParameter,
    ClientMustUpgrade(Api, Api),
    ServerMustUpgrade(Api, Api),
    WrongAuth,
    Ldap,
    NotAuthorized(String),
    TrialExpired,
    NotFound,
}

impl SubsonicError {
    pub fn as_u16(&self) -> u16 {
        use self::SubsonicError::*;
        match *self {
            Generic(_) => 0,
            MissingParameter => 10,
            ClientMustUpgrade(..) => 20,
            ServerMustUpgrade(..) => 30,
            WrongAuth => 40,
            Ldap => 41,
            NotAuthorized(_) => 50,
            TrialExpired => 60,
            NotFound => 70,
        }
    }

    pub fn from_response(
        res: &json::Value,
        client_ver: Api,
    ) -> self::Result<SubsonicError> {
        macro_rules! get {
            ($j:ident $f:expr) => (
                $j.get($f)
                    .ok_or(Error::ParseError(stringify!(no field "$f")))?
            )
        };
        macro_rules! parse {
            ($j:ident $f:ident) => ($j.$f().ok_or(Error::ParseError(stringify!(failed parsing $j.$f)))?)
        };

        let r = get!(res "subsonic-response");

        let server_ver = get!(r "version");
        let error = get!(r "error");

        let code = get!(error "code");
        let message = get!(error "message");

        use self::SubsonicError::*;

        Ok(match code.as_u64().ok_or(Error::ParseError("not a u64"))? {
            0 => Generic(parse!(message as_str).to_string()),
            10 => MissingParameter,
            20 => {
                ClientMustUpgrade(client_ver, parse!(server_ver as_str).into())
            }
            30 => {
                ServerMustUpgrade(client_ver, parse!(server_ver as_str).into())
            }
            40 => WrongAuth,
            41 => Ldap,
            50 => NotAuthorized(
                parse!(message as_str)
                    .split(' ')
                    .next()
                    .unwrap()
                    .to_string(),
            ),
            60 => TrialExpired,
            70 => NotFound,
            _ => return Err(Error::ParseError("unable to match error code")),
        })
    }
}

impl fmt::Display for SubsonicError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::SubsonicError::*;
        match *self {
            Generic(ref s) => write!(f, "Generic error: {}", s),
            MissingParameter => write!(f, "Missing a required parameter"),
            ClientMustUpgrade(cli, srv) => write!(
                f,
                "Incompatible protocol: client has {}, server has {}; client \
                 must upgrade",
                cli, srv
            ),
            ServerMustUpgrade(cli, srv) => write!(
                f,
                "Incompatible protocol: client has {}, server has {}; client \
                 must upgrade",
                cli, srv
            ),
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
box_err!(hyper::error::UriError, InvalidUrl);
box_err!(io::Error, Io);
