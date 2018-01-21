use json;

use error::*;
use util::*;

#[derive(Debug)]
pub struct Album {
    pub id: u64,
    pub name: String,
    pub artist: Option<String>,
    artist_id: Option<u64>,
    cover_id: Option<String>,
    pub duration: u64,
    pub year: Option<u64>,
    pub genre: Option<String>,
    songs: Vec<u64>
}

impl Album {
    pub fn from(j: &json::Value) -> Result<Album> {
        if !j.is_object() {
            return Err(Error::ParseError("not an object"))
        }

        let mut songs = vec![];
        for song in fetch!(j->song: as_array).iter() {
            songs.push(fetch!(song->id: as_str, u64))
        }

        Ok(Album {
            id: fetch!(j->id: as_str, u64),
            name: fetch!(j->name: as_str).into(),
            artist: fetch_maybe!(j->artist: as_str).map(|v| v.to_string()),
            artist_id: fetch_maybe!(j->artistId: as_str, u64),
            cover_id: fetch_maybe!(j->coverArt: as_str).map(|v| v.to_string()),
            duration: fetch!(j->duration: as_u64),
            year: fetch_maybe!(j->year: as_u64),
            genre: fetch_maybe!(j->genre: as_str).map(|v| v.to_string()),
            songs: songs,
        })
    }
}
