//! Album APIs.

use std::{fmt, result};

use serde::de::{Deserialize, Deserializer};
use serde_json;

use crate::query::{Arg, IntoArg, Query};
use crate::search::SearchPage;
use crate::{Client, Error, Media, Result, Song};

#[allow(missing_docs)]
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

impl fmt::Display for ListType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

impl Default for ListType {
    fn default() -> Self {
        ListType::AlphaByArtist
    }
}

impl IntoArg for ListType {
    fn into_arg(self) -> Arg {
        self.to_string().into_arg()
    }
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
#[readonly::make]
pub struct Album {
    pub id: String,
    pub name: String,
    pub artist: Option<String>,
    pub artist_id: Option<String>,
    pub cover_id: Option<String>,
    pub duration: u64,
    pub year: Option<u64>,
    pub genre: Option<String>,
    pub song_count: u64,
    pub songs: Vec<Song>,
}

impl Album {
    /// Returns a single album from the Subsonic server.
    ///
    /// # Errors
    ///
    /// Aside from errors the `Client` may cause, the method will error if
    /// there is no album matching the provided ID.
    pub fn get(client: &Client, id: String) -> Result<Album> {
        self::get_album(client, id)
    }

    /// Lists all albums on the server. Supports paging.
    pub fn list(
        client: &Client,
        list_type: ListType,
        page: SearchPage,
        folder: usize,
    ) -> Result<Vec<Album>> {
        self::get_albums(client, list_type, page.count, page.offset, folder)
    }

    /// Returns all songs in the album.
    pub fn songs(&self, client: &Client) -> Result<Vec<Song>> {
        if self.songs.len() as u64 != self.song_count {
            Ok(self::get_album(client, self.id.clone())?.songs)
        } else {
            Ok(self.songs.clone())
        }
    }

    /// Returns detailed information about the album.
    pub fn info(&self, client: &Client) -> Result<AlbumInfo> {
        let res = client.get("getArtistInfo", Query::with("id", self.id.clone()))?;
        Ok(serde_json::from_value(res)?)
    }
}

impl fmt::Display for Album {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref artist) = self.artist {
            write!(f, "{} - ", artist)?;
        } else {
            write!(f, "Unknown Artist - ")?;
        }

        write!(f, "{}", self.name)?;

        if let Some(year) = self.year {
            write!(f, " [{}] ", year)?;
        }

        Ok(())
    }
}

impl<'de> Deserialize<'de> for Album {
    fn deserialize<D>(de: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _Album {
            id: String,
            name: String,
            artist: Option<String>,
            artist_id: Option<String>,
            cover_art: Option<String>,
            song_count: u64,
            duration: u64,
            // created: String,
            year: Option<u64>,
            genre: Option<String>,
            #[serde(default)]
            song: Vec<Song>,
        }

        let raw = _Album::deserialize(de)?;

        Ok(Album {
            id: raw.id.parse().unwrap(),
            name: raw.name,
            artist: raw.artist,
            artist_id: raw.artist_id.map(|i| i.parse().unwrap()),
            cover_id: raw.cover_art,
            duration: raw.duration,
            year: raw.year,
            genre: raw.genre,
            song_count: raw.song_count,
            songs: raw.song,
        })
    }
}

