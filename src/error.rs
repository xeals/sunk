use hyper;
use std::convert::From;
use std::io;

pub type Result<T> = ::std::result::Result<T, self::Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Unknown error: {}", _0)] UnknownError(&'static str),
    #[fail(display = "Invalid URL: {}", _0)] InvalidUrl(#[cause] hyper::error::UriError),
    #[fail(display = "IO error: {}", _0)] Io(#[cause] io::Error),
    #[fail(display = "API error: {}", _0)] ServerError(String), // InvalidUrl(&'static str)
    #[fail(display = "Connection error: {}", _0)] HyperError(#[cause] hyper::Error),
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
