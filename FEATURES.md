# Features

Tracker to match Subsonic API calls with the respective functions. Expect this
to change.

The API details are [here](http://www.subsonic.org/pages/api.jsp).

The aim is to match feature parity with v1.14.0, then v1.16.0. The minimum
supported version will be v1.8.0, to take advantage of functions that organise
by ID3 tags.

## System

| `ping`       | `Sunk::check_connection()` |
| `getLicense` | `Sunk::check_license()`    |

## Browsing

| `getMusicFolders`   | `Sunk::music_folders()`  |
| `getIndexes`        |                          |
| `getMusicDirectory` |                          |
| `getGenres`         | `Sunk::genres`           |
| `getArtists`        | `artist::get_artists()`  |
| `getArtist`         | `artist::get_artist()`   |
| `getAlbum`          | `album::get_album()`     |
| `getSong`           | `song::get_song()`       |
| `getVideos`         |                          |
| `getVideoInfo`      |                          |
| `getArtistInfo`     | alternative since 1.11.0 |
| `getArtistInfo2`    | `Artist::info()`         |
| `getAlbumInfo`      | alternative since 1.14.0 |
| `getAlbumInfo2`     | `Album::info()`          |
| `getSimilarSongs`   |                          |
| `getSimilarSongs2`  |                          |
| `getTopSongs`       |                          |

## Album/song lists

| `getAlbumList`    | alternatives since 1.8.0     |
| `getAlbumList2`   | `album::get_albums()`        |
| `getRandomSongs`  | `song::get_random_songs()`   |
| `getSongsByGenre` | `song::get_songs_in_genre()` |
| `getNowPlaying`   |                              |
| `getStarred`      |                              |
| `getStarred2`     |                              |

## Searching

| `search`  | deprecated since 1.4.0   |
| `search2` | alternatives since 1.8.0 |
| `search3` | `Sunk::search()`         |

## Playlists

| `getPlaylists`   | `playlist::get_playlists()`   |
| `getPlaylist`    | `playlist::get_playlist()`    |
| `createPlaylist` | `playlist::create_playlist()` |
| `updatePlaylist` | `playlist::update_playlist()` |
| `deletePlaylist` | `playlist::delete_playlist()` |

## Media retrieval

| `stream`      | `Song::stream()` and `Song::stream_url()` |
| `download`    | `Song::download_url()`                    |
| `hls`         | `Song::hls()`                             |
| `getCaptions` |                                           |
| `getCoverArt` | `$Struct::cover_art()`                    |
| `getLyrics`   | `song::get_lyrics()`                      |
| `getAvatar`   |                                           |

## Media annotation

| `star`      |   |
| `unstar`    |   |
| `setRating` |   |
| `scrobble`  |   |

## Sharing

| `getShares`   |   |
| `createShare` |   |
| `updateShare` |   |
| `deleteShare` |   |

## Podcast

| `getPodcasts`            |   |
| `getNewestPodcasts`      |   |
| `refreshPodcasts`        |   |
| `createPodcastChannel`   |   |
| `deletePodcastChannel`   |   |
| `deletePodcastEpisode`   |   |
| `downloadPodcastEpisode` |   |

## Jukebox

| `jukeboxControl` | |

## Internet radio

| `getInternetRadioStations`   |   |
| `createInternetRadioStation` |   |
| `updateInternetRadioStation` |   |
| `deleteInternetRadioStation` |   |
    
## Chat

| `getChatMessages` |   |
| `addChatMessage`  |   |
    
## User management

| `getUser`    |   |
| `getUsers`   |   |
| `createUser` |   |
| `updateUser` |   |
| `deleteUser` |   |
changePassword

## Bookmarks

| `getBookmarks`   |   |
| `createBookmark` |   |
| `deleteBookmark` |   |
| `getPlayQueue`   |   |
| `savePlayQueue`  |   |
    
## Media library scanning

| `getScanStatus` |   |
| `startScan`     |   |