impl Media for Album {
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

#[allow(missing_docs)]
#[derive(Debug)]
pub struct AlbumInfo {
    pub notes: String,
    pub lastfm_url: String,
    pub musicbrainz_id: String,
    pub image_urls: (String, String, String),
}

impl<'de> Deserialize<'de> for AlbumInfo {
    fn deserialize<D>(de: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _AlbumInfo {
            notes: String,
            music_brainz_id: String,
            last_fm_url: String,
            small_image_url: String,
            medium_image_url: String,
            large_image_url: String,
        }

        let raw = _AlbumInfo::deserialize(de)?;

        Ok(AlbumInfo {
            notes: raw.notes,
            musicbrainz_id: raw.music_brainz_id,
            lastfm_url: raw.last_fm_url,
            image_urls: (
                raw.small_image_url,
                raw.medium_image_url,
                raw.large_image_url,
            ),
        })
    }
}

fn get_album(client: &Client, id: String) -> Result<Album> {
    let res = client.get("getAlbum", Query::with("id", id))?;
    Ok(serde_json::from_value::<Album>(res)?)
}

fn get_albums<U>(
    client: &Client,
    list_type: ListType,
    size: U,
    offset: U,
    folder_id: U,
) -> Result<Vec<Album>>
where
    U: Into<Option<usize>>,
{
    let args = Query::new()
        .arg("type", list_type)
        .arg("size", size.into())
        .arg("offset", offset.into())
        .arg("musicFolderId", folder_id.into())
        .build();

    let album = client.get("getAlbumList2", args)?;
    Ok(get_list_as!(album, Album))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util;

    #[test]
    fn demo_get_albums() {
        let srv = test_util::demo_site().unwrap();
        let albums = get_albums(&srv, ListType::AlphaByArtist, None, None, None).unwrap();

        assert!(!albums.is_empty())
    }

    #[test]
    fn parse_album() {
        let parsed = serde_json::from_value::<Album>(raw()).unwrap();

        assert_eq!(parsed.id, "1");
        assert_eq!(parsed.name, String::from("Bellevue"));
        assert_eq!(parsed.song_count, 9);
    }

    #[test]
    fn parse_album_deep() {
        let parsed = serde_json::from_value::<Album>(raw()).unwrap();

        assert_eq!(parsed.songs[0].id, "27");
        assert_eq!(parsed.songs[0].title, String::from("Bellevue Avenue"));
        assert_eq!(parsed.songs[0].duration, Some(198));
    }

    fn raw() -> serde_json::Value {
        serde_json::from_str(r#"{
         "id" : "1",
         "name" : "Bellevue",
         "artist" : "Misteur Valaire",
         "artistId" : "1",
         "coverArt" : "al-1",
         "songCount" : 9,
         "duration" : 1920,
         "playCount" : 2223,
         "created" : "2017-03-12T11:07:25.000Z",
         "genre" : "(255)",
         "song" : [ {
            "id" : "27",
            "parent" : "25",
            "isDir" : false,
            "title" : "Bellevue Avenue",
            "album" : "Bellevue",
            "artist" : "Misteur Valaire",
            "track" : 1,
            "genre" : "(255)",
            "coverArt" : "25",
            "size" : 5400185,
            "contentType" : "audio/mpeg",
            "suffix" : "mp3",
            "duration" : 198,
            "bitRate" : 216,
            "path" : "Misteur Valaire/Bellevue/01 - Misteur Valaire - Bellevue Avenue.mp3",
            "averageRating" : 3.0,
            "playCount" : 706,
            "created" : "2017-03-12T11:07:27.000Z",
            "starred" : "2017-06-01T19:48:25.635Z",
            "albumId" : "1",
            "artistId" : "1",
            "type" : "music"
         }, {
            "id" : "31",
            "parent" : "25",
            "isDir" : false,
            "title" : "Don't Get Là",
            "album" : "Bellevue",
            "artist" : "Misteur Valaire",
            "track" : 2,
            "genre" : "(255)",
            "coverArt" : "25",
            "size" : 4866004,
            "contentType" : "audio/mpeg",
            "suffix" : "mp3",
            "duration" : 172,
            "bitRate" : 224,
            "path" : "Misteur Valaire/Bellevue/02 - Misteur Valaire - Don_t Get L.mp3",
            "playCount" : 310,
            "created" : "2017-03-12T11:07:28.000Z",
            "starred" : "2017-08-27T07:52:23.926Z",
            "albumId" : "1",
            "artistId" : "1",
            "type" : "music"
         }, {
            "id" : "29",
            "parent" : "25",
            "isDir" : false,
            "title" : "Space Food",
            "album" : "Bellevue",
            "artist" : "Misteur Valaire",
            "track" : 3,
            "genre" : "(255)",
            "coverArt" : "25",
            "size" : 8954200,
            "contentType" : "audio/mpeg",
            "suffix" : "mp3",
            "duration" : 303,
            "bitRate" : 235,
            "path" : "Misteur Valaire/Bellevue/03 - Misteur Valaire - Space Food.mp3",
            "playCount" : 233,
            "created" : "2017-03-12T11:07:26.000Z",
            "albumId" : "1",
            "artistId" : "1",
            "type" : "music"
         }, {
            "id" : "32",
            "parent" : "25",
            "isDir" : false,
            "title" : "Known By Sight (feat. Milk & Bone)",
            "album" : "Bellevue",
            "artist" : "Misteur Valaire",
            "track" : 4,
            "genre" : "(255)",
            "coverArt" : "25",
            "size" : 6219273,
            "contentType" : "audio/mpeg",
            "suffix" : "mp3",
            "duration" : 231,
            "bitRate" : 214,
            "path" : "Misteur Valaire/Bellevue/04 - Misteur Valaire - Known By Sight _feat. Milk _ Bone_.mp3",
            "playCount" : 216,
            "created" : "2017-03-12T11:07:27.000Z",
            "albumId" : "1",
            "artistId" : "1",
            "type" : "music"
         }, {
            "id" : "33",
            "parent" : "25",
            "isDir" : false,
            "title" : "La Nature à Son Meilleur",
            "album" : "Bellevue",
            "artist" : "Misteur Valaire",
            "track" : 5,
            "genre" : "(255)",
            "coverArt" : "25",
            "size" : 5169929,
            "contentType" : "audio/mpeg",
            "suffix" : "mp3",
            "duration" : 187,
            "bitRate" : 220,
            "path" : "Misteur Valaire/Bellevue/05 - Misteur Valaire - La Nature  Son Meilleur.mp3",
            "playCount" : 190,
            "created" : "2017-03-12T11:07:26.000Z",
            "albumId" : "1",
            "artistId" : "1",
            "type" : "music"
         }, {
            "id" : "34",
            "parent" : "25",
            "isDir" : false,
            "title" : "Interlude",
            "album" : "Bellevue",
            "artist" : "Misteur Valaire",
            "track" : 6,
            "genre" : "(255)",
            "coverArt" : "25",
            "size" : 2403983,
            "contentType" : "audio/mpeg",
            "suffix" : "mp3",
            "duration" : 99,
            "bitRate" : 191,
            "path" : "Misteur Valaire/Bellevue/06 - Misteur Valaire - Interlude.mp3",
            "playCount" : 149,
            "created" : "2017-03-12T11:07:28.000Z",
            "albumId" : "1",
            "artistId" : "1",
            "type" : "music"
         }, {
            "id" : "28",
            "parent" : "25",
            "isDir" : false,
            "title" : "Old Orford",
            "album" : "Bellevue",
            "artist" : "Misteur Valaire",
            "track" : 7,
            "genre" : "(255)",
            "coverArt" : "25",
            "size" : 6403652,
            "contentType" : "audio/mpeg",
            "suffix" : "mp3",
            "duration" : 223,
            "bitRate" : 228,
            "path" : "Misteur Valaire/Bellevue/07 - Misteur Valaire - Old Orford.mp3",
            "playCount" : 160,
            "created" : "2017-03-12T11:07:25.000Z",
            "albumId" : "1",
            "artistId" : "1",
            "type" : "music"
         }, {
            "id" : "30",
            "parent" : "25",
            "isDir" : false,
            "title" : "El Kid",
            "album" : "Bellevue",
            "artist" : "Misteur Valaire",
            "track" : 8,
            "genre" : "(255)",
            "coverArt" : "25",
            "size" : 6506923,
            "contentType" : "audio/mpeg",
            "suffix" : "mp3",
            "duration" : 234,
            "bitRate" : 221,
            "path" : "Misteur Valaire/Bellevue/08 - Misteur Valaire - El Kid.mp3",
            "playCount" : 134,
            "created" : "2017-03-12T11:07:28.000Z",
            "albumId" : "1",
            "artistId" : "1",
            "type" : "music"
         }, {
            "id" : "26",
            "parent" : "25",
            "isDir" : false,
            "title" : "Banana Land",
            "album" : "Bellevue",
            "artist" : "Misteur Valaire",
            "track" : 9,
            "genre" : "(255)",
            "coverArt" : "25",
            "size" : 6870947,
            "contentType" : "audio/mpeg",
            "suffix" : "mp3",
            "duration" : 273,
            "bitRate" : 200,
            "path" : "Misteur Valaire/Bellevue/09 - Misteur Valaire - Banana Land.mp3",
            "playCount" : 125,
            "created" : "2017-03-12T11:07:25.000Z",
            "albumId" : "1",
            "artistId" : "1",
            "type" : "music"
         } ]
      }"#).unwrap()
    }
}
