use serde::de::{Deserialize, Deserializer};
use std::result;

use client::Client;
use error::{Error, Result};
use query::{Arg, IntoArg};

pub mod format;
pub mod podcast;
pub mod song;
pub mod video;
pub mod radio;

pub use self::song::Song;
pub use self::video::Video;

use self::format::{AudioFormat, VideoFormat};

/// A trait for forms of streamable media.
pub trait Streamable {
    /// Returns the raw bytes of the media.
    ///
    /// The method does not provide any information about the encoding of the
    /// media without evaluating the stream itself.
    fn stream(&self, client: &mut Client) -> Result<Vec<u8>>;

    /// Returns a constructed URL for streaming with desired arguments.
    ///
    /// This would be used in conjunction with a streaming library to directly
    /// take the URI and stream it.
    fn stream_url(&self, client: &mut Client) -> Result<String>;

    fn download(&self, client: &mut Client) -> Result<Vec<u8>>;

    /// Returns a constructed URL for downloading the song.
    ///
    /// `download_url()` does not support transcoding, while `stream_url()`
    /// does.
    fn download_url(&self, client: &mut Client) -> Result<String>;

    /// Returns the default encoding of the media.
    ///
    /// A Subsonic server is able to transcode media for streaming to reduce
    /// data size (for example, it may transcode FLAC to MP3 to reduce file
    /// size, or downsample high bitrate files). Where possible, the method will
    /// return the default transcoding of the media (if enabled); otherwise, it
    /// will return the original encoding.
    fn encoding(&self) -> &str;

    /// Sets the maximum bitrate the media will use when streaming.
    ///
    /// The bit rate is measured in Kbps. Higher bit rate media will be
    /// downsampled to this bit rate.
    ///
    /// Supported values are 0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192,
    /// 224, 256, 320. The method will not error or panic when using a non-legal
    /// value, but the server may not provide the requested bit rate. A bit rate
    /// of 0 will disable a limit (i.e., use the original bit rate.)
    fn set_max_bit_rate(&mut self, bit_rate: usize);

    /// Sets the transcoding format the media will use when streaming.
    ///
    /// The possible transcoding formats are those defined by the Subsonic
    /// server as valid transcoding targets. Default values are `"mp3"`,
    /// `"flv"`, `"mkv"`, and `"mp4"`. The special value `"raw"` is additionally
    /// supported on servers implementing version 1.9.0 of the API.
    ///
    /// The method will not error or panic when using a non-supported format,
    /// but the server may not provide that transcoded format.
    fn set_transcoding(&mut self, format: &str);
}

/// A trait deriving common methods for any form of media.
pub trait Media {
    /// Returns whether or not the media has an associated cover.
    fn has_cover_art(&self) -> bool;

    /// Returns the cover ID associated with the media, if any.
    ///
    /// The ID may be a number, an identifier-number pair, or simply empty.
    /// This is due to the introduction of ID3 tags into the Subsonic API;
    /// collections of media (such as albums or playlists) will typically
    /// have an identifier-number ID, while raw media (such as songs or videos)
    /// will have a numeric or no identifier.
    ///
    /// Because the method has the potential to return either a string-y or
    /// numeric ID, the number is coerced into a `&str` to avoid type
    /// checking workarounds.
    fn cover_id(&self) -> Option<&str>;

    /// Returns the raw bytes of the cover art of the media.
    ///
    /// The image is guaranteed to be valid and displayable by the Subsonic
    /// server (as long as the method does not error), but makes no guarantees
    /// on the encoding of the image.
    ///
    /// # Errors
    ///
    /// Aside from errors that the [`Client`] may cause, the method will error
    /// if the media does not have an associated cover art.
    ///
    /// [`Client`]: ../client/struct.Client.html
    fn cover_art<U: Into<Option<usize>>>(
        &self,
        client: &mut Client,
        size: U,
    ) -> Result<Vec<u8>>;

    /// Returns the URL pointing to the cover art of the media.
    ///
    /// # Errors
    ///
    /// Aside from errors that the [`Client`] may cause, the method will error
    /// if the media does not have an associated cover art.
    ///
    /// [`Client`]: ../client/struct.Client.html
    fn cover_art_url<U: Into<Option<usize>>>(
        &self,
        client: &mut Client,
        size: U,
    ) -> Result<String>;
}

/// Information about currently playing media.
///
/// Due to the "now playing" information possibly containing both audio and
/// video, compromises are made. `NowPlaying` only stores the ID, title, and
/// content type of the media. This is most of the information afforded through
/// the web interface. For more detailed information, `NowPlaying::info()` gives
/// the full `Song` or `Video` struct, though requires another web reqeust.
#[derive(Debug)]
pub struct NowPlaying {
    user: String,
    minutes_ago: usize,
    player_id: usize,
    id: usize,
    is_video: bool,
}

impl NowPlaying {
    /// Fetches information about the currently playing song.
    ///
    /// # Errors
    ///
    /// Aside from the inherent errors from the [`Client`], the method will
    /// error if the `NowPlaying` is not a song.
    ///
    /// [`Client`]: ../client/struct.Client.html
    pub fn song_info(&self, client: &mut Client) -> Result<Song> {
        if self.is_video {
            Err(Error::Other("Now Playing info is not a song"))
        } else {
            song::get_song(client, self.id as u64)
        }
    }

    /// Fetches information about the currently playing video.
    ///
    /// # Errors
    ///
    /// Aside from the inherent errors from the [`Client`], the method will
    /// error if the `NowPlaying` is not a video.
    ///
    /// [`Client`]: ../client/struct.Client.html
    pub fn video_info(&self, client: &mut Client) -> Result<Video> {
        if !self.is_video {
            Err(Error::Other("Now Playing info is not a video"))
        } else {
            Video::get(client, self.id)
        }
    }

    /// Returns `true` if the currently playing media is a song.
    pub fn is_song(&self) -> bool { !self.is_video }

    /// Returns `true` if the currently playing media is a video.
    pub fn is_video(&self) -> bool { self.is_video }
}

impl<'de> Deserialize<'de> for NowPlaying {
    fn deserialize<D>(de: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _NowPlaying {
            username: String,
            minutes_ago: usize,
            player_id: usize,
            id: String,
            is_dir: bool,
            title: String,
            size: usize,
            content_type: String,
            suffix: String,
            transcoded_content_type: Option<String>,
            transcoded_suffix: Option<String>,
            path: String,
            is_video: bool,
            created: String,
            #[serde(rename = "type")]
            media_type: String,
        }

        let raw = _NowPlaying::deserialize(de)?;

        Ok(NowPlaying {
            user: raw.username,
            minutes_ago: raw.minutes_ago,
            player_id: raw.player_id,
            id: raw.id.parse().unwrap(),
            is_video: raw.is_video,
        })
    }
}
