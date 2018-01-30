use serde_json;
use serde::de::{Deserialize, Deserializer};
use std::result;

use client::Client;
use error::Result;
use media::song::Song;
use query::Query;

#[derive(Debug)]
pub struct Jukebox<'a> {
    client: &'a mut Client,
}

#[derive(Debug, Deserialize)]
pub struct JukeboxStatus {
    #[serde(rename = "currentIndex")]
    pub index: isize,
    pub playing: bool,
    #[serde(rename = "gain")]
    pub volume: f32,
    pub position: usize,
}

#[derive(Debug)]
pub struct JukeboxPlaylist {
    pub status: JukeboxStatus,
    pub songs: Vec<Song>,
}

impl<'de> Deserialize<'de> for JukeboxPlaylist {
    fn deserialize<D>(de: D) -> result::Result<Self, D::Error> where
        D: Deserializer<'de>
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
    pub fn start(client: &'a mut Client) -> Jukebox {
        Jukebox { client }
    }

    fn send_action_with<T, U>(
        &mut self,
        action: &str,
        index: U,
        offset: U,
        ids: &[usize],
        gain: T
    ) -> Result<JukeboxStatus>
    where
        T: Into<Option<f32>>,
        U: Into<Option<usize>>,
    {
        let args = Query::with("action", action)
            .arg("index", index.into())
            .arg("offset", offset.into())
            .arg_list("id", ids.to_vec())
            .arg("gain", gain.into())
            .build();
        let res = self.client.get("jukeboxControl", args)?;
        Ok(serde_json::from_value(res)?)
    }

    fn send_action(&mut self, action: &str) -> Result<JukeboxStatus> {
        self.send_action_with(action, None, None, &[], None)
    }

    pub fn playlist(&mut self) -> Result<JukeboxPlaylist> {
        let res = self.client.get("jukeboxControl", Query::with("action", "get"))?;
        Ok(serde_json::from_value::<JukeboxPlaylist>(res)?)
    }

    pub fn status(&mut self) -> Result<JukeboxStatus> {
        self.send_action("status")
    }

    pub fn play(&mut self) -> Result<JukeboxStatus> {
        self.send_action("start")
    }

    pub fn stop(&mut self) -> Result<JukeboxStatus> {
        self.send_action("stop")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_playlist() {
        let parsed = serde_json::from_str::<JukeboxPlaylist>(r#"{
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
         }"#).unwrap();

        assert_eq!(parsed.songs.len(), 2);
        assert!(!parsed.status.playing);
        assert_eq!(parsed.status.volume, 0.75);
    }
}
