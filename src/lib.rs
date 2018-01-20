#[macro_use]
extern crate failure;
extern crate hyper;
extern crate futures;
extern crate tokio_core as tokio;
extern crate serde_json;
// extern crate url;

extern crate md5;
extern crate rand;

pub mod error;
pub mod sunk;

#[cfg(test)]
mod tests {
}
