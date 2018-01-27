#![recursion_limit = "128"]

#[macro_use]
extern crate failure;
extern crate reqwest;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

extern crate md5;
extern crate rand;

mod util;

pub use sunk::{Sunk, License};

pub mod album;
pub mod artist;
pub mod error;
pub mod playlist;
pub mod song;

pub mod api;
pub mod library;
pub mod query;
mod sunk;
pub mod response;
pub mod user;

#[cfg(test)]
mod test_util;
