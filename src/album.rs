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
    cover_id: Option<String>,
    pub duration: u64,
    pub year: Option<u64>,
    pub genre: Option<String>,
    songs: Vec<u64>,
    song_count: u64,
}

impl Album {
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
            cover_id: fetch_maybe!(j->coverArt: as_str).map(|v| v.to_string()),
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
    let mut args = Query::new();
    args.push("type", list_type.to_string());
    args.push_some("size", map_str(size));
    args.push_some("offset", map_str(offset));
    args.push_some("musicFolderId", map_str(folder_id));

    let (_, res) = sunk.get("getAlbumList2", args)?;

    let mut albums = vec![];
    for album in pointer!(res, "/subsonic-response/albumList2/album")
        .as_array()
        .ok_or(Error::ParseError("albumList2 not an array"))?
    {
        albums.push(Album::from(album)?);
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
}
