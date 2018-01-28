use client::Client;
use error::Result;
use query::Query;
use serde::de::{Deserialize, Deserializer};
use serde_json;
use std::result;

#[derive(Debug)]
pub struct Podcast {
    id: usize,
    url: String,
    title: String,
    description: String,
    cover_art: String,
    image_url: String,
    status: String,
    episodes: Vec<Episode>,
    error: Option<String>,
}

#[derive(Debug)]
pub struct Episode {
    id: usize,
    parent: usize,
    is_dir: bool,
    title: String,
    album: String,
    artist: String,
    year: usize,
    cover_art: String,
    size: usize,
    content_type: String,
    suffix: String,
    duration: usize,
    bitrate: usize,
    is_video: bool,
    created: String,
    artist_id: String,
    media_type: String,
    stream_id: String,
    channel_id: String,
    description: String,
    status: String,
    publish_date: String,
}

impl Podcast {
    /// Fetches the details of a single podcast and its episodes.
    pub fn get<U>(client: &mut Client, id: U) -> Result<Podcast>
    where
        U: Into<Option<usize>>,
    {
        let mut channel = client.get("getPodcasts", Query::with("id", id.into()))?;
        Ok(get_list_as!(channel, Podcast).remove(0))
    }
    /// Returns a list of all podcasts the server subscribes to and,
    /// optionally, their episodes.
    pub fn list<B, U>(
        client: &mut Client,
        include_episodes: B,
    ) -> Result<Vec<Podcast>>
    where
        B: Into<Option<bool>>,
        U: Into<Option<usize>>,
    {
        let channel = client.get(
            "getPodcasts",
            Query::with("includeEpisodes", include_episodes.into()),
        )?;
        Ok(get_list_as!(channel, Podcast))
    }
}

impl Episode {
    /// Returns a list of the newest episodes of podcasts the server subscribes
    /// to. Optionally takes a number of episodes to maximally return.
    pub fn newest<U>(client: &mut Client, count: U) -> Result<Vec<Episode>>
    where
        U: Into<Option<usize>>,
    {
        let episode = client
            .get("getNewestPodcasts", Query::with("count", count.into()))?;
        Ok(get_list_as!(episode, Episode))
    }
}

impl<'de> Deserialize<'de> for Podcast {
    fn deserialize<D>(de: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _Podcast {
            id: String,
            url: String,
            title: String,
            description: String,
            cover_art: String,
            image_url: String,
            status: String,
            #[serde(default)]
            episode: Vec<Episode>,
            #[serde(default)]
            error_message: String,
        }

        let raw = _Podcast::deserialize(de)?;

        Ok(Podcast {
            id: raw.id.parse().unwrap(),
            url: raw.url,
            title: raw.title,
            description: raw.description,
            cover_art: raw.cover_art,
            image_url: raw.image_url,
            status: raw.status,
            episodes: raw.episode,
            error: if raw.error_message.is_empty() {
                None
            } else {
                Some(raw.error_message)
            },
        })
    }
}

impl<'de> Deserialize<'de> for Episode {
    fn deserialize<D>(de: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _Episode {
            id: String,
            parent: String,
            is_dir: bool,
            title: String,
            album: String,
            artist: String,
            year: usize,
            cover_art: String,
            size: usize,
            content_type: String,
            suffix: String,
            duration: usize,
            bit_rate: usize,
            is_video: bool,
            created: String,
            artist_id: String,
            #[serde(rename = "type")]
            _type: String,
            stream_id: String,
            channel_id: String,
            description: String,
            status: String,
            publish_date: String,
        }

        let raw = _Episode::deserialize(de)?;

        Ok(Episode {
            id: raw.id.parse().unwrap(),
            parent: raw.parent.parse().unwrap(),
            is_dir: raw.is_dir,
            title: raw.title,
            album: raw.album,
            artist: raw.artist,
            year: raw.year,
            cover_art: raw.cover_art,
            size: raw.size,
            content_type: raw.content_type,
            suffix: raw.suffix,
            duration: raw.duration,
            bitrate: raw.bit_rate,
            is_video: raw.is_video,
            created: raw.created,
            artist_id: raw.artist_id,
            media_type: raw._type,
            stream_id: raw.stream_id,
            channel_id: raw.channel_id,
            description: raw.description,
            status: raw.status,
            publish_date: raw.publish_date,
        })
    }
}
