//! Jukebox management and control APIs.

use std::result;

use serde::de::{Deserialize, Deserializer};
use serde_json;

use crate::query::Query;
use crate::{Client, Result, Song};

/// A wrapper on a `Client` to control just the jukebox.
///
/// Any method invoked on a `Jukebox` will fail if the user that created the
/// `Client` is not authorized to control the jukebox.
#[derive(Debug)]
pub struct Jukebox<'a> {
    client: &'a Client,
}

/// A representation of the jukebox's current status.
#[derive(Debug, Deserialize)]
pub struct JukeboxStatus {
    /// Current index in the playlist (zero-indexed). `-1` means that the
    /// jukebox has had its playlist cleared and has not since been played.
    #[serde(rename = "currentIndex")]
    pub index: isize,
    /// Whether or not the jukebox is currently active.
    pub playing: bool,
    /// Volume level of the jukebox, from `0` to `1.0`.
    #[serde(rename = "gain")]
    pub volume: f32,
    #[allow(missing_docs)]
    pub position: usize,
}

/// A more detailed representation of the jukebox's status. Includes its
/// current playlist.
#[derive(Debug)]
pub struct JukeboxPlaylist {
    /// The jukebox's status.
    pub status: JukeboxStatus,
    /// The jukebox's current playlist.
    pub songs: Vec<Song>,
}

impl<'de> Deserialize<'de> for JukeboxPlaylist {
    fn deserialize<D>(de: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct _Playlist {
            #[serde(rename = "currentIndex")]
            index: isize,
            playing: bool,
            gain: f32,
            position: usize,
            entry: Vec<Song>,
        }
        let raw = _Playlist::deserialize(de)?;
        Ok(JukeboxPlaylist {
            status: JukeboxStatus {
                index: raw.index,
                playing: raw.playing,
                volume: raw.gain,
                position: raw.position,
            },
            songs: raw.entry,
        })
    }
}

impl<'a> Jukebox<'a> {
    /// Creates a new handler to the jukebox of the client.
    pub fn start(client: &'a Client) -> Jukebox {
        Jukebox { client }
    }

    fn send_action_with<U>(&self, action: &str, index: U, ids: &[String]) -> Result<JukeboxStatus>
    where
        U: Into<Option<usize>>,
    {
        let args = Query::with("action", action)
            .arg("index", index.into())
            .arg_list("id", ids)
            .build();
        let res = self.client.get("jukeboxControl", args)?;
        Ok(serde_json::from_value(res)?)
    }

    fn send_action(&self, action: &str) -> Result<JukeboxStatus> {
        self.send_action_with(action, None, &[])
    }

    /// Returns the current playlist of the jukebox, as well as its status. The
    /// status is also returned as it contains the position of the jukebox
    /// in its playlist.
    pub fn playlist(&self) -> Result<JukeboxPlaylist> {
        let res = self
            .client
            .get("jukeboxControl", Query::with("action", "get"))?;
        Ok(serde_json::from_value::<JukeboxPlaylist>(res)?)
    }

    /// Returns the status of the jukebox.
    pub fn status(&self) -> Result<JukeboxStatus> {
        self.send_action("status")
    }

    /// Tells the jukebox to start playing.
    pub fn play(&self) -> Result<JukeboxStatus> {
        self.send_action("start")
    }

    /// Tells the jukebox to pause playback.
    pub fn stop(&self) -> Result<JukeboxStatus> {
        self.send_action("stop")
    }

    /// Moves the jukebox's currently playing song to the provided index
    /// (zero-indexed).
    ///
    /// Using an index outside the range of the jukebox playlist will play the
    /// last song in the playlist.
    pub fn skip_to(&self, n: usize) -> Result<JukeboxStatus> {
        self.send_action_with("skip", n, &[])
    }

    /// Adds the song to the jukebox's playlist.
    pub fn add(&self, song: &Song) -> Result<JukeboxStatus> {
        self.send_action_with("add", None, &[song.id.clone()])
    }

