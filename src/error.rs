use hyper;
use std::convert::From;
use std::io;

pub type Result<T> = ::std::result::Result<T, self::Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Unknown error: {}", _0)]
    UnknownError(&'static str),
    #[fail(display = "Invalid URL: {}", _0)]
    InvalidUrl(#[cause] hyper::error::UriError),
    #[fail(display = "IO error: {}", _0)]
    Io(#[cause] io::Error),
    #[fail(display = "API error: {}", _0)]
    ServerError(String)
    // InvalidUrl(&'static str),
}

impl From<hyper::error::UriError> for Error {
    fn from(err: hyper::error::UriError) -> Error {
        Error::InvalidUrl(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}
