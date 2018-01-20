use hyper;
use std::convert::From;
use std::io;
use json;

pub type Result<T> = ::std::result::Result<T, self::Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Unknown error: {}", _0)] UnknownError(&'static str),
    #[fail(display = "Invalid URL: {}", _0)] InvalidUrl(#[cause] hyper::error::UriError),
    #[fail(display = "IO error: {}", _0)] Io(#[cause] io::Error),
    #[fail(display = "API error: {}", _0)] ServerError(String), // InvalidUrl(&'static str)
    #[fail(display = "Connection error: {}", _0)] HyperError(#[cause] hyper::Error),
    #[fail(display = "Bad field: {}", _0)] ParseError(&'static str),
}

pub fn subsonic_err(err: u64, tar_ver: &str, srv_ver: &json::Value, msg: &json::Value) -> Result<()> {
    macro_rules! err (
        ($e:expr) => (return Err(Error::ServerError($e.into())))
    );

    match err {
        0 => err!(format!("unexpected response: {}", msg)),
        10 => err!("missing a required parameter"),
        20 => err!(format!(
            "incompatible protocol: \
             client must upgrade, server has {}",
            srv_ver)),
        30 => err!(format!(
            "incompatible protocol: \
             expected >= {}, server has {}",
            tar_ver, srv_ver
        )),
        40 => err!("wrong username or password"),
        41 => err!("token auth not supported for LDAP server"),
        50 => err!("user is not authorized for that action"),
        60 => err!("subsonic trial period has expired"),
        70 => err!("requested data not found"),
        _ => err!(format!("unexpected response: {}", msg)),
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
