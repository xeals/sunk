use serde::de::{Deserialize, Deserializer};
use serde_json;
use std::result;

use query::Query;
use {Client, Error, Media, Result, Song};

#[derive(Debug)]
pub struct Playlist {
    id: u64,
    name: String,
    duration: u64,
    cover_id: String,
    song_count: u64,
    songs: Vec<Song>,
}

impl Playlist {
    /// Fetches the songs contained in a playlist.
    pub fn songs(&self, client: &Client) -> Result<Vec<Song>> {
        if self.songs.len() as u64 != self.song_count {
            Ok(get_playlist(client, self.id)?.songs)
        } else {
            Ok(self.songs.clone())
        }
    }
}

impl<'de> Deserialize<'de> for Playlist {
    fn deserialize<D>(de: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _Playlist {
            id: String,
            name: String,
            #[serde(default)]
            comment: String,
            owner: String,
            song_count: u64,
            duration: u64,
            created: String,
            changed: String,
            cover_art: String,
            #[serde(default)]
            songs: Vec<Song>,
        }

        let raw = _Playlist::deserialize(de)?;

        Ok(Playlist {
            id: raw.id.parse().unwrap(),
            name: raw.name,
            duration: raw.duration,
            cover_id: raw.cover_art,
            song_count: raw.song_count,
            songs: raw.songs,
        })
    }
}

impl Media for Playlist {
    fn has_cover_art(&self) -> bool {
        !self.cover_id.is_empty()
    }

    fn cover_id(&self) -> Option<&str> {
        Some(self.cover_id.as_ref())
    }

    fn cover_art<U: Into<Option<usize>>>(&self, client: &Client, size: U) -> Result<Vec<u8>> {
        let cover = self
            .cover_id()
            .ok_or_else(|| Error::Other("no cover art found"))?;
        let query = Query::with("id", cover).arg("size", size.into()).build();

        client.get_bytes("getCoverArt", query)
    }

    fn cover_art_url<U: Into<Option<usize>>>(&self, client: &Client, size: U) -> Result<String> {
        let cover = self
            .cover_id()
            .ok_or_else(|| Error::Other("no cover art found"))?;
        let query = Query::with("id", cover).arg("size", size.into()).build();

        client.build_url("getCoverArt", query)
    }
}

fn get_playlists(client: &Client, user: Option<String>) -> Result<Vec<Playlist>> {
    let playlist = client.get("getPlaylists", Query::with("username", user))?;
    Ok(get_list_as!(playlist, Playlist))
}

fn get_playlist(client: &Client, id: u64) -> Result<Playlist> {
    let res = client.get("getPlaylist", Query::with("id", id))?;
    Ok(serde_json::from_value::<Playlist>(res)?)
}

/// Creates a playlist with the given name.
///
/// Since API version 1.14.0, the newly created playlist is returned. In earlier
/// versions, an empty response is returned.
fn create_playlist(client: &Client, name: String, songs: &[u64]) -> Result<Option<Playlist>> {
    let args = Query::new()
        .arg("name", name)
        .arg_list("songId", songs)
        .build();

    let res = client.get("createPlaylist", args)?;

    // TODO API is private
    // if client.api >= "1.14.0".into() {
    Ok(Some(serde_json::from_value(res)?))
    // } else {
    // Ok(None)
    // }
}

/// Updates a playlist. Only the owner of the playlist is privileged to do so.
fn update_playlist<'a, B, S>(
    client: &Client,
    id: u64,
    name: S,
    comment: S,
    public: B,
    to_add: &[u64],
    to_remove: &[u64],
) -> Result<()>
where
    S: Into<Option<&'a str>>,
    B: Into<Option<bool>>,
{
    let args = Query::new()
        .arg("id", id)
        .arg("name", name.into())
        .arg("comment", comment.into())
        .arg("public", public.into())
        .arg_list("songIdToAdd", to_add)
        .arg_list("songIndexToRemove", to_remove)
        .build();

    client.get("updatePlaylist", args)?;
    Ok(())
}

fn delete_playlist(client: &Client, id: u64) -> Result<()> {
    client.get("deletePlaylist", Query::with("id", id))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_util;

    // The demo playlist exists, but can't be accessed
    #[test]
    fn remote_playlist_songs() {
        let parsed = serde_json::from_value::<Playlist>(raw()).unwrap();
        let mut srv = test_util::demo_site().unwrap();
        let songs = parsed.songs(&mut srv);

        match songs {
            Err(::error::Error::Api(::error::ApiError::NotAuthorized(_))) => assert!(true),
            Err(e) => panic!("unexpected error: {}", e),
            Ok(_) => panic!("test should have failed; insufficient privilege"),
        }
    }

    fn raw() -> serde_json::Value {
        serde_json::from_str(
            r#"{
            "id" : "1",
            "name" : "Sleep Hits",
            "owner" : "user",
            "public" : false,
            "songCount" : 32,
            "duration" : 8334,
            "created" : "2018-01-01T14:45:07.464Z",
            "changed" : "2018-01-01T14:45:07.478Z",
            "coverArt" : "pl-2"
        }"#,
        )
        .unwrap()
    }
}
