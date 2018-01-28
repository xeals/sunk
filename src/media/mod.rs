use serde::de::{Deserialize, Deserializer};
use std::result;

use client::Client;

pub mod song;
pub mod format;

// use format::{AudioFormat, VideoFormat};
use self::format::AudioFormat;
use error::{Error, Result};
use query::{Arg, IntoArg};
use self::song::Song;

pub trait Media {
    fn stream<A: StreamArgs>(&self, &mut Client, A) -> Result<Vec<u8>>;

    /// Returns a constructed URL for streaming with desired arguments.
    ///
    /// This would be used in conjunction with a streaming library to directly
    /// take the URI and stream it.
    fn stream_url<A: StreamArgs>(&self, &mut Client, A) -> Result<String>;

    fn download(&self, &mut Client) -> Result<Vec<u8>>;

    /// Returns a constructed URL for downloading the song.
    ///
    /// `download_url()` does not support transcoding, while `stream_url()`
    /// does.
    fn download_url(&self, &mut Client) -> Result<String>;
}

pub trait StreamArgs {
    fn into_arg_set(self) -> Vec<(String, Arg)>;
}

#[derive(Debug)]
pub struct MusicStreamArgs {
    max_bit_rate: Option<usize>,
    format: Option<AudioFormat>,
    estimate_content_length: Option<bool>,
}

impl MusicStreamArgs {
    pub fn new<B, F, U>(
        max_bit_rate: U,
        format: F,
        estimate_content_length: B,
    ) -> MusicStreamArgs
    where
        B: Into<Option<bool>>,
        F: Into<Option<AudioFormat>>,
        U: Into<Option<usize>>,
    {
        MusicStreamArgs {
            max_bit_rate: max_bit_rate.into(),
            format: format.into(),
            estimate_content_length: estimate_content_length.into(),
        }
    }
}

impl StreamArgs for MusicStreamArgs {
    fn into_arg_set(self) -> Vec<(String, Arg)> {
        vec![
            ("maxBitRate".to_string(), self.max_bit_rate.into_arg()),
            ("format".to_string(), self.format.into_arg()),
            ("estimateContentLength".to_string(), self.estimate_content_length.into_arg())
        ]
    }
}

// #[derive(Debug)]
// pub struct VideoStreamArgs {
//     max_bit_rate: usize,
//     format: VideoFormat,
//     time_offset: usize,
//     size: (usize, usize),
//     estimate_content_length: bool,
//     converted: bool,
// }

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
    /// # Errors
    ///
    /// Aside from the inherent errors from the [`Client`], the method will error
    /// if the `NowPlaying` is not a song.
    ///
    /// [`Client`]: ../client/struct.Client.html
    pub fn song_info<M>(&self, client: &mut Client) -> Result<Song>
    {
        if self.is_video {
            Err(Error::Other("Now Playing info is not a song"))
        } else {
            song::get_song(client, self.id as u64)
        }
    }
}

impl<'de> Deserialize<'de> for NowPlaying {
    fn deserialize<D>(de: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>
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
            media_type: String
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
