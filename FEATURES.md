# Features

Tracker to match Subsonic API calls with the respective functions. Expect this
to change.

The API details are [here](http://www.subsonic.org/pages/api.jsp).

The aim is to match feature parity with v1.14.0, then v1.16.0. The minimum
supported version will be v1.8.0, to take advantage of functions that organise
by ID3 tags.

## System

|              |                              |
|--------------|------------------------------|
| `ping`       | `Client::check_connection()` |
| `getLicense` | `Client::check_license()`    |

## Browsing

|                     |                                |
|---------------------|--------------------------------|
| `getMusicFolders`   | `Client::music_folders()`      |
| `getIndexes`        | unsupported (in favour of ID3) |
| `getMusicDirectory` | unsupported (in favour of ID3) |
| `getGenres`         | `Client::genres()`             |
| `getArtists`        | `artist::get_artists()`        |
| `getArtist`         | `artist::get_artist()`         |
| `getAlbum`          | `album::get_album()`           |
| `getSong`           | `song::get_song()`             |
| `getVideos`         | `Video::list()`                |
| `getVideoInfo`      | `Video::info()`                |
| `getArtistInfo`     | does not use ID3               |
| `getArtistInfo2`    | `Artist::info()`               |
| `getAlbumInfo`      | does not use ID3               |
| `getAlbumInfo2`     | `Album::info()`                |
| `getSimilarSongs`   | does not use ID3               |
| `getSimilarSongs2`  | `Song::similar()`              |
| `getTopSongs`       | `Artist::top_songs()`          |

## Album/song lists

|                   |                              |
|-------------------|------------------------------|
| `getAlbumList`    | does not use ID3             |
| `getAlbumList2`   | `album::get_albums()`        |
| `getRandomSongs`  | `song::get_random_songs()`   |
| `getSongsByGenre` | `song::get_songs_in_genre()` |
| `getNowPlaying`   | `Client::now_playing()`      |
| `getStarred`      | does not use ID3             |
| `getStarred2`     | `Client::starred()`          |

## Searching

|           |                        |
|-----------|------------------------|
| `search`  | deprecated since 1.4.0 |
| `search2` | does not use ID3       |
| `search3` | `Client::search()`     |

## Playlists

|                  |                               |
|------------------|-------------------------------|
| `getPlaylists`   | `playlist::get_playlists()`   |
| `getPlaylist`    | `playlist::get_playlist()`    |
| `createPlaylist` | `playlist::create_playlist()` |
| `updatePlaylist` | `playlist::update_playlist()` |
| `deletePlaylist` | `playlist::delete_playlist()` |

## Media retrieval

|               |                                                           |
|---------------|-----------------------------------------------------------|
| `stream`      | `Streamable::stream()` and `Streamable::stream_url()`     |
| `download`    | `Streamable::download()` and `Streamable::download_url()` |
| `hls`         | `Song::hls()`                                             |
| `getCaptions` | `Video::captions()`                                       |
| `getCoverArt` | `Media::cover_art()`                                      |
| `getLyrics`   | `song::get_lyrics()`                                      |
| `getAvatar`   | `User::avatar()`                                          |

## Media annotation

|             |                             |
|-------------|-----------------------------|
| `star`      | `Annotatable::star()`       |
| `unstar`    | `Annotatable::unstar()`     |
| `setRating` | `Annotatable::set_rating()` |
| `scrobble`  | `Annotatable::scrobble()`   |

## Sharing

|               |   |
|---------------|---|
| `getShares`   |   |
| `createShare` |   |
| `updateShare` |   |
| `deleteShare` |   |

## Podcast

|                          |                                        |
|--------------------------|----------------------------------------|
| `getPodcasts`            | `Podcast::get()` and `Podcast::list()` |
| `getNewestPodcasts`      | `Episode::newest()`                    |
| `refreshPodcasts`        |                                        |
| `createPodcastChannel`   |                                        |
| `deletePodcastChannel`   |                                        |
| `deletePodcastEpisode`   |                                        |
| `downloadPodcastEpisode` |                                        |

## Jukebox

|                  |   |
|------------------|---|
| `jukeboxControl` |   |

## Internet radio

|                              |   |
|------------------------------|---|
| `getInternetRadioStations`   |   |
| `createInternetRadioStation` |   |
| `updateInternetRadioStation` |   |
| `deleteInternetRadioStation` |   |
    
## Chat

|                   |   |
|-------------------|---|
| `getChatMessages` |   |
| `addChatMessage`  |   |
    
## User management

|                  |                           |
|------------------|---------------------------|
| `getUser`        | `user::get_user()`        |
| `getUsers`       | `user::get_users()`       |
| `createUser`     |                           |
| `updateUser`     | `user::update_user()`     |
| `deleteUser`     | `user::delete_user()`     |
| `changePassword` | `user::change_password()` |

## Bookmarks

|                  |   |
|------------------|---|
| `getBookmarks`   |   |
| `createBookmark` |   |
| `deleteBookmark` |   |
| `getPlayQueue`   |   |
| `savePlayQueue`  |   |
    
## Media library scanning

|                 |                          |
|-----------------|--------------------------|
| `getScanStatus` | `Client::scan_status()`  |
| `startScan`     | `Client::scan_library()` |
