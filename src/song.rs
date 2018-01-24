use serde::de::{Deserialize, Deserializer};
use error::*;
use serde_json;
use sunk::Sunk;

use query::Query;
use util::*;
use library::search;

/// Audio encoding format.
///
/// Recognises all of Subsonic's default transcoding formats.
#[derive(Debug)]
pub enum AudioFormat {
    Aac,
    Aif,
    Aiff,
    Ape,
    Flac,
    Flv,
    M4a,
    Mp3,
    Mpc,
    Oga,
    Ogg,
    Ogx,
    Opus,
    Shn,
    Wav,
    Wma,
    Raw,
}

impl ::std::fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Debug, Clone)]
pub struct Song {
    pub id: u64,
    // parent: u64,
    pub title: String,
    pub album: Option<String>,
    album_id: Option<u64>,
    pub artist: Option<String>,
    artist_id: Option<u64>,
    pub track: Option<u64>,
    pub year: Option<u64>,
    pub genre: Option<String>,
    cover_id: Option<u64>,
    size: u64,
    duration: u64,
    path: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct SongSerde {
    id: String,
    parent: String,
    isDir: bool,
    title: String,
    album: Option<String>,
    artist: Option<String>,
    track: Option<u64>,
    year: Option<u64>,
    genre: Option<String>,
    coverArt: Option<String>,
    size: u64,
    contentType: String,
    suffix: String,
    duration: u64,
    bitRate: u64,
    path: String,
    isVideo: Option<bool>,
    playCount: u64,
    discNumber: Option<u64>,
    created: String,
    albumId: Option<String>,
    artistId: Option<String>,
    // type: String,
}

impl Song {
    pub fn try_from(json: serde_json::Value) -> Result<Song> {
        let serde: SongSerde = serde_json::from_value(json)?;
        Ok(Song {
            id: serde.id.parse()?,
            title: serde.title,
            album: serde.album,
            album_id: serde.albumId.map(|i| i.parse().unwrap()),
            artist: serde.artist,
            artist_id: serde.artistId.map(|i| i.parse().unwrap()),
            cover_id: serde.coverArt.map(|i| i.parse().unwrap()),
            track: serde.track,
            year: serde.year,
            genre: serde.genre,
            size: serde.size,
            duration: serde.duration,
            path: serde.path,
        })
    }

    /// Returns a constructed URL for streaming with desired arguments.
    ///
    /// This would be used in conjunction with a streaming library to directly
    /// take the URI and stream it.
    pub fn stream_url(
        &self,
        sunk: &mut Sunk,
        bitrate: Option<u64>,
        format: Option<AudioFormat>,
    ) -> Result<String> {
        let args = Query::new()
            .arg("id", self.id.to_string())
            .maybe_arg("maxBitRate", map_str(bitrate))
            .maybe_arg("format", map_str(format))
            .build();
        sunk.try_binary("stream", args)
    }

    /// Returns a constructed URL for downloading the song.
    ///
    /// `download_url()` does not support transcoding, while `stream_url()`
    /// does.
    pub fn download_url(&self, sunk: &mut Sunk) -> Result<String> {
        self.stream_url(sunk, None, None)
    }

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
        sunk: &mut Sunk,
        bitrates: Option<Vec<u64>>,
    ) -> Result<String> {
        let args = Query::new()
            .arg("id", self.id)
            .maybe_arg_list("bitrate", bitrates)
            .build();

        sunk.get_raw("hls", args)
    }

    /// Returns the URL of the cover art. Size is a single parameter and the
    /// image will be scaled on its longest edge.
    impl_cover_art!();
}

impl<'de> Deserialize<'de> for Song {
    fn deserialize<D>(de: D) -> ::std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let raw = SongSerde::deserialize(de)?;
        Ok(Song {
            id: raw.id.parse().unwrap(),
            title: raw.title,
            album: raw.album,
            album_id: raw.albumId.map(|i| i.parse().unwrap()),
            artist: raw.artist,
            artist_id: raw.artistId.map(|i| i.parse().unwrap()),
            cover_id: raw.coverArt.map(|i| i.parse().unwrap()),
            track: raw.track,
            year: raw.year,
            genre: raw.genre,
            size: raw.size,
            duration: raw.duration,
            path: raw.path,
        })
    }
}

