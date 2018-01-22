use json;

use sunk::Sunk;
use query::Query;
use error::*;
use util::*;

#[derive(Debug, Clone)]
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
    cover_id: String,
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
    coverArt: String,
    songCount: u64,
    duration: u64,
    created: String,
    year: Option<u64>,
    genre: Option<String>,
}

impl Album {
    /// Deserialzises a `json::Value` into an album.
    pub fn from_json(json: json::Value) -> Result<Album> {
        // `getAlbum` returns the songs in the album, but `albumList` does not.
        let mut songs = Vec::new();
        if let Some(Some(list)) = json.get("song").map(|v| v.as_array()) {
            for song in list {
                println!("found: {}", song);
                songs.push(song
                    .try_get("id")?
                    .as_str().unwrap()
                    .parse::<u64>()?);
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

    pub fn from(j: &json::Value) -> Result<Album> {
        if !j.is_object() {
            return Err(Error::ParseError("not an object"))
        }

        let mut songs = vec![];
        if let Some(_) = j.get("song") {
            for song in fetch!(j->song: as_array).iter() {
                songs.push(fetch!(song->id: as_str, u64))
            }
        }

        Ok(Album {
            id: fetch!(j->id: as_str, u64),
            name: fetch!(j->name: as_str).into(),
            artist: fetch_maybe!(j->artist: as_str).map(|v| v.to_string()),
            artist_id: fetch_maybe!(j->artistId: as_str, u64),
            cover_id: fetch!(j->coverArt: as_str).into(),
            duration: fetch!(j->duration: as_u64),
            year: fetch_maybe!(j->year: as_u64),
            genre: fetch_maybe!(j->genre: as_str).map(|v| v.to_string()),
            songs: songs,
            song_count: fetch!(j->songCount: as_u64),
        })
    }
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
    // for album in pointer!(res, "/subsonic-response/albumList2/album")
    //     .as_array()
    //     .ok_or(Error::ParseError("albumList2 not an array"))?
    for album in res.try_get("subsonic-response")?
    .try_get("albumList2")?
    .try_get("album")?
    .try_array()?
        {
            albums.push(Album::from_json(album)?);
        }
    Ok(albums)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_util::*;

    #[test]
    fn test_get_albums() {
        let (s, u, p) = load_credentials().unwrap();
        let mut srv = Sunk::new(&s, &u, &p).unwrap();
        let albums = get_albums(&mut srv, ListType::AlphaByArtist, None, None, None).unwrap();

        println!("{:?}", albums);
        // assert!(true)
        panic!()
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
        let alb = Album::from_json(json).unwrap();
        println!("{:?}", alb);
        assert_eq!(alb.id, 200);
        assert_eq!(alb.cover_id, "al-200".to_string());
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
        let alb = Album::from_json(json).unwrap();
        println!("{:?}", alb);
        assert_eq!(alb.id, 314);
        assert_eq!(alb.name, "#3".to_string());
        assert!(alb.songs.is_empty());
    }
}
