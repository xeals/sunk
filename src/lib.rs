#![warn(missing_docs)]

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
pub mod search;
mod query;
mod version;
mod response;
mod user;

#[cfg(test)]
mod test_util;

pub use self::client::Client;
pub use self::collections::{Album, AlbumInfo, ListType};
pub use self::collections::{Artist, ArtistInfo, SimilarArtist};
pub use self::collections::{Genre, MusicFolder};
pub use self::collections::Playlist;
pub use self::error::{ApiError, Error, Result, UrlError};
pub use self::jukebox::{Jukebox, JukeboxPlaylist, JukeboxStatus};
pub use self::media::{Media, NowPlaying, RadioStation, Streamable};
pub use self::media::{podcast, song, video};
pub use self::user::{User, UserBuilder};
pub use self::version::Version;

use self::song::{Lyrics, RandomSongs, Song};
use self::video::Video;
