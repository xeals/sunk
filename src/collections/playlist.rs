//! Playlist APIs.

use std::result;

use serde::de::{Deserialize, Deserializer};
use serde_json;

use crate::query::Query;
use crate::{Client, Error, Media, Result, Song};

#[allow(missing_docs)]
#[derive(Debug)]
#[readonly::make]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub duration: u64,
    pub cover_id: String,
    pub song_count: u64,
    pub songs: Vec<Song>,
}

impl Playlist {
    /// Fetches the songs contained in a playlist.
    pub fn songs(&self, client: &Client) -> Result<Vec<Song>> {
        if self.songs.len() as u64 != self.song_count {
            Ok(get_playlist(client, self.id.clone())?.songs)
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
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _Playlist {
            id: String,
            name: String,
            // #[serde(default)]
            // comment: String,
            // owner: String,
            song_count: u64,
            duration: u64,
            // created: String,
            // changed: String,
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

#[allow(missing_docs)]
pub fn get_playlists(client: &Client, user: Option<String>) -> Result<Vec<Playlist>> {
    let playlist = client.get("getPlaylists", Query::with("username", user))?;
    Ok(get_list_as!(playlist, Playlist))
}

#[allow(missing_docs)]
pub fn get_playlist(client: &Client, id: String) -> Result<Playlist> {
    let res = client.get("getPlaylist", Query::with("id", id))?;
    Ok(serde_json::from_value::<Playlist>(res)?)
}

/// Creates a playlist with the given name.
///
/// Since API version 1.14.0, the newly created playlist is returned. In earlier
/// versions, an empty response is returned.
pub fn create_playlist(client: &Client, name: String, songs: &[u64]) -> Result<Option<Playlist>> {
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
pub fn update_playlist<'a, B, S>(
    client: &Client,
    id: String,
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

#[allow(missing_docs)]
pub fn delete_playlist(client: &Client, id: String) -> Result<()> {
    client.get("deletePlaylist", Query::with("id", id))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util;

    // The demo playlist exists, but can't be accessed
    #[test]
    fn remote_playlist_songs() {
        let parsed = serde_json::from_value::<Playlist>(raw()).unwrap();
        let srv = test_util::demo_site().unwrap();
        let songs = parsed.songs(&srv);

        assert!(matches!(
            songs,
            Err(crate::error::Error::Api(
                crate::error::ApiError::NotAuthorized(_)
            ))
        ));
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
