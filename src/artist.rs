use json;

use query::Query;
use error::*;
use util::*;
use song::Song;
use sunk::Sunk;
use album::Album;

#[derive(Debug)]
pub struct Artist {
    id: u64,
    pub name: String,
    cover_id: Option<String>,
    albums: Vec<u64>,
}

impl Artist {
    pub fn from(j: &json::Value) -> Result<Artist> {
        if !j.is_object() {
            return Err(Error::ParseError("not an object"))
        }

        let mut albums = vec![];
        for album in fetch!(j->album: as_array).iter() {
            albums.push(fetch!(album->id: as_str, u64))
        }

        Ok(Artist {
            id:       fetch!(j->id: as_str, u64),
            name:     fetch!(j->name: as_str).into(),
            cover_id: Some(fetch!(j->coverArt: as_str).into()),
            albums:   albums,
        })
    }

    pub fn albums(&self, sunk: &mut Sunk) -> Result<Vec<Album>> {
        let mut album_list = vec![];
        for id in &self.albums {
            let (_, res) = sunk.get("getAlbum", Query::from("id", id))?;
            album_list.push(
                Album::from(pointer!(res, "/subsonic-response/album"))?
            )
        }

        Ok(album_list)
    }

    impl_cover_art!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_util::*;

    fn raw() -> json::Value {
        json!({
            "id" : "1",
            "name" : "Backstreet Boys",
            "coverArt" : "ar-1",
            "albumCount" : 1,
            "album" : [ {
                "id" : "1",
                "name" : "The Hits: Chapter One",
                "artist" : "Backstreet Boys",
                "artistId" : "1",
                "coverArt" : "al-1",
                "songCount" : 2,
                "duration" : 499,
                "created" : "2018-01-01T10:30:10.000Z",
                "year" : 2001,
                "genre" : "Pop"
            } ]
        })
    }

    #[test]
    fn parse_artist() {
        let parsed = Artist::from(&raw()).unwrap();

        assert_eq!(parsed.name, "Backstreet Boys".to_string());
        assert_eq!(parsed.albums.len(), 1);
        assert_eq!(parsed.albums, vec![1]);
    }

    #[test]
    fn album_list() {
        let (s, u, p) = load_credentials().unwrap();
        let mut srv = Sunk::new(&s, &u, &p).unwrap();
        let parsed = Artist::from(&raw()).unwrap();
        let albums = parsed.albums(&mut srv).unwrap();

        println!("Parsed: {:?}", albums);
        assert_eq!(albums[0].name, "The Hits: Chapter One".to_string());
        assert_eq!(albums[0].year, Some(2001));
    }

    #[test]
    fn cover_art() {
        let (s, u, p) = load_credentials().unwrap();
        let mut srv = Sunk::new(&s, &u, &p).unwrap();
        let parsed = Artist::from(&raw()).unwrap();
        let cover = parsed.cover_art(&mut srv, None).unwrap();

        println!("{}", cover);
        assert!()
    }
}
