//! Video APIs.

use std::result;

use serde::de::{Deserialize, Deserializer};
use serde_json;

use crate::query::Query;
use crate::{Client, Error, Media, Result, Streamable};

#[allow(missing_docs)]
#[derive(Debug)]
#[readonly::make]
pub struct Video {
    pub id: String,
    pub parent: usize,
    pub is_dir: bool,
    pub title: String,
    pub album: Option<String>,
    pub cover_id: Option<String>,
    pub size: usize,
    pub content_type: String,
    pub suffix: String,
    pub transcoded_suffix: Option<String>,
    pub transcoded_content_type: Option<String>,
    pub duration: usize,
    pub bitrate: usize,
    pub path: String,
    pub is_video: bool,
    pub created: String,
    pub play_count: Option<u64>,
    pub media_type: String,
    pub bookmark_position: Option<u64>,
    pub original_height: Option<u64>,
    pub original_width: Option<u64>,
    pub stream_br: Option<usize>,
    pub stream_size: Option<(usize, usize)>,
    pub stream_offset: usize,
    pub stream_tc: Option<String>,
}

impl Video {
    #[allow(missing_docs)]
    pub fn get(client: &Client, id: String) -> Result<Video> {
        Video::list(client)?
            .into_iter()
            .find(|v| v.id == id)
            .ok_or(Error::Other("no video found"))
    }

    #[allow(missing_docs)]
    pub fn list(client: &Client) -> Result<Vec<Video>> {
        let video = client.get("getVideos", Query::none())?;
        Ok(get_list_as!(video, Video))
    }

    #[allow(missing_docs)]
    pub fn info<'a, S>(&self, client: &Client, format: S) -> Result<VideoInfo>
    where
        S: Into<Option<&'a str>>,
    {
        let args = Query::with("id", self.id.clone())
            .arg("format", format.into())
            .build();
        let res = client.get("getVideoInfo", args)?;
        Ok(serde_json::from_value(res)?)
    }

    /// Returns the raw video captions.
    pub fn captions<'a, S>(&self, client: &Client, format: S) -> Result<String>
    where
        S: Into<Option<&'a str>>,
    {
        let args = Query::with("id", self.id.clone())
            .arg("format", format.into())
            .build();
        let res = client.get_raw("getCaptions", args)?;
        Ok(res)
    }

    /// Sets the size that the video will stream at, measured in pixels.
    pub fn set_size(&mut self, width: usize, height: usize) {
        self.stream_size = Some((width, height));
    }

    /// Sets the time (in seconds) that a stream will be offset by.
    ///
    /// For example, to start playback at 1:40, use an offset of 100 seconds.
    ///
    /// Can be used to implement video skipping.
    pub fn set_start_time(&mut self, offset: usize) {
        self.stream_offset = offset;
    }
}

impl Streamable for Video {
    fn stream(&self, client: &Client) -> Result<Vec<u8>> {
        let args = Query::with("id", self.id.clone())
            .arg("maxBitRate", self.stream_br)
            .arg(
                "size",
                self.stream_size.map(|(w, h)| format!("{}x{}", w, h)),
            )
            .arg("timeOffset", self.stream_offset)
            .build();
        client.get_bytes("stream", args)
    }

    fn stream_url(&self, client: &Client) -> Result<String> {
        let args = Query::with("id", self.id.clone())
            .arg("maxBitRate", self.stream_br)
            .arg(
                "size",
                self.stream_size.map(|(w, h)| format!("{}x{}", w, h)),
            )
            .arg("timeOffset", self.stream_offset)
            .build();
        client.build_url("stream", args)
    }

    fn download(&self, client: &Client) -> Result<Vec<u8>> {
        client.get_bytes("download", Query::with("id", self.id.clone()))
    }

    fn download_url(&self, client: &Client) -> Result<String> {
        client.build_url("download", Query::with("id", self.id.clone()))
    }

    fn encoding(&self) -> &str {
        self.transcoded_content_type
            .as_ref()
            .unwrap_or(&self.content_type)
    }

    fn set_max_bit_rate(&mut self, bit_rate: usize) {
        self.stream_br = Some(bit_rate);
    }

    fn set_transcoding(&mut self, format: &str) {
        self.stream_tc = Some(format.to_string());
    }
}

impl Media for Video {
    fn has_cover_art(&self) -> bool {
        self.cover_id.is_some()
    }

    fn cover_id(&self) -> Option<&str> {
        self.cover_id.as_deref()
    }

    fn cover_art<U: Into<Option<usize>>>(&self, client: &Client, size: U) -> Result<Vec<u8>> {
        let cover = self.cover_id().ok_or(Error::Other("no cover art found"))?;
        let query = Query::with("id", cover).arg("size", size.into()).build();

        client.get_bytes("getCoverArt", query)
    }

    fn cover_art_url<U: Into<Option<usize>>>(&self, client: &Client, size: U) -> Result<String> {
        let cover = self.cover_id().ok_or(Error::Other("no cover art found"))?;
        let query = Query::with("id", cover).arg("size", size.into()).build();

        client.build_url("getCoverArt", query)
    }
}

impl<'de> Deserialize<'de> for Video {
    fn deserialize<D>(de: D) -> ::std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _Video {
            id: String,
            parent: String,
            is_dir: bool,
            title: String,
            album: Option<String>,
            cover_art: Option<String>,
            size: usize,
            content_type: String,
            suffix: String,
            transcoded_suffix: Option<String>,
            transcoded_content_type: Option<String>,
            duration: usize,
            bit_rate: usize,
            path: String,
            is_video: bool,
            play_count: Option<u64>,
            created: String,
            #[serde(rename = "type")]
            media_type: String,
            bookmark_position: Option<u64>,
            original_height: Option<u64>,
            original_width: Option<u64>,
        }

