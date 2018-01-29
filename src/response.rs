use serde_json;

use error::{ApiError, Error, Result};

/// A top-level response from a Subsonic server.
#[derive(Debug, Deserialize)]
pub struct Response {
    #[serde(rename = "subsonic-response")]
    inner: InnerResponse,
}

/// A struct containing the possible responses of the Subsonic API.
// #[derive(Debug, Deserialize)]
// #[serde(untagged)]
// enum InnerResponse {
//     Album {status: String, version: String, album: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     AlbumInfo {status: String, version: String, album_info: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     AlbumList {status: String, version: String, album_list: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     AlbumList2 {status: String, version: String, album_list2: serde_json::Value},
//     Albums {status: String, version: String, albums: serde_json::Value},
//     Artist {status: String, version: String, artist: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     ArtistInfo {status: String, version: String, artist_info: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     ArtistInfo2 {status: String, version: String, artist_info2: serde_json::Value},
//     Artists {status: String, version: String, artists: serde_json::Value},
//     Bookmarks {status: String, version: String, bookmarks: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     ChatMessages {status: String, version: String, chat_messages: serde_json::Value},
//     Directory {status: String, version: String, directory: serde_json::Value},
//     Genres {status: String, version: String, genres: serde_json::Value},
//     Indexes {status: String, version: String, indexes: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     InternetRadioStations {status: String, version: String, internet_radio_stations: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     JukeboxPlaylist {status: String, version: String, jukebox_playlist: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     JukeboxStatus {status: String, version: String, jukebox_status: serde_json::Value},
//     License {status: String, version: String, license: serde_json::Value},
//     Lyrics {status: String, version: String, lyrics: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     MusicFolders {status: String, version: String, music_folders: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     NewestPodcasts {status: String, version: String, newest_podcasts: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     NowPlaying {status: String, version: String, now_playing: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     PlayQueue {status: String, version: String, play_queue: serde_json::Value},
//     Playlist {status: String, version: String, playlist: serde_json::Value},
//     Playlists {status: String, version: String, playlists: serde_json::Value},
//     Podcasts {status: String, version: String, podcasts: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     RandomSongs {status: String, version: String, random_songs: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     ScanStatus {status: String, version: String, scan_status: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     SearchResult {status: String, version: String, search_result: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     SearchResult2 {status: String, version: String, search_result2: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     SearchResult3 {status: String, version: String, search_result3: serde_json::Value},
//     Shares {status: String, version: String, shares: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     SimilarSongs {status: String, version: String, similar_songs: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     SimilarSongs2 {status: String, version: String, similar_songs2: serde_json::Value},
//     Song {status: String, version: String, song: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     SongsByGenre {status: String, version: String, songs_by_genre: serde_json::Value},
//     Starred {status: String, version: String, starred: serde_json::Value},
//     Starred2 {status: String, version: String, starred2: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     TopSongs {status: String, version: String, top_songs: serde_json::Value},
//     User {status: String, version: String, user: serde_json::Value},
//     Users {status: String, version: String, users: serde_json::Value},
//     #[serde(rename_all = "camelCase")]
//     VideoInfo {status: String, version: String, video_info: serde_json::Value},
//     Videos {status: String, version: String, videos: serde_json::Value},
//     Error{status: String, version: String, error: ApiError},
//     Empty{status: String, version: String},
// }
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InnerResponse {
    status: String,
    version: String,
    #[doc(hidden)]
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
    /// [Subsonic API]: ../error/enum.ApiError.html
    // pub fn into_value(self) -> Option<serde_json::Value> {
    //     println!("{:?}", self);
    //     match self.inner {
    //         InnerResponse::Empty{..} => Some(serde_json::Value::Null),
    //         InnerResponse::Error{..} => None,
    //         InnerResponse::License {license, ..} => Some(license),
    //         InnerResponse::MusicFolders {music_folders, ..} => Some(music_folders),
    //         InnerResponse::Indexes {indexes, ..} => Some(indexes),
    //         InnerResponse::Directory {directory, ..} => Some(directory),
    //         InnerResponse::Genres {genres, ..} => Some(genres),
    //         InnerResponse::Artists {artists, ..} => Some(artists),
    //         InnerResponse::Artist {artist, ..} => Some(artist),
    //         InnerResponse::Albums {albums, ..} => Some(albums),
    //         InnerResponse::Album {album, ..} => Some(album),
    //         InnerResponse::Song {song, ..} => Some(song),
    //         InnerResponse::Videos {videos, ..} => Some(videos),
    //         InnerResponse::VideoInfo {video_info, ..} => Some(video_info),
    //         InnerResponse::ArtistInfo {artist_info, ..} => Some(artist_info),
    //         InnerResponse::ArtistInfo2 {artist_info2, ..} => Some(artist_info2),
    //         InnerResponse::AlbumInfo {album_info, ..} => Some(album_info),
    //         InnerResponse::SimilarSongs {similar_songs, ..} => Some(similar_songs),
    //         InnerResponse::SimilarSongs2 {similar_songs2, ..} => Some(similar_songs2),
    //         InnerResponse::TopSongs {top_songs, ..} => Some(top_songs),
    //         InnerResponse::AlbumList {album_list, ..} => Some(album_list),
    //         InnerResponse::AlbumList2 {album_list2, ..} => Some(album_list2),
    //         InnerResponse::RandomSongs {random_songs, ..} => Some(random_songs),
    //         InnerResponse::SongsByGenre {songs_by_genre, ..} => Some(songs_by_genre),
    //         InnerResponse::NowPlaying {now_playing, ..} => Some(now_playing),
    //         InnerResponse::Starred {starred, ..} => Some(starred),
    //         InnerResponse::Starred2 {starred2, ..} => Some(starred2),
    //         InnerResponse::SearchResult {search_result, ..} => Some(search_result),
    //         InnerResponse::SearchResult2 {search_result2, ..} => Some(search_result2),
    //         InnerResponse::SearchResult3 {search_result3, ..} => Some(search_result3),
    //         InnerResponse::Playlists {playlists, ..} => Some(playlists),
    //         InnerResponse::Playlist {playlist, ..} => Some(playlist),
    //         InnerResponse::Lyrics {lyrics, ..} => Some(lyrics),
    //         InnerResponse::Shares {shares, ..} => Some(shares),
    //         InnerResponse::Podcasts {podcasts, ..} => Some(podcasts),
    //         InnerResponse::NewestPodcasts {newest_podcasts, ..} => Some(newest_podcasts),
    //         InnerResponse::JukeboxStatus {jukebox_status, ..} => Some(jukebox_status),
    //         InnerResponse::JukeboxPlaylist {jukebox_playlist, ..} => Some(jukebox_playlist),
    //         InnerResponse::InternetRadioStations {internet_radio_stations, ..} => Some(internet_radio_stations),
    //         InnerResponse::ChatMessages {chat_messages, ..} => Some(chat_messages),
    //         InnerResponse::User {user, ..} => Some(user),
    //         InnerResponse::Users {users, ..} => Some(users),
    //         InnerResponse::Bookmarks {bookmarks, ..} => Some(bookmarks),
    //         InnerResponse::PlayQueue {play_queue, ..} => Some(play_queue),
    //         InnerResponse::ScanStatus {scan_status, ..} => Some(scan_status),
    //     }
    // }
    pub fn into_value(self) -> Option<serde_json::Value> {
        // TODO Big time; make this not an `if ... else if ...` mess.
        macro_rules! maybe {
            ($f:ident) => ({
                if let Some(v)  = self.inner.$f {
                    return Some(v)
                }
            })
        }

        if let Some(err) = self.inner.error {
            return None
        }

        maybe!(license);
        maybe!(music_folders);
        maybe!(music_folders);
        maybe!(indexes);
        maybe!(directory);
        maybe!(genres);
        maybe!(artists);
        maybe!(artist);
        maybe!(albums);
        maybe!(album);
        maybe!(song);
        maybe!(videos);
        maybe!(video_info);
        maybe!(artist_info);
        maybe!(artist_info2);
        maybe!(album_info);
        maybe!(similar_songs);
        maybe!(similar_songs2);
        maybe!(top_songs);
        maybe!(album_list);
        maybe!(album_list2);
        maybe!(random_songs);
        maybe!(songs_by_genre);
        maybe!(now_playing);
        maybe!(starred);
        maybe!(starred2);
        maybe!(search_result);
        maybe!(search_result2);
        maybe!(search_result3);
        maybe!(playlists);
        maybe!(playlist);
        maybe!(lyrics);
        maybe!(shares);
        maybe!(podcasts);
        maybe!(newest_podcasts);
        maybe!(jukebox_status);
        maybe!(jukebox_playlist);
        maybe!(internet_radio_stations);
        maybe!(chat_messages);
        maybe!(user);
        maybe!(users);
        maybe!(bookmarks);
        maybe!(play_queue);
        maybe!(scan_status);
        None
    }

    /// Extracts the error struct of the response. Returns `None` if the
    /// response was not a failure.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// #[macro_use]
    /// extern crate serde_json;
    /// extern crate sunk;
    /// use sunk::response::Response;
    ///
    /// # fn run() -> Result<(), sunk::Error> {
    /// let fail = r#"{"subsonic-response": {
    ///     "status": "failed",
    ///     "version": "1.14.0",
    ///     "error": {
    ///         "code": 70,
    ///         "message": "Requested resource not found"
    ///     }
    /// }"#;
    /// let fail = serde_json::from_str::<Response>(fail)?;
    /// assert!(fail.into_error().is_some());
    ///
    /// let success = r#"{"subsonic-response": {
    ///     "status": "ok",
    ///     "version": "1.14.0"
    /// }}"#;
    /// let success = serde_json::from_str::<Response>(success)?;
    /// assert!(success.into_error().is_none());
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// #   run().unwrap();
    /// # }
    /// ```
    pub fn into_error(self) -> Option<Error> {
        self.inner.error.map(|e| e.into())
        // if let InnerResponse::Error{error, ..} = self.inner {
        //     Some(error.into())
        // } else {
        //     None
        // }
    }

    /// Returns `true` if the response is `"ok"`.
    pub fn is_ok(&self) -> bool {
        self.inner.error.is_none()
        // if let InnerResponse::Error{..} = self.inner {
        //     false
        // } else {
        //     true
        // }
    }

    /// Returns `true` if the response is `"failed"`.
    pub fn is_err(&self) -> bool { !self.is_ok() }
}
