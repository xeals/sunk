use sunk::Sunk;
use std::convert::From;
use json;
use error::*;

use macros::*;

/// Audio encoding format.
///
/// Recognises all of Subsonic's default transcoding formats.
#[derive(Debug)]
pub enum AudioFormat {
    Aac, Aif, Aiff, Ape, Flac, Flv, M4a, Mp3,
    Mpc, Oga, Ogg, Ogx, Opus, Shn, Wav, Wma,
    Raw
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
    title: Option<String>,
    album: Option<String>,
    album_id: Option<u64>,
    artist: Option<String>,
    artist_id: Option<u64>,
    track: Option<u64>,
    year: Option<u64>,
    genre: Option<String>,
    cover_id: Option<u64>,
    size: u64,
    duration: u64,
    path: String,
}

impl Song {
    pub fn from(j: &json::Value) -> Result<Song> {
        if !j.is_object() { return Err(Error::ParseError("not an object")) }

        Ok(Song {
            id: fetch!(j->id: as_str, u64),
            title: fetch_maybe!(j->title: as_str).map(|v| v.into()),
            album: fetch_maybe!(j->album: as_str).map(|v| v.into()),
            album_id: fetch_maybe!(j->albumId: as_str, u64),
            artist: fetch_maybe!(j->artist: as_str).map(|v| v.into()),
            artist_id: fetch_maybe!(j->artistId: as_str, u64),
            track: fetch_maybe!(j->track: as_u64),
            year: fetch_maybe!(j->year: as_u64),
            genre: fetch_maybe!(j->genre: as_str).map(|v| v.into()),
            cover_id: fetch_maybe!(j->coverArt: as_str, u64),
            size: fetch!(j->size: as_u64),
            duration: fetch!(j->duration: as_u64),
            path: fetch!(j->path: as_str).into(),
        })
    }

    /// Returns a constructed URL for streaming with desired arguments.
    ///
    /// This would be used in conjunction with a streaming library to directly
    /// take the URI and stream it.
    pub fn stream_url(
        &self,
        sunk: &Sunk,
        bitrate: Option<u64>,
        format: Option<AudioFormat>
    ) -> Result<String> {
        let mut args = vec![("id", self.id.to_string())];
        push_if_some!(args, "maxBitRate", bitrate);
        push_if_some!(args, "format", format);
        ::sunk::build_url(sunk, "stream", args)
    }

    /// Returns a constructed URL for downloading the song.
    ///
    /// `download_url()` does not support transcoding, while `stream_url()`
    /// does.
    pub fn download_url(&self, sunk: &Sunk) -> Result<String> {
        self.stream_url(sunk, None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
