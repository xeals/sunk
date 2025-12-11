//! Artist APIs.

use std::{fmt, result};

use serde::de::{Deserialize, Deserializer};
use serde_json;

use crate::query::Query;
use crate::{Album, Client, Error, Media, Result, Song};

/// Basic information about an artist.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct Artist {
    pub id: usize,
    pub name: String,
    cover_id: Option<String>,
    albums: Vec<Album>,
    pub album_count: usize,
}

/// Detailed information about an artist.
#[derive(Debug, Clone)]
pub struct ArtistInfo {
    /// A blurb about the artist.
    pub biography: String,
    /// The artist's [MusicBrainz](https://musicbrainz.org/) ID.
    pub musicbrainz_id: String,
    /// The artist's [last.fm](https://last.fm) landing page.
    pub lastfm_url: String,
    /// URLs for the artist's image; available in small, medium, and large.
    pub image_urls: (String, String, String),
    /// Artists similar to this one. Provided by last.fm.
    similar_artists: Vec<Artist>,
}

impl Artist {
    #[allow(missing_docs)]
    pub fn get(client: &Client, id: usize) -> Result<Artist> {
        self::get_artist(client, id)
    }

    /// Returns a list of albums released by the artist.
    pub fn albums(&self, client: &Client) -> Result<Vec<Album>> {
        if self.albums.len() != self.album_count {
            Ok(self::get_artist(client, self.id)?.albums)
        } else {
            Ok(self.albums.clone())
        }
    }

    /// Queries last.fm for more information about the artist.
    pub fn info(&self, client: &Client) -> Result<ArtistInfo> {
        let res = client.get("getArtistInfo", Query::with("id", self.id))?;
        Ok(serde_json::from_value(res)?)
    }

    /// Returns a number of random artists similar to this one.
    ///
    /// last.fm suggests a number of similar artists to the one the method is
    /// called on. Optionally takes a `count` to specify the maximum number of
    /// results to return, and whether to only include artists in the Subsonic
    /// library (defaults to true).
    pub fn similar<B, U>(
        &self,
        client: &Client,
        count: U,
        include_not_present: B,
    ) -> Result<Vec<Artist>>
    where
        B: Into<Option<bool>>,
        U: Into<Option<usize>>,
    {
        let args = Query::with("id", self.id)
            .arg("count", count.into())
            .arg("includeNotPresent", include_not_present.into())
            .build();
        let res = serde_json::from_value::<ArtistInfo>(client.get("getArtistInfo", args)?)?;
        Ok(res.similar_artists)
    }

    /// Returns the top `count` most played songs released by the artist.
    pub fn top_songs<U>(&self, client: &Client, count: U) -> Result<Vec<Song>>
    where
        U: Into<Option<usize>>,
    {
        let args = Query::with("id", self.id)
            .arg("count", count.into())
            .build();

        let song = client.get("getTopSongs", args)?;
        Ok(get_list_as!(song, Song))
    }
}

impl<'de> Deserialize<'de> for Artist {
    fn deserialize<D>(de: D) -> ::std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _Artist {
            id: String,
            name: String,
            cover_art: Option<String>,
            album_count: usize,
            #[serde(default)]
            album: Vec<Album>,
        }

        let raw = _Artist::deserialize(de)?;

        Ok(Artist {
            id: raw.id.parse().unwrap(),
            name: raw.name,
            cover_id: raw.cover_art,
            album_count: raw.album_count,
            albums: raw.album,
        })
    }
}

impl Media for Artist {
    fn has_cover_art(&self) -> bool {
        self.cover_id.is_some()
    }

    fn cover_id(&self) -> Option<&str> {
        self.cover_id.as_deref()
    }

    fn cover_art<U: Into<Option<usize>>>(&self, client: &Client, size: U) -> Result<Vec<u8>> {
        let cover = self.cover_id().ok_or(Error::Other("no cover art found"))?;
        let query = Query::with("id", cover).arg("size", size.into()).build();

        client.get_bytes("getCoverArt", query)
    }

    fn cover_art_url<U: Into<Option<usize>>>(&self, client: &Client, size: U) -> Result<String> {
        let cover = self.cover_id().ok_or(Error::Other("no cover art found"))?;
        let query = Query::with("id", cover).arg("size", size.into()).build();

        client.build_url("getCoverArt", query)
    }
}

impl fmt::Display for Artist {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl<'de> Deserialize<'de> for ArtistInfo {
    fn deserialize<D>(de: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _ArtistInfo {
            biography: String,
            music_brainz_id: String,
            last_fm_url: String,
            small_image_url: String,
            medium_image_url: String,
            large_image_url: String,
            similar_artist: Vec<Artist>,
        }

        let raw = _ArtistInfo::deserialize(de)?;

        Ok(ArtistInfo {
            biography: raw.biography,
            musicbrainz_id: raw.music_brainz_id,
            lastfm_url: raw.last_fm_url,
            image_urls: (
                raw.small_image_url,
                raw.medium_image_url,
                raw.large_image_url,
            ),
            similar_artists: raw.similar_artist,
        })
    }
}

/// Fetches an artist from the Subsonic server.
fn get_artist(client: &Client, id: usize) -> Result<Artist> {
    let res = client.get("getArtist", Query::with("id", id))?;
    Ok(serde_json::from_value::<Artist>(res)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util;

    #[test]
    fn parse_artist() {
        let parsed = serde_json::from_value::<Artist>(raw()).unwrap();

        assert_eq!(parsed.id, 1);
        assert_eq!(parsed.name, String::from("Misteur Valaire"));
        assert_eq!(parsed.album_count, 1);
    }

    #[test]
    fn parse_artist_deep() {
        let parsed = serde_json::from_value::<Artist>(raw()).unwrap();

        assert_eq!(parsed.albums.len(), parsed.album_count);
        assert_eq!(parsed.albums[0].id, "1");
        assert_eq!(parsed.albums[0].name, String::from("Bellevue"));
        assert_eq!(parsed.albums[0].song_count, 9);
    }

    #[test]
    fn remote_artist_album_list() {
        let srv = test_util::demo_site().unwrap();
        let parsed = serde_json::from_value::<Artist>(raw()).unwrap();
        let albums = parsed.albums(&srv).unwrap();

        assert_eq!(albums[0].id, "1");
        assert_eq!(albums[0].name, String::from("Bellevue"));
        assert_eq!(albums[0].song_count, 9);
    }

    #[test]
    fn remote_artist_cover_art() {
        let srv = test_util::demo_site().unwrap();
        let parsed = serde_json::from_value::<Artist>(raw()).unwrap();
        assert_eq!(parsed.cover_id, Some(String::from("ar-1")));

        let cover = parsed.cover_art(&srv, None).unwrap();
        assert!(!cover.is_empty())
    }

    fn raw() -> serde_json::Value {
        serde_json::from_str(
            r#"{
            "id" : "1",
            "name" : "Misteur Valaire",
            "coverArt" : "ar-1",
            "albumCount" : 1,
            "album" : [ {
                "id" : "1",
                "name" : "Bellevue",
                "artist" : "Misteur Valaire",
                "artistId" : "1",
                "coverArt" : "al-1",
                "songCount" : 9,
                "duration" : 1920,
                "playCount" : 2223,
                "created" : "2017-03-12T11:07:25.000Z",
                "genre" : "(255)"
            } ]
        }"#,
        )
        .unwrap()
    }
}
