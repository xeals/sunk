use error::*;
use json;
use sunk::Sunk;

use util::*;
use query::Query;

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

#[derive(Debug)]
pub struct Song {
    id: u64,
    // parent: u64,
    title:     Option<String>,
    album:     Option<String>,
    album_id:  Option<u64>,
    artist:    Option<String>,
    artist_id: Option<u64>,
    track:     Option<u64>,
    year:      Option<u64>,
    genre:     Option<String>,
    cover_id:  Option<u64>,
    size:      u64,
    duration:  u64,
    path:      String,
}

impl Song {
    pub fn from(j: &json::Value) -> Result<Song> {
        if !j.is_object() {
            return Err(Error::ParseError("not an object"))
        }

        Ok(Song {
            id:        fetch!(j->id: as_str, u64),
            title:     fetch_maybe!(j->title: as_str).map(|v| v.into()),
            album:     fetch_maybe!(j->album: as_str).map(|v| v.into()),
            album_id:  fetch_maybe!(j->albumId: as_str, u64),
            artist:    fetch_maybe!(j->artist: as_str).map(|v| v.into()),
            artist_id: fetch_maybe!(j->artistId: as_str, u64),
            track:     fetch_maybe!(j->track: as_u64),
            year:      fetch_maybe!(j->year: as_u64),
            genre:     fetch_maybe!(j->genre: as_str).map(|v| v.into()),
            cover_id:  fetch_maybe!(j->coverArt: as_str, u64),
            size:      fetch!(j->size: as_u64),
            duration:  fetch!(j->duration: as_u64),
            path:      fetch!(j->path: as_str).into(),
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
    pub fn hls(&self, sunk: &mut Sunk, bitrates: Option<Vec<u64>>) -> Result<String> {
        let args = Query::new()
            .arg("id", self.id)
            .maybe_arg_list("bitrate", bitrates)
            .build();

        let raw = sunk.get_raw("hls", args)?;
        {
            let fline = raw.split('\n').next()
                .ok_or(Error::StreamError("unexpected EOF"))?;
            if fline.contains("xml") || fline.contains('{') {
                return Err(Error::Api(SubsonicError::from_u16(70)?))
            }
        }
        Ok(raw)
    }

    /// Returns the URL of the cover art. Size is a single parameter and the
    /// image will be scaled on its longest edge.
    impl_cover_art!();
}

/// Searches for lyrics matching the artist and title. Returns an empty string
/// if no lyrics are found.
pub fn get_lyrics(sunk: &mut Sunk, artist: Option<&str>, title: Option<&str>) -> Result<String> {
    let args = Query::new()
        .maybe_arg("artist", artist)
        .maybe_arg("title", title)
        .build();
    let res = sunk.get("getLyrics", args)?;
    pointer!(res, "/subsonic-response/lyrics").as_str()
        .ok_or(Error::ParseError("not a string")).map(|s| s.to_string())
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

        let parsed = Song::from(&raw);
        assert!(parsed.is_ok());
    }

    #[test]
    fn get_hls() {
        let (s, u, p) = load_credentials().unwrap();
        let mut srv = Sunk::new(&s, &u, &p).unwrap();
        let song = Song::from(&json!(
            {
                "id": "1633",
                "duration": 240,
                "size": 1073,
                "path": "A/Afterglow/That Is How I Roll!/01 That Is How I Roll!.flac"
            }
        )).unwrap();

        let hls = song.hls(&mut srv, None);
        assert!(hls.is_ok());
    }
}
