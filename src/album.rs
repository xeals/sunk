use json;

use sunk::Sunk;
use song;
use query::Query;
use error::*;
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
            Starred => "starred"
        };
        write!(f, "{}", fmt)
    }
}

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
    songs: Vec<u64>,
    song_count: u64,
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
}

impl Album {
    /// Deserialzises a JSON value into an album.
    ///
    /// # Notes
    ///
    /// This is a temporary function until TryFrom is stabilised.
    pub fn try_from(json: json::Value) -> Result<Album> {
        // `getAlbum` returns the songs in the album, but `albumList` does not.
        let mut songs = Vec::new();
        if let Some(Some(list)) = json.get("song").map(|v| v.as_array()) {
            for song in list {
                if let Some(Some(id)) = song.get("id").map(|i| i.as_str()) {
                    info!("Found song {} for album {}", song, json["name"]);
                    songs.push(id.parse::<u64>()?);
                }
            }
        }

        let serde: AlbumSerde = json::from_value(json)?;
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
        let mut song_list = Vec::new();

        if self.songs.len() as u64 != self.song_count {
            let songs = get_album(sunk, self.id)?.songs;

            for id in &songs {
                song_list.push(song::get_song(sunk, *id)?);
            }
        } else {
            for id in &self.songs {
                song_list.push(song::get_song(sunk, *id)?);
            }
        }

        Ok(song_list)
    }
}

pub fn get_album(sunk: &mut Sunk, id: u64) -> Result<Album> {
    let res = sunk.get("getAlbum", Query::with("id", id))?;
    Album::try_from(res)
}

pub fn get_albums(
    sunk: &mut Sunk,
    list_type: ListType,
    size: Option<u64>,
    offset: Option<u64>,
    folder_id: Option<u64>
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
            albums.push(Album::try_from(album)?);
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
        let albums = get_albums(&mut srv, ListType::AlphaByArtist, None, None, None).unwrap();

        println!("{:?}", albums);
        assert!(!albums.is_empty())
    }

    #[test]
    fn parse_from_get_album() {
        let json = json!(
            {
                "id" : "200",
                "name" : "Aqours オリジナルソングCD 1",
                "artist" : "高海千歌(CV⋯伊波杏樹)",
                "artistId" : "126",
                "coverArt" : "al-200",
                "songCount" : 2,
                "duration" : 544,
                "created" : "2018-01-01T10:31:42.000Z",
                "year" : 2017,
                "genre" : "J-Pop",
                "song" : [ {
                    "id" : "1450",
                    "title" : "One More Sunshine Story",
                    "album" : "Aqours オリジナルソングCD 1",
                    "artist" : "高海千歌(CV⋯伊波杏樹)"
                }]
            }
        );
        let alb = Album::try_from(json).unwrap();

        assert_eq!(alb.id, 200);
        assert_eq!(alb.cover_id, Some("al-200".to_string()));
        assert_eq!(alb.songs, vec![1450]);
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
        let alb = Album::try_from(json).unwrap();

        assert_eq!(alb.id, 314);
        assert_eq!(alb.name, "#3".to_string());
        assert!(alb.songs.is_empty());
    }
}
