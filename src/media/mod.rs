use sunk::Sunk;

pub mod song;
pub mod format;

// use format::{AudioFormat, VideoFormat};
use self::format::AudioFormat;
use error::{Error, Result};
use query::{Arg, IntoArg};
use self::song::Song;

pub trait Media {
    fn stream<A: StreamArgs>(&self, &mut Sunk, A) -> Result<Vec<u8>>;

    /// Returns a constructed URL for streaming with desired arguments.
    ///
    /// This would be used in conjunction with a streaming library to directly
    /// take the URI and stream it.
    fn stream_url<A: StreamArgs>(&self, &mut Sunk, A) -> Result<String>;

    fn download(&self, &mut Sunk) -> Result<Vec<u8>>;

    /// Returns a constructed URL for downloading the song.
    ///
    /// `download_url()` does not support transcoding, while `stream_url()`
    /// does.
    fn download_url(&self, &mut Sunk) -> Result<String>;
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
    /// Aside from the inherent errors from the [`Sunk`], the method will error
    /// if the `NowPlaying` is not a song.
    ///
    /// [`Sunk`]: ../struct.sunk.html
    pub fn song_info<M>(&self, sunk: &mut Sunk) -> Result<Song>
    {
        if self.is_video {
            Err(Error::Other("Now Playing info is not a song"))
        } else {
            song::get_song(sunk, self.id as u64)
        }
    }
}
