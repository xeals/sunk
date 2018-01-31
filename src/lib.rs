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
mod client;

pub use client::Client;
pub use error::{Error, Result};

pub mod error;
pub mod media;

pub mod album;
pub mod artist;
pub mod playlist;

pub mod jukebox;
pub mod annotate;
pub mod library;
pub mod query;
pub mod api;
pub mod response;
pub mod user;

#[cfg(test)]
mod test_util;
