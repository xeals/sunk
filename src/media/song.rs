//! Song APIs.

use std::fmt;
use std::ops::Range;

use serde::de::{Deserialize, Deserializer};
use serde_json;

use crate::query::Query;
use crate::search::SearchPage;
use crate::{Client, Error, HlsPlaylist, Media, Result, Streamable};

/// A work of music contained on a Subsonic server.
#[derive(Debug, Clone)]
#[readonly::make]
pub struct Song {
    /// Unique identifier for the song.
    pub id: String,
    /// Title of the song. Prefers the song's ID3 tags, but will fall back to
    /// the file name.
    pub title: String,
    /// Album the song belongs to. Reads from the song's ID3 tags.
    pub album: Option<String>,
    /// The ID of the released album.
    pub album_id: Option<String>,
    /// Credited artist for the song. Reads from the song's ID3 tags.
    pub artist: Option<String>,
    /// The ID of the releasing artist.
    pub artist_id: Option<String>,
    /// Position of the song in the album.
    pub track: Option<u64>,
    /// Year the song was released.
    pub year: Option<u64>,
    /// Genre of the song.
    pub genre: Option<String>,
    /// ID of the song's cover art. Defaults to the parent album's cover.
    pub cover_id: Option<String>,
    /// File size of the song, in bytes.
    pub size: u64,
    /// An audio MIME type.
    pub content_type: String,
    /// The file extension of the song.
    pub suffix: String,
    /// The MIME type that the song will be transcoded to.
    pub transcoded_content_type: Option<String>,
    /// The file extension that the song will be transcoded to.
    pub transcoded_suffix: Option<String>,
    /// Duration of the song, in seconds.
    pub duration: Option<u64>,
    /// The absolute path of the song in the server database.
    pub path: String,
    /// Will always be "song".
    pub media_type: String,
    /// Bit rate the song will be downsampled to.
    pub stream_br: Option<usize>,
    /// Format the song will be transcoded to.
    pub stream_tc: Option<String>,
}

impl Song {
    /// Returns a single song from the Subsonic server.
    ///
    /// # Errors
    ///
    /// Aside from other errors the `Client` may cause, the server will return
    /// an error if there is no song matching the provided ID.
    pub fn get(client: &Client, id: String) -> Result<Song> {
        let res = client.get("getSong", Query::with("id", id))?;
        Ok(serde_json::from_value(res)?)
    }

    /// Returns a number of random songs similar to this one.
    ///
    /// last.fm suggests a number of similar songs to the one the method is
    /// called on. Optionally takes a `count` to specify the maximum number of
    /// results to return.
    pub fn similar<U>(&self, client: &Client, count: U) -> Result<Vec<Song>>
    where
        U: Into<Option<usize>>,
    {
        let args = Query::with("id", self.id.clone())
            .arg("count", count.into())
            .build();

        let song = client.get("getSimilarSongs2", args)?;
        Ok(get_list_as!(song, Song))
    }

    /// Returns a number of random songs. Optionally accepts a maximum number
    /// of results to return.
    ///
    /// Some parts of the query can be modified. Use [`random_with`] to be able
    /// to set these optional fields.
    ///
    /// [`random_with`]: #method.random_with
    pub fn random<U>(client: &Client, size: U) -> Result<Vec<Song>>
    where
        U: Into<Option<usize>>,
    {
        let arg = Query::with("size", size.into().unwrap_or(10));
        let song = client.get("getRandomSongs", arg)?;
        Ok(get_list_as!(song, Song))
    }

    /// Creates a new builder to request a set of random songs.
    ///
    /// See the [struct level documentation] for more information on how to use
    /// the builder.
    ///
    /// [struct level documentation]: ./struct.RandomSongs.html
    pub fn random_with(client: &Client) -> RandomSongs {
        RandomSongs::new(client, 10)
    }

    /// Lists all the songs in a provided genre. Supports paging through the
    /// result.
    ///
    /// See the [struct level documentation] about paging for more.
    ///
    /// [struct level documentation]: ../search/struct.SearchPage.html
    pub fn list_in_genre<U>(
        client: &Client,
        genre: &str,
        page: SearchPage,
        folder_id: U,
    ) -> Result<Vec<Song>>
    where
        U: Into<Option<u64>>,
    {
        let args = Query::with("genre", genre)
            .arg("count", page.count)
            .arg("offset", page.offset)
            .arg("musicFolderId", folder_id.into())
            .build();

        let song = client.get("getSongsByGenre", args)?;
        Ok(get_list_as!(song, Song))
    }

