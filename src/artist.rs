use serde_json;
use serde::de::{Deserialize, Deserializer};

use error::*;
use query::Query;
use sunk::Sunk;
use util::*;

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

#[derive(Debug)]
pub struct ArtistInfo {
    biography: String,
    musicbrainz_id: String,
    lastfm_url: String,
    image_urls: (String, String, String),
    similar_artists: Vec<(usize, String)>
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct ArtistInfoSerde {
    biography: String,
    musicBrainzId: String,
    lastFmUrl: String,
    smallImageUrl: String,
    mediumImageUrl: String,
    largeImageUrl: String,
    similarArtist: Vec<SimilarArtistSerde>
}

#[derive(Debug, Deserialize)]
struct SimilarArtistSerde {
    id: String,
    name: String,
}

impl Artist {
    /// Deserializes a JSON value into an artist.
    ///
    /// # Notes
    ///
    /// This is a temporary function until TryFrom is stabilised.
    pub fn try_from(json: serde_json::Value) -> Result<Artist> {
        let mut albums = Vec::new();
        if let Some(Some(list)) = json.get("album").map(|a| a.as_array()) {
            for album in list {
                info!(
                    "Found album {} for artist {}",
                    album["name"], json["name"]
                );
                albums.push(album::Album::try_from(album.clone())?);
            }
        }

        let serde: ArtistSerde = serde_json::from_value(json)?;
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

    pub fn info(
        &self,
        sunk: &mut Sunk,
        count: Option<usize>,
        include_not_present: Option<bool>)
        -> Result<ArtistInfo>
    {
        let args = Query::with("id", self.id.to_string())
            .maybe_arg("count", map_str(count))
            .maybe_arg("includeNotPresent", map_str(include_not_present))
            .build();
        let res = sunk.get("getArtistInfo", args)?;

        let serde: ArtistInfoSerde = serde_json::from_value(res)?;
        Ok(ArtistInfo {
            biography: serde.biography,
            musicbrainz_id: serde.musicBrainzId,
            lastfm_url: serde.lastFmUrl,
            image_urls: (serde.smallImageUrl, serde.mediumImageUrl, serde.largeImageUrl),
            similar_artists: serde.similarArtist.iter().map(|a| (a.id.parse().unwrap(), a.name.to_string())).collect(),
        })
    }

    impl_cover_art!();
}

// impl<'de> Deserialize<'de> for Artist {
//     fn deserialize<D>(de: D) -> std::result::Result<Self, D::Error>
//     where
//         D: Deserializer<'de>
//     {
//         let raw = ArtistSerde::deserialize(de)?;
//         let album = raw["album"];
//         let albums = get_list_as!(album, album::Album);

//         Ok(Artist {
//             id: raw.id.parse()?,
//             name: raw.name,
//             cover_id: raw.coverArt,
//             album_count: raw.albumCount,
//             albums,
//         })
//     }
// }

pub fn get_artist(sunk: &mut Sunk, id: u64) -> Result<Artist> {
    let res = sunk.get("getArtist", Query::with("id", id))?;
    Artist::try_from(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_util::*;

    fn raw() -> serde_json::Value {
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
