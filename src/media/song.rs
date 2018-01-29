use client::Client;
use error::{Error, Result};
use serde::de::{Deserialize, Deserializer};
use serde_json;

use library::search;
use media::{Media, StreamArgs, Streamable};
use query::Query;

#[derive(Debug, Clone)]
pub struct Song {
    pub id: u64,
    pub title: String,
    pub album: Option<String>,
    album_id: Option<u64>,
    pub artist: Option<String>,
    artist_id: Option<u64>,
    pub track: Option<u64>,
    pub year: Option<u64>,
    pub genre: Option<String>,
    cover_id: Option<String>,
    pub size: u64,
    content_type: String,
    suffix: String,
    transcoded_content_type: Option<String>,
    transcoded_suffix: Option<String>,
    pub duration: Option<u64>,
    path: String,
    pub media_type: String,
}

impl Song {
    /// Creates an HLS (HTTP Live Streaming) playlist used for streaming video
    /// or audio. HLS is a streaming protocol implemented by Apple and works by
    /// breaking the overall stream into a sequence of small HTTP-based file
    /// downloads. It's supported by iOS and newer versions of Android. This
    /// method also supports adaptive bitrate streaming, see the bitRate
    /// parameter.
    ///
    ///  Returns an M3U8 playlist on success (content type
    ///  "application/vnd.apple.mpegurl").
    pub fn hls(
        &self,
        client: &mut Client,
        bitrates: Vec<u64>,
    ) -> Result<String> {
        let args = Query::with("id", self.id)
            .arg_list("bitrate", bitrates)
            .build();

        client.get_raw("hls", args)
    }

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
}

impl Streamable for Song {
    fn stream<A>(&self, client: &mut Client, args: A) -> Result<Vec<u8>>
    where
        A: StreamArgs,
    {
        let mut q = Query::with("id", self.id);
        q.extend(args.into_arg_set());
        client.get_bytes("stream", q)
    }

    fn stream_url<A>(&self, client: &mut Client, args: A) -> Result<String>
    where
        A: StreamArgs,
    {
        let mut q = Query::with("id", self.id);
        q.extend(args.into_arg_set());
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
        })
    }
}

pub fn get_song(client: &mut Client, id: u64) -> Result<Song> {
    let res = client.get("getSong", Query::with("id", id))?;
    Ok(serde_json::from_value(res)?)
}

pub fn get_random_songs<'a, S, U>(
    client: &mut Client,
    size: U,
    genre: S,
    from_year: U,
    to_year: U,
    folder_id: U,
) -> Result<Vec<Song>>
where
    S: Into<Option<&'a str>>,
    U: Into<Option<u64>>,
{
    let args = Query::with("size", size.into().unwrap_or(10))
        .arg("genre", genre.into())
        .arg("fromYear", from_year.into())
        .arg("toYear", to_year.into())
        .arg("musicFolderId", folder_id.into())
        .build();

    let song = client.get("getRandomSongs", args)?;
    Ok(get_list_as!(song, Song))
}

pub fn get_songs_in_genre<U>(
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

/// Searches for lyrics matching the artist and title. Returns `None` if no
/// lyrics are found.
pub fn get_lyrics<'a, S>(
    client: &mut Client,
    artist: S,
    title: S,
) -> Result<Option<Lyrics>>
where
    S: Into<Option<&'a str>>,
{
    let args = Query::with("artist", artist.into())
        .arg("title", title.into())
        .build();
    let res = client.get("getLyrics", args)?;

    if res.get("value").is_some() {
        Ok(Some(serde_json::from_value(res)?))
    } else {
        Ok(None)
    }
}

#[derive(Debug, Deserialize)]
pub struct Lyrics {
    title: String,
    artist: String,
    value: String,
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

        let hls = song.hls(&mut srv, vec![]);
        assert!(hls.is_ok());
    }

    fn raw() -> serde_json::Value {
        json!({
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
        })
    }
}
