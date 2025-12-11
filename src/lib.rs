//! # sunk
//!
//! `sunk` provides natural Rust bindings to the [Subsonic] music server API.
//! Many other popular music servers, such as Libresonic and Airsonic, also use
//! the Subsonic API, so this crate can stretch far beyond just Subsonic.
//!
//! [Subsonic]: http://www.subsonic.org/pages/index.jsp
//!
//! # Basic usage
//!
//! ```no_run
//! extern crate sunk;
//! use sunk::song::Song;
//! use sunk::{Album, Artist, Client, Streamable};
//!
//! # fn run() -> sunk::Result<()> {
//! let site = "http://subsonic.example.com";
//! let username = "admin";
//! let password = "hunter2";
//!
//! let client = Client::new(site, username, password)?;
//!
//! let random_songs = Song::random(&client, 20)?;
//! for mut song in random_songs {
//!     song.set_max_bit_rate(320);
//!     let mut reader = song.stream(&client)?;
//!     // Use the reader to stream the audio data
//! }
//! # Ok(())
//! # }
//! # fn main() { }
//! ```
//!
//! # Philosophy
//!
//! The fundamental principle behind the way `sunk` works is to be able to pivot
//! on everything. If you have something returned from a query, you should be
//! able to investigate that object more deeply. This is modelled after what
//! you'd typically expect from mobile or web clients.
//!
//! An example can be seen below:
//!
//! ```no_run
//! # extern crate sunk;
//! # use sunk::{Client, Album, Artist, Streamable};
//! # use sunk::song::Song;
//! # fn run() -> sunk::Result<()> {
//! # let site = "http://subsonic.example.com";
//! # let username = "admin";
//! # let password = "hunter2";
//! let client = Client::new(site, username, password)?;
//!
//! // I want to play some <insert artist here>.
//! let an_artist = Artist::get(&client, 20)?;
//! let artist_info = an_artist.info(&client)?;
//! let artists_albums = an_artist.albums(&client)?;
//!
//! // I love this album. Let's download it.
//! let ref fav_album = artists_albums[0];
//! let album_info_and_similar = fav_album.info(&client)?;
//! let album_songs = fav_album.songs(&client)?;
//!
//! use std::fs::File;
//! use std::io::Write;
//! for song in &album_songs {
//!     let bytes = song.download(&client)?;
//!     let mut file =
//!         File::create(song.title.clone() + "." + song.encoding())?;
//!     file.write(&bytes)?;
//! }
//!
//! // I want to find stuff like this song.
//! let ref this_is_good = album_songs[6];
//! let similar = this_is_good.similar(&client, 10)?;
//! # Ok(())
//! # }
//! # fn main() { }
//! ```
//!
//! This has the result of many methods requiring an active connection to a
//! `Client` to fetch more information.
//!
//! # Debugging
//!
//! The crate uses [`log`] as its debugging backend. If your crate uses log,
//! you'll see output from `sunk` at any information level starting at warning
//! (for critical processes) or info (for most other processes).
//!
//! [`log`]: https://doc.rust-lang.org/log/log/index.html
//!
//! # Development
//!
//! The crate is still under active development. Methods and paths may change,
//! and many have not been tested due to the requirement of having full access
//! to a Subsonic instance. See the repository for any changes and development
//! status.
//!
//! Bug reports and broken features are encouraged to be reported! **If
//! something does not work as reported, it's probably broken.**

#![deny(missing_docs)]

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

pub mod annotate;
pub mod collections;
pub mod jukebox;
pub mod media;
pub mod query;
pub mod response;
pub mod search;
pub mod user;
pub mod version;

#[cfg(test)]
mod test_util;

pub use self::client::Client;
pub use self::collections::Playlist;
pub use self::collections::{Album, AlbumInfo, ListType};
pub use self::collections::{Artist, ArtistInfo};
pub use self::collections::{Genre, MusicFolder};
pub use self::error::{ApiError, Error, Result, UrlError};
pub use self::jukebox::{Jukebox, JukeboxPlaylist, JukeboxStatus};
pub use self::media::{podcast, song, video};
pub use self::media::{Hls, HlsPlaylist, Media, NowPlaying, RadioStation, Streamable};
use self::song::{Lyrics, Song};
pub use self::user::{User, UserBuilder};
pub use self::version::Version;
