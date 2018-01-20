#[macro_use]
extern crate failure;
extern crate futures;
extern crate hyper;
extern crate serde;
extern crate serde_json as json;
extern crate tokio_core as tokio;
extern crate hyper_tls;
#[macro_use]
extern crate log;

extern crate md5;
extern crate rand;

pub mod error;
pub mod sunk;

#[cfg(test)]
mod tests {}
