use serde::de::{Deserialize, Deserializer};
use serde_json;

use client::Client;
use error::{Error, Result};
use library::search;
use media::{Media, Streamable};
use query::Query;

/// A work of music contained on a Subsonic server.
#[derive(Debug, Clone)]
pub struct Song {
    /// Unique identifier for the song.
    pub id: u64,
    /// Title of the song. Prefers the song's ID3 tags, but will fall back to
    /// the file name.
    pub title: String,
    /// Album the song belongs to. Reads from the song's ID3 tags.
    pub album: Option<String>,
    album_id: Option<u64>,
    /// Credited artist for the song. Reads from the song's ID3 tags.
    pub artist: Option<String>,
    artist_id: Option<u64>,
    /// Position of the song in the album.
    pub track: Option<u64>,
    /// Year the song was released.
    pub year: Option<u64>,
    /// Genre of the song.
    pub genre: Option<String>,
    cover_id: Option<String>,
    /// File size of the song, in bytes.
    pub size: u64,
    content_type: String,
    suffix: String,
    transcoded_content_type: Option<String>,
    transcoded_suffix: Option<String>,
    /// Duration of the song, in seconds.
    pub duration: Option<u64>,
    path: String,
    media_type: String,
    stream_br: Option<usize>,
    stream_tc: Option<String>,
}

impl Song {
    /// Returns a single song from the Subsonic server.
    ///
    /// # Errors
    ///
    /// The server will return an error if there is no song matching the
    /// provided ID.
    pub fn get(client: &mut Client, id: u64) -> Result<Song> {
        let res = client.get("getSong", Query::with("id", id))?;
        Ok(serde_json::from_value(res)?)
    }

    /// Returns a number of random songs similar to this one.
    ///
    /// last.fm suggests a number of similar songs to the one the method is
    /// called on. Optionally takes a `count` to specify the maximum number of
    /// results to return.
    pub fn similar<U>(&self, client: &mut Client, count: U) -> Result<Vec<Song>>
    where
        U: Into<Option<usize>>,
    {
        let args = Query::with("id", self.id)
            .arg("count", count.into())
            .build();

        let song = client.get("getSimilarSongs2", args)?;
        Ok(get_list_as!(song, Song))
    }

    /// Returns a number of random songs. Optionally accepts a maximum number of results to return.
    ///
    /// Some parts of the query can be modified. Use [`random_with`] to be able to set these
    /// optional fields.
    ///
    /// [`random_with`]: #method.random_with
    pub fn random<S, U>( client: &mut Client, size: U) -> Result<Vec<Song>>
    where
        U: Into<Option<usize>>,
    {
        let arg = Query::with("size", size.into().unwrap_or(10));
        let song = client.get("getRandomSongs", arg)?;
        Ok(get_list_as!(song, Song))
    }

    /// Creates a new builder to request a set of random songs.
    ///
    /// See the [struct level documentation] for more information on how to use the builder.
    ///
    /// [struct level documentation]: struct.RandomSongs.html
    pub fn random_with<'a>(client: &mut Client) -> RandomSongs {
        RandomSongs::new(client, 10)
    }

    /// Lists all the songs in a provided genre. Supports paging through the
    /// result.
    ///
    /// See the documentation for [`SearchPage`] for paging.
    ///
    /// [`SearchPage`]: ../../library/search/struct.SearchPage.html
    pub fn list_in_genre<U>(
        client: &mut Client,
        genre: &str,
        page: search::SearchPage,
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
    pub fn hls(
        &self,
        client: &mut Client,
        bit_rates: &[u64],
    ) -> Result<String> {
        let args = Query::with("id", self.id)
            .arg_list("bitrate", bit_rates)
            .build();

        client.get_raw("hls", args)
    }
}

impl Streamable for Song {
    fn stream(&self, client: &mut Client) -> Result<Vec<u8>> {
        let mut q = Query::with("id", self.id);
        q.arg("maxBitRate", self.stream_br);
        client.get_bytes("stream", q)
    }

    fn stream_url(&self, client: &mut Client) -> Result<String> {
        let mut q = Query::with("id", self.id);
        q.arg("maxBitRate", self.stream_br);
        client.build_url("stream", q)
    }

    fn download(&self, client: &mut Client) -> Result<Vec<u8>> {
        client.get_bytes("download", Query::with("id", self.id))
    }

