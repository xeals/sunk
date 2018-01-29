#![recursion_limit = "128"]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate md5;
extern crate rand;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
mod macros;

pub use client::{Client, License};

pub mod error;
pub use error::{Error, Result};

pub mod album;
pub mod artist;
pub mod playlist;
pub mod media;

pub mod api;
pub mod library;
pub mod query;
mod client;
pub mod response;
pub mod user;

#[cfg(test)]
mod test_util;