    /// Adds a song matching the provided ID to the playlist.
    ///
    /// # Errors
    ///
    /// The method will return an error if a song matching the provided ID
    /// cannot be found.
    pub fn add_id(&self, id: String) -> Result<JukeboxStatus> {
        self.send_action_with("add", None, &[id.clone()])
    }

    /// Adds all the songs to the jukebox's playlist.
    pub fn add_all(&self, songs: &[Song]) -> Result<JukeboxStatus> {
        self.send_action_with(
            "add",
            None,
            &songs.iter().map(|s| s.id.clone()).collect::<Vec<_>>(),
        )
    }

    /// Adds multiple songs matching the provided IDs to the playlist.
    ///
    /// # Errors
    ///
    /// The method will return an error if at least one ID cannot be matched to
    /// a song.
    pub fn add_all_ids(&self, ids: &[String]) -> Result<JukeboxStatus> {
        self.send_action_with("add", None, ids)
    }

    /// Clears the jukebox's playlist.
    pub fn clear(&self) -> Result<JukeboxStatus> {
        self.send_action("clear")
    }

    /// Removes the song at the provided index from the playlist.
    pub fn remove_id(&self, idx: usize) -> Result<JukeboxStatus> {
        self.send_action_with("remove", idx, &[])
    }

    /// Shuffles the jukebox's playlist.
    pub fn shuffle(&self) -> Result<JukeboxStatus> {
        self.send_action("shuffle")
    }

    /// Sets the jukebox's playback volume.
    ///
    /// Seting the volume above `1.0` will have no effect.
    pub fn set_volume(&self, volume: f32) -> Result<JukeboxStatus> {
        let args = Query::with("action", "setGain").arg("gain", volume).build();
        let res = self.client.get("jukeboxControl", args)?;
        Ok(serde_json::from_value(res)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_playlist() {
        let parsed = serde_json::from_str::<JukeboxPlaylist>(
            r#"{
            "currentIndex" : 0,
            "playing" : false,
            "gain" : 0.75,
            "position" : 0,
            "entry" : [ {
                "id" : "1887",
                "parent" : "1880",
                "isDir" : false,
                "title" : "トリコリコPLEASE!!",
                "album" : "トリコリコPLEASE!!",
                "artist" : "AZALEA",
                "track" : 1,
                "year" : 2016,
                "coverArt" : "1880",
                "size" : 33457239,
                "contentType" : "audio/flac",
                "suffix" : "flac",
                "transcodedContentType" : "audio/ogg",
                "transcodedSuffix" : "ogg",
                "duration" : 227,
                "bitRate" : 1090,
                "path" : "A/AZALEA/トリコリコPLEASE!!/01 トリコリコPLEASE!!.flac",
                "isVideo" : false,
                "playCount" : 34,
                "discNumber" : 1,
                "created" : "2018-01-01T10:30:10.000Z",
                "albumId" : "260",
                "artistId" : "147",
                "type" : "music"
            }, {
                "id" : "1888",
                "parent" : "1880",
                "isDir" : false,
                "title" : "ときめき分類学",
                "album" : "トリコリコPLEASE!!",
                "artist" : "AZALEA",
                "track" : 2,
                "year" : 2016,
                "coverArt" : "1880",
                "size" : 40146569,
                "contentType" : "audio/flac",
                "suffix" : "flac",
                "transcodedContentType" : "audio/ogg",
                "transcodedSuffix" : "ogg",
                "duration" : 291,
                "bitRate" : 1033,
                "path" : "A/AZALEA/トリコリコPLEASE!!/02 ときめき分類学.flac",
                "isVideo" : false,
                "playCount" : 14,
                "discNumber" : 1,
                "created" : "2018-01-01T10:30:10.000Z",
                "albumId" : "260",
                "artistId" : "147",
                "type" : "music"
            } ]
         }"#,
        )
        .unwrap();

        assert_eq!(parsed.songs.len(), 2);
        assert!(!parsed.status.playing);
        assert_eq!(parsed.status.volume, 0.75);
    }
}