    /// Creates an HLS (HTTP Live Streaming) playlist used for streaming video
    /// or audio. HLS is a streaming protocol implemented by Apple and works by
    /// breaking the overall stream into a sequence of small HTTP-based file
    /// downloads. It's supported by iOS and newer versions of Android.
    ///
    ///  Returns an M3U8 playlist on success (content type
    ///  "application/vnd.apple.mpegurl").
    ///
    /// The method also supports adaptive streaming; when supplied with multiple
    /// bit rates, the server will create a variable playlist, suitable for
    /// adaptive bitrate streaming. The playlist will support streaming at all
    /// the specified bitrates. The `bit_rate` parameter can be omitted (with an
    /// empty array) to disable adaptive streaming, or given a single value to
    /// force streaming at that bit rate.
    pub fn hls(&self, client: &Client, bit_rates: &[u64]) -> Result<HlsPlaylist> {
        let args = Query::with("id", self.id.clone())
            .arg_list("bitrate", bit_rates)
            .build();

        let raw = client.get_raw("hls", args)?;
        raw.parse::<HlsPlaylist>()
    }
}

impl Streamable for Song {
    fn stream(&self, client: &Client) -> Result<Vec<u8>> {
        let mut q = Query::with("id", self.id.clone());
        q.arg("maxBitRate", self.stream_br);
        client.get_bytes("stream", q)
    }

    fn stream_url(&self, client: &Client) -> Result<String> {
        let mut q = Query::with("id", self.id.clone());
        q.arg("maxBitRate", self.stream_br);
        client.build_url("stream", q)
    }

    fn download(&self, client: &Client) -> Result<Vec<u8>> {
        client.get_bytes("download", Query::with("id", self.id.clone()))
    }

    fn download_url(&self, client: &Client) -> Result<String> {
        client.build_url("download", Query::with("id", self.id.clone()))
    }

    fn encoding(&self) -> &str {
        self.transcoded_content_type
            .as_ref()
            .unwrap_or(&self.content_type)
    }

    fn set_max_bit_rate(&mut self, bit_rate: usize) {
        self.stream_br = Some(bit_rate);
    }

    fn set_transcoding(&mut self, format: &str) {
        self.stream_tc = Some(format.to_string());
    }
}

impl Media for Song {
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

impl fmt::Display for Song {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref artist) = self.artist {
            write!(f, "{} - ", artist)?;
        } else {
            write!(f, "Unknown Artist - ")?;
        }

        if let Some(ref album) = self.album {
            write!(f, "{}", album)?;
        } else {
            write!(f, "Unknown Album")?;
        }

        if let Some(year) = self.year {
            write!(f, " [{}]", year)?;
        }

        write!(f, " - {}", self.title)?;

        Ok(())
    }
}

impl<'de> Deserialize<'de> for Song {
    fn deserialize<D>(de: D) -> ::std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _Song {
            id: String,
            // parent: String,
            // is_dir: bool,
            title: String,
            album: Option<String>,
            artist: Option<String>,
            track: Option<u64>,
            year: Option<u64>,
            genre: Option<String>,
            cover_art: Option<String>,
            size: u64,
            content_type: String,
            suffix: String,
            transcoded_content_type: Option<String>,
            transcoded_suffix: Option<String>,
            duration: Option<u64>,
            // bit_rate: Option<u64>,
            path: String,
            // is_video: Option<bool>,
            // play_count: u64,
            // disc_number: Option<u64>,
            // created: String,
            album_id: Option<String>,
            artist_id: Option<String>,
            #[serde(rename = "type")]
            media_type: String,
        }

        let raw = _Song::deserialize(de)?;

        Ok(Song {
            id: raw.id.parse().unwrap(),
            title: raw.title,
            album: raw.album,
            album_id: raw.album_id.map(|i| i.parse().unwrap()),
            artist: raw.artist,
            artist_id: raw.artist_id.map(|i| i.parse().unwrap()),
            cover_id: raw.cover_art,
            track: raw.track,
            year: raw.year,
            genre: raw.genre,
            size: raw.size,
            content_type: raw.content_type,
            suffix: raw.suffix,
            transcoded_content_type: raw.transcoded_content_type,
            transcoded_suffix: raw.transcoded_suffix,
            duration: raw.duration,
            path: raw.path,
            media_type: raw.media_type,
            stream_br: None,
            stream_tc: None,
        })
    }
}

/// A struct matching a lyric search result.
#[derive(Debug, Deserialize)]
pub struct Lyrics {
    /// Title of the song.
    pub title: String,
    /// Artist that performed the song.
    pub artist: String,
    /// Lyrics to the song.
    #[serde(rename = "value")]
    pub lyrics: String,
}

