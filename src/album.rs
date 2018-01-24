use serde_json;
use serde::de::{Deserialize, Deserializer};

use error::*;
use query::Query;
use song;
use sunk::Sunk;
use util::*;

#[derive(Debug, Clone, Copy)]
pub enum ListType {
    AlphaByArtist,
    AlphaByName,
    Frequent,
    Highest,
    Newest,
    Random,
    Recent,
    Starred,
}

impl ::std::fmt::Display for ListType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        use self::ListType::*;
        let fmt = match *self {
            AlphaByArtist => "alphabeticalByArtist",
            AlphaByName => "alphabeticalByName",
            Frequent => "frequent",
            Highest => "highest",
            Newest => "newest",
            Random => "random",
            Recent => "recent",
            Starred => "starred",
        };
        write!(f, "{}", fmt)
    }
}

#[derive(Debug, Clone)]
pub struct Album {
    pub id: u64,
    pub name: String,
    pub artist: Option<String>,
    artist_id: Option<u64>,
    cover_id: Option<String>,
    pub duration: u64,
    pub year: Option<u64>,
    pub genre: Option<String>,
    song_count: u64,
    songs: Vec<song::Song>,
}

/// Internal struct matching exactly what `serde` expects.
#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct AlbumSerde {
    id: String,
    name: String,
    artist: Option<String>,
    artistId: Option<String>,
    coverArt: Option<String>,
    songCount: u64,
    duration: u64,
    created: String,
    year: Option<u64>,
    genre: Option<String>,
    songs: Option<Vec<song::Song>>
}

impl Album {
    /// Deserialzises a JSON value into an album.
    ///
    /// # Notes
    ///
    /// This is a temporary function until TryFrom is stabilised.
    pub fn try_from(json: serde_json::Value) -> Result<Album> {
        let mut songs = Vec::new();
        if let Some(Some(list)) = json.get("song").map(|v| v.as_array()) {
            for song in list {
                info!("Found song {} for album {}", song["name"], json["name"]);
                songs.push(song::Song::try_from(song.clone())?);
            }
        }

        let serde: AlbumSerde = serde_json::from_value(json)?;
        Ok(Album {
            id: serde.id.parse()?,
            name: serde.name,
            artist: serde.artist,
            artist_id: serde.artistId.map(|i| i.parse().unwrap()),
            cover_id: serde.coverArt,
            duration: serde.duration,
            year: serde.year,
            genre: serde.genre,
            song_count: serde.songCount,
            songs,
        })
    }

    pub fn songs(&self, sunk: &mut Sunk) -> Result<Vec<song::Song>> {
        if self.songs.len() as u64 != self.song_count {
            Ok(get_album(sunk, self.id)?.songs)
        } else {
            Ok(self.songs.clone())
        }
    }
}

impl<'de> Deserialize<'de> for Album {
    fn deserialize<D>(de: D) -> ::std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let raw = AlbumSerde::deserialize(de)?;

        Ok(Album {
            id: raw.id.parse().unwrap(),
            name: raw.name,
            artist: raw.artist,
            artist_id: raw.artistId.map(|i| i.parse().unwrap()),
            cover_id: raw.coverArt,
            duration: raw.duration,
            year: raw.year,
            genre: raw.genre,
            song_count: raw.songCount,
            songs: raw.songs.unwrap_or_default(),
        })
    }
}

pub fn get_album(sunk: &mut Sunk, id: u64) -> Result<Album> {
    let res = sunk.get("getAlbum", Query::with("id", id))?;
    Ok(serde_json::from_value::<Album>(res)?)
}

pub fn get_albums(
    sunk: &mut Sunk,
    list_type: ListType,
    size: Option<u64>,
    offset: Option<u64>,
    folder_id: Option<u64>,
) -> Result<Vec<Album>> {
    let args = Query::new()
        .arg("type", list_type.to_string())
        .maybe_arg("size", map_str(size))
        .maybe_arg("offset", map_str(offset))
        .maybe_arg("musicFolderId", map_str(folder_id))
        .build();

    let res = sunk.get("getAlbumList2", args)?;

    let mut albums = vec![];
    if let Some(album_arr) = res["album"].as_array() {
        for album in album_arr.clone() {
            albums.push(serde_json::from_value::<Album>(album)?);
        }
    }
    Ok(albums)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_util::*;

    #[test]
    fn remote_get_albums() {
        let (s, u, p) = load_credentials().unwrap();
        let mut srv = Sunk::new(&s, &u, &p).unwrap();
        let albums =
            get_albums(&mut srv, ListType::AlphaByArtist, None, None, None)
                .unwrap();

        println!("{:?}", albums);
        assert!(!albums.is_empty())
    }

    #[test]
    fn parse_from_get_album() {
        let json = json!(
            {
                "id" : "18",
                "name" : "Hooked on a Feeling",
                "artist" : "Blue Swede",
                "artistId" : "8",
                "coverArt" : "al-18",
                "songCount" : 1,
                "duration" : 172,
                "created" : "2018-01-01T10:30:15.000Z",
                "year" : 1974,
                "genre" : "Classic Rock",
                "song" : [ {
                    "id" : "201",
                    "parent" : "200",
                    "isDir" : false,
                    "title" : "Hooked on a Feeling",
                    "album" : "Hooked on a Feeling",
                    "artist" : "Blue Swede",
                    "track" : 1,
                    "year" : 1974,
                    "genre" : "Classic Rock",
                    "coverArt" : "200",
                    "size" : 7191717,
                    "contentType" : "audio/mpeg",
                    "suffix" : "mp3",
                    "duration" : 172,
                    "bitRate" : 320,
                    "path" : "B/Blue Swede/Hooked on a Feeling/01 Hooked on a Feeling.mp3",
                    "isVideo" : false,
                    "playCount" : 0,
                    "discNumber" : 1,
                    "created" : "2018-01-01T10:30:15.000Z",
                    "albumId" : "18",
                    "artistId" : "8",
                    "type" : "music"
                } ]
            }
        );
        let alb = serde_json::from_value::<Album>(json).unwrap();

        assert_eq!(alb.id, 18);
        assert_eq!(alb.cover_id, Some("al-18".to_string()));
        assert_eq!(alb.songs.len(), 1);
        assert_eq!(alb.songs[0].title, "Hooked on a Feeling".to_string())
    }

    #[test]
    fn parse_from_album_list() {
        let json = json!(
            {
                "id" : "314",
                "name" : "#3",
                "artist" : "The Script",
                "artistId" : "177",
                "coverArt" : "al-314",
                "songCount" : 7,
                "duration" : 1736,
                "created" : "2018-01-01T10:31:35.000Z",
                "year" : 2012,
                "genre" : "Pop"
            }
        );
        let alb = serde_json::from_value::<Album>(json).unwrap();

        assert_eq!(alb.id, 314);
        assert_eq!(alb.name, "#3".to_string());
        assert!(alb.songs.is_empty());
    }
}
