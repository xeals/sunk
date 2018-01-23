use json;

use query::Query;
use error::*;
use util::*;
use sunk::Sunk;

use album;
use song;

#[derive(Debug)]
pub struct Artist {
    id: u64,
    pub name: String,
    cover_id: Option<String>,
    albums: Vec<album::Album>,
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
                info!("Found album {} for artist {}", album["name"], json["name"]);
                albums.push(album::Album::try_from(album.clone())?);
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

    pub fn albums(&self, sunk: &mut Sunk) -> Result<Vec<album::Album>> {
        if self.albums.len() as u64 != self.album_count {
            Ok(get_artist(sunk, self.id)?.albums)
        } else {
            Ok(self.albums.clone())
        }
    }

    impl_cover_art!();
}

pub fn get_artist(sunk: &mut Sunk, id: u64) -> Result<Artist> {
    let res = sunk.get("getArtist", Query::with("id", id))?;
    Artist::try_from(res)
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
    }

    #[test]
    fn remote_artist_album_list() {
        let (s, u, p) = load_credentials().unwrap();
        let mut srv = Sunk::new(&s, &u, &p).unwrap();
        let parsed = Artist::try_from(raw()).unwrap();
        let albums = parsed.albums(&mut srv).unwrap();

        println!("Parsed: {:?}", albums);
        assert_eq!(albums[0].name, "The Hits: Chapter One".to_string());
        assert_eq!(albums[0].year, Some(2001));
    }

    #[test]
    fn remote_artist_cover_art() {
        let (s, u, p) = load_credentials().unwrap();
        let mut srv = Sunk::new(&s, &u, &p).unwrap();
        let parsed = Artist::try_from(raw()).unwrap();
        let cover = parsed.cover_art(&mut srv, None).unwrap();

        println!("{:?}", cover);
        assert!(!cover.is_empty())
    }
}
