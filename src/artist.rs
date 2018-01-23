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
    album_count: u64,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct ArtistSerde {
    id: String,
    name: String,
    coverArt: Option<String>,
    albumCount: u64,
}

impl Artist {
    /// Deserializes a JSON value into an artist.
    ///
    /// # Notes
    ///
    /// This is a temporary function until TryFrom is stabilised.
    pub fn try_from(json: json::Value) -> Result<Artist> {
        let mut albums = Vec::new();
        if let Some(Some(list)) = json.get("album").map(|a| a.as_array()) {
            for album in list {
                if let Some(Some(id)) = album.get("id").map(|i| i.as_str()) {
                    info!("Found album {} for artist {}", album, json["name"]);
                    albums.push(id.parse::<u64>()?);
                }
            }
        }

        let serde: ArtistSerde = json::from_value(json)?;
        Ok(Artist {
            id: serde.id.parse()?,
            name: serde.name,
            cover_id: serde.coverArt,
            album_count: serde.albumCount,
            albums,
        })
    }
    pub fn from(j: &json::Value) -> Result<Artist> {
        if !j.is_object() {
            return Err(Error::ParseError("not an object"))
        }

        let mut albums = vec![];
        if j.get("album").is_some() {
            for album in fetch!(j->album: as_array).iter() {
                albums.push(fetch!(album->id: as_str, u64))
            }
        }

        Ok(Artist {
            id:       fetch!(j->id: as_str, u64),
            name:     fetch!(j->name: as_str).into(),
            cover_id: Some(fetch!(j->coverArt: as_str).into()),
            albums:   albums,
            album_count: fetch!(j->albumCount: as_u64),
        })
    }

    pub fn albums(&self, sunk: &mut Sunk) -> Result<Vec<Album>> {
        let mut album_list = vec![];

        // Building an artist from `get_artists()` doesn't populate the album
        // list, but `get_artist()` does. An artist can, however, exist without
        // an album.
        if self.albums.is_empty() && self.album_count > 0 {
            let albums = get_artist(sunk, self.id)?.albums;

            for id in &albums {
                let res = sunk.get("getAlbum", Query::with("id", id))?;
                album_list.push(
                    Album::from_json(res)?
                )
            }
        } else {
            for id in &self.albums {
                let res = sunk.get("getAlbum", Query::with("id", id))?;
                album_list.push(
                    Album::from_json(res)?
                )
            }
        };

        Ok(album_list)
    }

    impl_cover_art!();
}

pub fn get_artist(sunk: &mut Sunk, id: u64) -> Result<Artist> {
    let res = sunk.get("getArtist", Query::with("id", id))?;
    Artist::from(&res["subsonic-response"]["playlist"])
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
        let parsed = Artist::try_from(raw()).unwrap();

        assert_eq!(parsed.name, "Backstreet Boys".to_string());
        assert_eq!(parsed.albums.len(), 1);
        assert_eq!(parsed.albums, vec![1]);
    }

    #[test]
    fn remote_artist_album_list() {
        let (s, u, p) = load_credentials().unwrap();
        let mut srv = Sunk::new(&s, &u, &p).unwrap();
        let parsed = Artist::from(&raw()).unwrap();
        let albums = parsed.albums(&mut srv).unwrap();

        println!("Parsed: {:?}", albums);
        assert_eq!(albums[0].name, "The Hits: Chapter One".to_string());
        assert_eq!(albums[0].year, Some(2001));
    }

    #[test]
    fn remote_artist_cover_art() {
        let (s, u, p) = load_credentials().unwrap();
        let mut srv = Sunk::new(&s, &u, &p).unwrap();
        let parsed = Artist::from(&raw()).unwrap();
        let cover = parsed.cover_art(&mut srv, None).unwrap();

        println!("{:?}", cover);
        assert!(!cover.is_empty())
    }
}
