use std::result;

use serde::de::{Deserialize, Deserializer};

pub mod album;
pub mod artist;
pub mod playlist;

pub use self::album::{Album, AlbumInfo, ListType};
pub use self::artist::{Artist, ArtistInfo};
pub use self::playlist::Playlist;

/// A representation of a music folder on a Subsonic server.
#[derive(Debug)]
pub struct MusicFolder {
    /// The index number of the folder.
    pub id: usize,
    /// The name assigned to the folder.
    pub name: String,
    _private: bool,
}

impl<'de> Deserialize<'de> for MusicFolder {
    fn deserialize<D>(de: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct _MusicFolder {
            id: String,
            name: String,
        }

        let raw = _MusicFolder::deserialize(de)?;
        Ok(MusicFolder {
            id: raw.id.parse().unwrap(),
            name: raw.name,
            _private: false,
        })
    }
}

/// A genre contained on a Subsonic server.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Genre {
    /// The name of the genre.
    pub name: String,
    /// The number of songs in the genre.
    pub song_count: u64,
    /// The number of albums in the genre.
    pub album_count: u64,
    #[serde(default)]
    _private: bool,
}
