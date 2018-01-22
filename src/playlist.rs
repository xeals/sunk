use json;

use query::Query;
use error::*;
use util::*;
use song::Song;
use sunk::Sunk;

#[derive(Debug)]
pub struct Playlist {
    id:         u64,
    name:       String,
    song_count: u64,
    duration:   u64,
    cover:      String,
}

impl Playlist {
    /// Parses a JSON map into a Playlist struct.
    pub fn from(j: &json::Value) -> Result<Playlist> {
        if !j.is_object() {
            return Err(Error::ParseError("not an object"))
        }

        Ok(Playlist {
            id:         fetch!(j->id: as_str, u64),
            name:       fetch!(j->name: as_str).into(),
            song_count: fetch!(j->songCount: as_u64),
            duration:   fetch!(j->duration: as_u64),
            cover:      fetch!(j->coverArt: as_str).into(),
        })
    }

    /// Fetches the songs contained in a playlist.
    pub fn songs(&self, sunk: &mut Sunk) -> Result<Vec<Song>> {
        get_playlist_content(sunk, self.id)
    }

    impl_cover_art!();
}

fn get_playlists(
    sunk: &mut Sunk,
    user: Option<String>,
) -> Result<Vec<Playlist>> {
    let res = sunk.get("getPlaylists", Query::maybe_with("username", user))?;

    let mut pls = vec![];
    for pl in pointer!(res, "/subsonic-response/playlists/playlist")
        .as_array()
        .ok_or(Error::ParseError("not an array"))?
    {
        pls.push(Playlist::from(pl)?);
    }
    Ok(pls)
}

fn get_playlist(sunk: &mut Sunk, id: u64) -> Result<Playlist> {
    let res = sunk.get("getPlaylist", Query::with("id", id))?;
    Playlist::from(&res["subsonic-response"]["playlist"])
}

fn get_playlist_content(sunk: &mut Sunk, id: u64) -> Result<Vec<Song>> {
    let res = sunk.get("getPlaylist", Query::with("id", id))?;
    let mut list = vec![];
    for song in pointer!(res, "/subsonic-response/playlist/entry")
        .as_array()
        .ok_or(Error::ParseError("not an array"))?
    {
        list.push(Song::from(song)?);
    }
    Ok(list)
}

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
    fn test_songs_from_playlist() {
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

        let parsed = Playlist::from(&raw).unwrap();
        let auth = load_credentials().unwrap();
        let mut srv = Sunk::new(&auth.0, &auth.1, &auth.2).unwrap();
        let songs = parsed.songs(&mut srv).unwrap();
    }
}
