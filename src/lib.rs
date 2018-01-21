#![recursion_limit = "128"]

#[macro_use]
extern crate failure;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_json as json;
extern crate tokio_core as tokio;

extern crate md5;
extern crate rand;

mod macros;
mod test_util;
pub mod error;
pub mod sunk;
pub mod song;
pub mod playlist;
pub mod api;
pub mod query;

#[cfg(test)]
mod tests {}