/// A builder struct for a query of random songs.
///
/// A `RandomSongs` can only be created with [`Song::random_with`]. This allows
/// customisation of the results to return.
///
/// The builder holds an internal reference of the client that it will query
/// using, so there's no need to provide it with one when sending the query.
///
/// If you don't need to customise a query and just need a set of random songs,
/// use [`Song::random`] instead, as it skips constructing the builder and
/// directly queries the Subsonic server.
///
/// [`Song::random_with`]: ./struct.Song.html#method.random_with
/// [`Song::random`]: ./struct.Song.html#method.random
///
/// # Examples
///
/// ```no_run
/// extern crate sunk;
/// use sunk::song::Song;
/// use sunk::Client;
///
/// # fn run() -> sunk::Result<()> {
/// # let site = "http://demo.subsonic.org";
/// # let user = "guest3";
/// # let password = "guest";
/// let client = Client::new(site, user, password)?;
///
/// // Get 25 songs from the last 10 years
/// let random = Song::random_with(&client)
///     .size(25)
///     .in_years(2008 .. 2018)
///     .request()?;
/// # Ok(())
/// # }
/// # fn main() { }
/// ```
#[derive(Debug)]
pub struct RandomSongs<'a> {
    client: &'a Client,
    size: usize,
    genre: Option<&'a str>,
    from_year: Option<usize>,
    to_year: Option<usize>,
    folder_id: Option<usize>,
}

impl<'a> RandomSongs<'a> {
    fn new(client: &'a Client, n: usize) -> RandomSongs<'a> {
        RandomSongs {
            client,
            size: n,
            genre: None,
            from_year: None,
            to_year: None,
            folder_id: None,
        }
    }

    /// Sets the number of songs to return.
    pub fn size(&mut self, n: usize) -> &mut RandomSongs<'a> {
        self.size = n;
        self
    }

    /// Sets the genre that songs will be in.
    ///
    /// Genres will vary between Subsonic instances, but can be found using the
    /// [`Client::genres`] method.
    ///
    /// [`Client::genres`]: ../struct.Client.html#method.genres
    pub fn genre(&mut self, genre: &'a str) -> &mut RandomSongs<'a> {
        self.genre = Some(genre);
        self
    }

    /// Sets a lower bound on the year that songs were released in.
    pub fn from_year(&mut self, year: usize) -> &mut RandomSongs<'a> {
        self.from_year = Some(year);
        self
    }

    /// Sets an upper bound on the year that songs were released in.
    pub fn to_year(&mut self, year: usize) -> &mut RandomSongs<'a> {
        self.to_year = Some(year);
        self
    }

    /// Sets both the lower and upper year bounds using a range.
    ///
    /// The range is set *inclusive* at both ends, unlike a standard Rust
    /// range. For example, a range `2013..2016` will return songs that
    /// were released in 2013, 2014, 2015, and 2016.
    pub fn in_years(&mut self, years: Range<usize>) -> &mut RandomSongs<'a> {
        self.from_year = Some(years.start);
        self.to_year = Some(years.end);
        self
    }

    /// Sets the folder index that songs must be in.
    ///
    /// Music folders are zero-indexed, and there will always be index `0`
    /// (provided the server is configured at all) . A list of music
    /// folders can be found using the [`Client::music_folders`] method.
    ///
    /// [`Client::music_folders`]: ../struct.Client.html#method.music_folders
    pub fn in_folder(&mut self, id: usize) -> &mut RandomSongs<'a> {
        self.folder_id = Some(id);
        self
    }

    /// Issues the query to the Subsonic server. Returns a list of random
    /// songs, modified by the builder.
    pub fn request(&mut self) -> Result<Vec<Song>> {
        let args = Query::with("size", self.size)
            .arg("genre", self.genre)
            .arg("fromYear", self.from_year)
            .arg("toYear", self.to_year)
            .arg("musicFolderId", self.folder_id)
            .build();

        let song = self.client.get("getRandomSongs", args)?;
        Ok(get_list_as!(song, Song))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util;

    #[test]
    fn parse_song() {
        let parsed = serde_json::from_value::<Song>(raw()).unwrap();

        assert_eq!(parsed.id, "27");
        assert_eq!(parsed.title, String::from("Bellevue Avenue"));
        assert_eq!(parsed.track, Some(1));
    }

    #[test]
    fn get_hls() {
        let srv = test_util::demo_site().unwrap();
        let song = serde_json::from_value::<Song>(raw()).unwrap();

        let hls = song.hls(&srv, &[]).unwrap();
        assert_eq!(hls.len(), 20)
    }

    fn raw() -> serde_json::Value {
        serde_json::from_str(
            r#"{
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
        }"#,
        )
        .unwrap()
    }
}