        let raw = _Video::deserialize(de)?;

        Ok(Video {
            id: raw.id.parse().unwrap(),
            parent: raw.parent.parse().unwrap(),
            is_dir: raw.is_dir,
            title: raw.title,
            album: raw.album,
            cover_id: raw.cover_art,
            size: raw.size,
            content_type: raw.content_type,
            suffix: raw.suffix,
            transcoded_content_type: raw.transcoded_content_type,
            transcoded_suffix: raw.transcoded_suffix,
            duration: raw.duration,
            bitrate: raw.bit_rate,
            path: raw.path,
            is_video: raw.is_video,
            play_count: raw.play_count,
            created: raw.created,
            media_type: raw.media_type,
            bookmark_position: raw.bookmark_position,
            original_height: raw.original_height,
            original_width: raw.original_width,
            stream_br: None,
            stream_size: None,
            stream_offset: 0,
            stream_tc: None,
        })
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct VideoInfo {
    pub id: usize,
    pub captions: Option<Captions>,
    pub audio_tracks: Vec<AudioTrack>,
    pub conversion: Option<Conversion>,
}

impl<'de> Deserialize<'de> for VideoInfo {
    fn deserialize<D>(de: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct _VideoInfo {
            id: String,
            captions: Option<Captions>,
            #[serde(rename = "audioTrack")]
            #[serde(default)]
            audio_tracks: Vec<AudioTrack>,
            conversion: Option<Conversion>,
        }
        let raw = _VideoInfo::deserialize(de)?;
        Ok(VideoInfo {
            id: raw.id.parse().unwrap(),
            captions: raw.captions,
            audio_tracks: raw.audio_tracks,
            conversion: raw.conversion,
        })
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct AudioTrack {
    pub id: usize,
    pub name: String,
    pub language_code: String,
}

impl<'de> Deserialize<'de> for AudioTrack {
    fn deserialize<D>(de: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct _AudioTrack {
            id: String,
            name: String,
            #[serde(rename = "languageCode")]
            language_code: String,
        }
        let raw = _AudioTrack::deserialize(de)?;
        Ok(AudioTrack {
            id: raw.id.parse().unwrap(),
            name: raw.name,
            language_code: raw.language_code,
        })
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Captions {
    pub id: usize,
    pub name: String,
}

impl<'de> Deserialize<'de> for Captions {
    fn deserialize<D>(de: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct _Captions {
            id: String,
            name: String,
        }
        let raw = _Captions::deserialize(de)?;
        Ok(Captions {
            id: raw.id.parse().unwrap(),
            name: raw.name,
        })
    }
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Conversion {
    pub id: usize,
    pub bitrate: usize,
}

impl<'de> Deserialize<'de> for Conversion {
    fn deserialize<D>(de: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct _Conversion {
            id: String,
            #[serde(rename = "bitRate")]
            bitrate: String,
        }
        let raw = _Conversion::deserialize(de)?;
        Ok(Conversion {
            id: raw.id.parse().unwrap(),
            bitrate: raw.bitrate.parse().unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_video() {
        let parsed = serde_json::from_value::<Video>(raw()).unwrap();

        assert_eq!(parsed.id, "460");
        assert_eq!(parsed.title, "Big Buck Bunny");
        assert!(!parsed.has_cover_art());
    }

    #[test]
    fn parse_video_info() {
        let parsed = serde_json::from_value::<VideoInfo>(raw_info()).unwrap();

        assert_eq!(parsed.id, 7058);
        assert_eq!(parsed.audio_tracks.len(), 5);
    }

    fn raw() -> serde_json::Value {
        serde_json::from_str(
            r#"{
            "id" : "460",
            "parent" : "24",
            "isDir" : false,
            "title" : "Big Buck Bunny",
            "album" : "Movies",
            "size" : 52464391,
            "contentType" : "video/mp4",
            "suffix" : "mp4",
            "duration" : 281,
            "bitRate" : 1488,
            "path" : "Movies/Big Buck Bunny.mp4",
            "isVideo" : true,
            "playCount" : 4035,
            "created" : "2017-03-12T11:06:30.000Z",
            "type" : "video",
            "bookmarkPosition" : 80000,
            "originalWidth" : 1280,
            "originalHeight" : 720
         }"#,
        )
        .unwrap()
    }

    fn raw_info() -> serde_json::Value {
        serde_json::from_str(
            r#"{
            "id": "7058",
            "captions": {
                "id": "0",
                "name": "Planes 2.srt"
            },
            "audioTrack": [
                {
                    "id": "1",
                    "name": "English",
                    "languageCode": "eng"
                },
                {
                    "id": "3",
                    "name": "Danish",
                    "languageCode": "dan"
                },
                {
                    "id": "4",
                    "name": "Finnish",
                    "languageCode": "fin"
                },
                {
                    "id": "5",
                    "name": "Norwegian",
                    "languageCode": "nor"
                },
                {
                    "id": "6",
                    "name": "Swedish",
                    "languageCode": "swe"
                }
            ],
            "conversion": {
                "id": "37",
                "bitRate": "1000"
            }
        }"#,
        )
        .unwrap()
    }
}
