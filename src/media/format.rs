use query::{Arg, IntoArg};
use std::fmt;

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

impl fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

impl IntoArg for AudioFormat {
    fn into_arg(self) -> Arg { self.to_string().into_arg() }
}

#[derive(Debug)]
pub enum VideoFormat {
    Avi,
    Mpg,
    Mpeg,
    Mp4,
    M4v,
    Mkv,
    Mov,
    Wmv,
    Ogv,
    Divx,
    M2ts,
}

impl fmt::Display for VideoFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

impl IntoArg for VideoFormat {
    fn into_arg(self) -> Arg { self.to_string().into_arg() }
}
