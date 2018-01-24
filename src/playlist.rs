use serde_json;

use error::*;
use query::Query;
use sunk::Sunk;
use util::*;

use song;

#[derive(Debug)]
pub struct Playlist {
    id: u64,
    name: String,
    duration: u64,
    cover_id: String,
    song_count: u64,
    songs: Vec<song::Song>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct PlaylistSerde {
    id: String,
    name: String,
    comment: Option<String>,
    owner: String,
    songCount: u64,
    duration: u64,
    created: String,
    changed: String,
    coverArt: String,
}

impl Playlist {
    /// Deserialzises a JSON value into an artist.
    ///
    /// # Notes
    ///
    /// This is a temporary function until TryFrom is stabilised.
    pub fn try_from(json: serde_json::Value) -> Result<Playlist> {
        let mut songs = Vec::new();
        if let Some(Some(list)) = json.get("entry").map(|e| e.as_array()) {
            for song in list {
                info!(
                    "Found song {} for playlist {}",
                    song["name"], json["name"]
                );
                songs.push(song::Song::try_from(song.clone())?);
            }
        }

        let serde: PlaylistSerde = serde_json::from_value(json)?;
        Ok(Playlist {
            id: serde.id.parse()?,
            name: serde.name,
            duration: serde.duration,
            cover_id: serde.coverArt,
            song_count: serde.songCount,
            songs,
        })
    }

    /// Fetches the songs contained in a playlist.
    pub fn songs(&self, sunk: &mut Sunk) -> Result<Vec<song::Song>> {
        if self.songs.len() as u64 != self.song_count {
            Ok(get_playlist(sunk, self.id)?.songs)
        } else {
            Ok(self.songs.clone())
        }
    }

    // impl_cover_art!();
}

fn get_playlists(
    sunk: &mut Sunk,
    user: Option<String>,
) -> Result<Vec<Playlist>> {
    let res = sunk.get("getPlaylists", Query::maybe_with("username", user))?;

    let mut pls = vec![];
    if let Some(pl) = res["playlist"].as_array() {
        for p in pl.clone() {
            pls.push(Playlist::try_from(p)?);
        }
    }
    Ok(pls)
}

fn get_playlist(sunk: &mut Sunk, id: u64) -> Result<Playlist> {
    let res = sunk.get("getPlaylist", Query::with("id", id))?;
    Playlist::try_from(res)
}

// fn get_playlist_songs(sunk: &mut Sunk, id: u64) -> Result<Vec<song::Song>> {
//     let res = sunk.get("getPlaylist", Query::with("id", id))?;

//     let mut list = Vec::new();
//     if let Some(songs) = res["entry"].as_array() {
//         for song in songs {
//             list.push(song::Song::from(song)?);
//         }
//     }
//     Ok(list)
// }

/// Creates a playlist with the given name.
///
/// Since API version 1.14.0, the newly created playlist is returned. In earlier
/// versions, an empty response is returned.
fn create_playlist(
    sunk: &mut Sunk,
    name: String,
    songs: Option<Vec<u64>>,
) -> Result<Option<Playlist>> {
    let args = Query::new()
        .arg("name", name)
        .maybe_arg_list("songId", map_vec_string(songs))
        .build();

    let res = sunk.get("createPlaylist", args)?;
    // TODO Match the API and return the playlist on new versions.

    Ok(None)
}

/// Updates a playlist. Only the owner of the playlist is privileged to do so.
fn update_playlist(
    sunk: &mut Sunk,
    id: u64,
    name: Option<String>,
    comment: Option<String>,
    public: Option<bool>,
    to_add: Option<Vec<u64>>,
    to_remove: Option<Vec<u64>>,
) -> Result<()> {
    let args = Query::new()
        .arg("id", id.to_string())
        .maybe_arg("name", name)
        .maybe_arg("comment", comment)
        .maybe_arg("public", map_str(public))
        .maybe_arg_list("songIdToAdd", map_vec_string(to_add))
        .maybe_arg_list("songIndexToRemove", map_vec_string(to_remove))
        .build();

    sunk.get("updatePlaylist", args)?;

    Ok(())
}

fn delete_playlist(sunk: &mut Sunk, id: u64) -> Result<()> {
    sunk.get("deletePlaylist", Query::with("id", id))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_util::*;

    #[test]
    fn remote_playlist_songs() {
        let raw = json!(
            {
                "id" : "1",
                "name" : "Sleep Hits",
                "owner" : "user",
                "public" : false,
                "songCount" : 32,
                "duration" : 8334,
                "created" : "2018-01-01T14:45:07.464Z",
                "changed" : "2018-01-01T14:45:07.478Z",
                "coverArt" : "pl-2"
            }
        );

        let parsed = Playlist::try_from(raw).unwrap();
        let auth = load_credentials().unwrap();
        let mut srv = Sunk::new(&auth.0, &auth.1, &auth.2).unwrap();
        let songs = parsed.songs(&mut srv).unwrap();

        println!("{:?}", songs);
        assert!(!songs.is_empty())
    }
}
