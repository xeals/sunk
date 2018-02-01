use serde::de::{Deserialize, Deserializer};
use std::result;

mod album;
mod artist;
mod playlist;

pub use self::album::{Album, AlbumInfo};
pub use self::artist::{Artist, ArtistInfo, SimilarArtist};
pub use self::playlist::Playlist;

#[derive(Debug)]
pub struct MusicFolder {
    pub id: usize,
    pub name: String,
}

impl MusicFolder {
    fn from(id: usize, name: String) -> MusicFolder { MusicFolder { id, name } }
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
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Genre {
    pub name: String,
    pub song_count: u64,
    pub album_count: u64,
}
