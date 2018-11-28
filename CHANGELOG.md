# 0.1

## 0.1.2

- Fix panic on missing bitrate for a song
  - Mostly encountered on non-FLAC/MPEG source files (ogg, etc.)
- Add `with_target` method to the `Client` to set target API version

## 0.1.1

- Improved support for HLS
  - The playlist is parsed in-library and can be played with `Hls::get_bytes`
  
## 0.1.0

- Initial release