    fn download_url(&self, client: &mut Client) -> Result<String> {
        client.build_url("download", Query::with("id", self.id))
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
    fn has_cover_art(&self) -> bool { self.cover_id.is_some() }

    fn cover_id(&self) -> Option<&str> {
        self.cover_id.as_ref().map(|s| s.as_str())
    }

    fn cover_art<U: Into<Option<usize>>>(
        &self,
        client: &mut Client,
        size: U,
    ) -> Result<Vec<u8>> {
        let cover = self.cover_id()
            .ok_or_else(|| Error::Other("no cover art found"))?;
        let query = Query::with("id", cover).arg("size", size.into()).build();

        client.get_bytes("getCoverArt", query)
    }

    fn cover_art_url<U: Into<Option<usize>>>(
        &self,
        client: &mut Client,
        size: U,
    ) -> Result<String> {
        let cover = self.cover_id()
            .ok_or_else(|| Error::Other("no cover art found"))?;
        let query = Query::with("id", cover).arg("size", size.into()).build();

        client.build_url("getCoverArt", query)
    }
}

impl<'de> Deserialize<'de> for Song {
    fn deserialize<D>(de: D) -> ::std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _Song {
            id: String,
            parent: String,
            is_dir: bool,
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
            bit_rate: u64,
            path: String,
            is_video: Option<bool>,
            play_count: u64,
            disc_number: Option<u64>,
            created: String,
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
    pub title: String,
    pub artist: String,
    #[serde(rename = "value")]
    pub lyrics: String,
}

/// A builder struct for a query of random songs.
///
/// A `RandomSongs` can only be created with [`Song::random_with`]. This allows customisation of the
/// results to return.
///
/// The builder holds an internal reference of the client that it will query using, so there's no
/// need to provide it with one when sending the query.
///
/// If you don't need to customise a query and just need a set of random songs, use
/// [`Song::random`] instead, as it skips constructing the builder and directly queries the
/// Subsonic server.
///
/// [`Song::random_with`]: struct.Song.html#method.random_with
/// [`Song::random`]: struct.Song.html#method.random
///
/// # Examples
///
/// ```no_run
/// extern crate sunk;
/// use sunk::Client;
/// use sunk::media::Song;
///
/// # fn run() -> sunk::error::Result<()> {
/// let mut server = Client::new("http://demo.subsonic.org", "guest3", "guest")?;
///
/// // Get 25 songs from the last 10 years
/// let random = Song::random_with(&mut server)
///                  .size(25)
///                  .in_years(2008..2018)
///                  .request()?;
/// # Ok(())
/// # }
/// # fn main() {
/// # run().unwrap();
/// # }
/// ```
#[derive(Debug)]
pub struct RandomSongs<'a> {
    client: &'a mut Client,
    size: usize,
    genre: Option<&'a str>,
    from_year: Option<usize>,
    to_year: Option<usize>,
    folder_id: Option<usize>,
}

use std::ops::Range;
impl<'a> RandomSongs<'a> {
    fn new(client: &'a mut Client, n: usize) -> RandomSongs<'a> {
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
    /// Genres will vary between Subsonic instances, but can be found using the [`Client::genres`]
    /// method.
    ///
    /// [`Client::genres`]: ../../client/struct.Client.html#method.genres
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
    /// The range is set *inclusive* at both ends, unlike a standard Rust range. For example, a
    /// range `2013..2016` will return songs that were released in 2013, 2014, 2015, and 2016.
    pub fn in_years(&mut self, years: Range<usize>) -> &mut RandomSongs<'a> {
        self.from_year = Some(years.start);
        self.to_year = Some(years.end);
        self
    }

    /// Sets the folder index that songs must be in.
    ///
    /// Music folders are zero-indexed, and there will always be index `0` (provided the server
    /// is configured at all) . A list of music folders can be found using the
    /// [`Client::music_folders`] method.
    ///
    /// [`Client::music_folders`]: ../../client/struct.Client.html#method.music_folders
    pub fn in_folder(&mut self, id: usize) -> &mut RandomSongs<'a> {
        self.folder_id = Some(id);
        self
    }

    /// Issues the query to the Subsonic server. Returns a list of random songs, modified by the
    /// builder.
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
    use test_util;

    #[test]
    fn parse_song() {
        let parsed = serde_json::from_value::<Song>(raw()).unwrap();

        assert_eq!(parsed.id, 27);
        assert_eq!(parsed.title, String::from("Bellevue Avenue"));
        assert_eq!(parsed.track, Some(1));
    }

    #[test]
    fn get_hls() {
        let mut srv = test_util::demo_site().unwrap();
        let song = serde_json::from_value::<Song>(raw()).unwrap();

        let hls = song.hls(&mut srv, &[]);
        assert!(hls.is_ok());
    }

    fn raw() -> serde_json::Value {
        serde_json::from_str(r#"{
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
        }"#).unwrap()
    }
}
