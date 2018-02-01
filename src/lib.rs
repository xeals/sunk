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
mod error;

mod media;
mod collections;

mod jukebox;
mod annotate;
mod library;
mod query;
mod version;
mod response;
mod user;

#[cfg(test)]
mod test_util;

pub use self::client::Client;
pub use self::collections::{Album, AlbumInfo};
pub use self::collections::{Artist, ArtistInfo, SimilarArtist};
pub use self::collections::Playlist;
pub use self::error::{ApiError, Error, Result, UriError};
pub use self::jukebox::{Jukebox, JukeboxPlaylist, JukeboxStatus};
pub use self::library::Genre;
pub use self::media::{Lyrics, Media, NowPlaying, RadioStation, Song,
                      Streamable, Video};
pub use self::media::podcast;
pub use self::user::{User, UserBuilder};
pub use self::version::Version;
