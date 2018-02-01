use serde_json;

use ApiError;

/// A top-level response from a Subsonic server.
#[derive(Debug, Deserialize)]
pub struct Response {
    #[serde(rename = "subsonic-response")]
    inner: InnerResponse,
}

/// A struct containing the possible responses of the Subsonic API.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InnerResponse {
    status: String,
    version: String,
    error: Option<ApiError>,
    license: Option<serde_json::Value>,
    music_folders: Option<serde_json::Value>,
    indexes: Option<serde_json::Value>,
    directory: Option<serde_json::Value>,
    genres: Option<serde_json::Value>,
    artists: Option<serde_json::Value>,
    artist: Option<serde_json::Value>,
    albums: Option<serde_json::Value>,
    album: Option<serde_json::Value>,
    song: Option<serde_json::Value>,
    videos: Option<serde_json::Value>,
    video_info: Option<serde_json::Value>,
    artist_info: Option<serde_json::Value>,
    artist_info2: Option<serde_json::Value>,
    album_info: Option<serde_json::Value>,
    similar_songs: Option<serde_json::Value>,
    similar_songs2: Option<serde_json::Value>,
    top_songs: Option<serde_json::Value>,
    album_list: Option<serde_json::Value>,
    album_list2: Option<serde_json::Value>,
    random_songs: Option<serde_json::Value>,
    songs_by_genre: Option<serde_json::Value>,
    now_playing: Option<serde_json::Value>,
    starred: Option<serde_json::Value>,
    starred2: Option<serde_json::Value>,
    search_result: Option<serde_json::Value>,
    search_result2: Option<serde_json::Value>,
    search_result3: Option<serde_json::Value>,
    playlists: Option<serde_json::Value>,
    playlist: Option<serde_json::Value>,
    lyrics: Option<serde_json::Value>,
    shares: Option<serde_json::Value>,
    podcasts: Option<serde_json::Value>,
    newest_podcasts: Option<serde_json::Value>,
    jukebox_status: Option<serde_json::Value>,
    jukebox_playlist: Option<serde_json::Value>,
    internet_radio_stations: Option<serde_json::Value>,
    chat_messages: Option<serde_json::Value>,
    user: Option<serde_json::Value>,
    users: Option<serde_json::Value>,
    bookmarks: Option<serde_json::Value>,
    play_queue: Option<serde_json::Value>,
    scan_status: Option<serde_json::Value>,
}

impl Response {
    /// Extracts the internal value of the response.
    ///
    /// # Errors
    ///
    /// This method will error if the response contained an error (as defined by
    /// the [Subsonic API]).
    ///
    /// [Subsonic API]: ./enum.ApiError.html
    pub fn into_value(self) -> Option<serde_json::Value> {
        // TODO Big time; make this not an `if ... else if ...` mess.
        macro_rules! choose {
            ( $($f:ident),* ) => ({ $(
                if let Some(v)  = self.inner.$f {
                    return Some(v)
                }
            )* })
        }

        if self.inner.error.is_some() {
            return None
        }

        choose!(
            album,
            album_info,
            album_list,
            album_list2,
            albums,
            artist,
            artist_info,
            artist_info2,
            artists,
            bookmarks,
            chat_messages,
            directory,
            genres,
            indexes,
            internet_radio_stations,
            jukebox_playlist,
            jukebox_status,
            license,
            lyrics,
            music_folders,
            music_folders,
            newest_podcasts,
            now_playing,
            play_queue,
            playlist,
            playlists,
            podcasts,
            random_songs,
            scan_status,
            search_result,
            search_result2,
            search_result3,
            shares,
            similar_songs,
            similar_songs2,
            song,
            songs_by_genre,
            starred,
            starred2,
            top_songs,
            user,
            users,
            video_info,
            videos
        );
        None
    }

    /// Extracts the error struct of the response. Returns `None` if the
    /// response was not a failure.
    pub fn into_error(self) -> Option<ApiError> { self.inner.error }

    /// Returns `true` if the response is `"ok"`.
    pub fn is_ok(&self) -> bool { self.inner.error.is_none() }

    /// Returns `true` if the response is `"failed"`.
    pub fn is_err(&self) -> bool { !self.is_ok() }

    // /// Returns `true` if the response is `"ok"`, but the response body
    // is empty. pub fn is_empty(&self) -> bool { self.is_ok() &&
    // self.into_value().is_none() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_err_result() {
        let fail = r#"{"subsonic-response": {
            "status": "failed",
            "version": "1.14.0",
            "error": {
                "code": 70,
                "message": "Requested resource not found"
            }
        }}"#;
        let fail = serde_json::from_str::<Response>(fail).unwrap();
        assert!(fail.into_error().is_some());

        let success = r#"{"subsonic-response": {
            "status": "ok",
            "version": "1.14.0"
        }}"#;
        let success = serde_json::from_str::<Response>(success).unwrap();
        assert!(success.into_error().is_none());
    }
}