pub fn get_song(sunk: &mut Sunk, id: u64) -> Result<Song> {
    let res = sunk.get("getSong", Query::with("id", id))?;
    Ok(serde_json::from_value(res)?)
}

pub fn get_random_songs(
    sunk: &mut Sunk,
    size: Option<u64>,
    genre: Option<&str>,
    from_year: Option<usize>,
    to_year: Option<usize>,
    folder_id: Option<usize>,
) -> Result<Vec<Song>>
{
    let args = Query::new()
        .arg("size", size.unwrap_or(10).to_string())
        .maybe_arg("genre", map_str(genre))
        .maybe_arg("fromYear", map_str(from_year))
        .maybe_arg("toYear", map_str(to_year))
        .maybe_arg("musicFolderId", map_str(folder_id))
        .build();

    let song = sunk.get("getRandomSongs", args)?;
    Ok(get_list_as!(song, Song))
}

pub fn get_songs_in_genre(
    sunk: &mut Sunk,
    genre: &str,
    page: search::SearchPage,
    folder_id: Option<usize>,
) -> Result<Vec<Song>>
{
    let args = Query::with("genre", genre.to_string())
        .arg("count", page.count.to_string())
        .arg("offset", page.offset.to_string())
        .maybe_arg("musicFolderId", map_str(folder_id))
        .build();

    let song = sunk.get("getSongsByGenre", args)?;
    Ok(get_list_as!(song, Song))
}

/// Searches for lyrics matching the artist and title. Returns `None` if no
/// lyrics are found.
pub fn get_lyrics(
    sunk: &mut Sunk,
    artist: Option<&str>,
    title: Option<&str>,
) -> Result<Option<Lyrics>> {
    let args = Query::new()
        .maybe_arg("artist", artist)
        .maybe_arg("title", title)
        .build();
    let res = sunk.get("getLyrics", args)?;
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
    use test_util::*;

    #[test]
    fn parse_test() {
        let raw = json!(
            {
                "id": "1633",
                "parent": "1632",
                "isDir": false,
                "title": "That Is How I Roll!",
                "album": "That Is How I Roll!",
                "artist": "Afterglow",
                "track": 1,
                "year": 2017,
                "genre": "J-Pop",
                "coverArt": "1632",
                "size": 32345658,
                "contentType": "audio/flac",
                "suffix": "flac",
                "transcodedContentType": "audio/mpeg",
                "transcodedSuffix": "mp3",
                "duration": 240,
                "bitRate": 1073,
                "path": "A/Afterglow/That Is How I Roll!/01 That Is How I Roll!.flac",
                "isVideo": false,
                "playCount": 16,
                "discNumber": 1,
                "created": "2018-01-01T10:30:04.000Z",
                "albumId": "222",
                "artistId": "138",
                "type": "music"
            }
        );

        let parsed = serde_json::from_value::<Song>(raw);
        assert!(parsed.is_ok());
    }

    #[test]
    fn get_hls() {
        let (s, u, p) = load_credentials().unwrap();
        let mut srv = Sunk::new(&s, &u, &p).unwrap();
        let song = serde_json::from_value::<Song>(json!(
            {
                "id": "1633",
                "parent": "1632",
                "isDir": false,
                "title": "That Is How I Roll!",
                "album": "That Is How I Roll!",
                "artist": "Afterglow",
                "track": 1,
                "year": 2017,
                "genre": "J-Pop",
                "coverArt": "1632",
                "size": 32345658,
                "contentType": "audio/flac",
                "suffix": "flac",
                "transcodedContentType": "audio/mpeg",
                "transcodedSuffix": "mp3",
                "duration": 240,
                "bitRate": 1073,
                "path": "A/Afterglow/That Is How I Roll!/01 That Is How I Roll!.flac",
                "isVideo": false,
                "playCount": 16,
                "discNumber": 1,
                "created": "2018-01-01T10:30:04.000Z",
                "albumId": "222",
                "artistId": "138",
                "type": "music"
            }
        )).unwrap();

        let hls = song.hls(&mut srv, None);
        assert!(hls.is_ok());
    }
}
