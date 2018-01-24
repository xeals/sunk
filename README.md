# sunk

A library for interfacing with the Subsonic API.

## Example

TBA

## Currently supported

- Playlist controls
- Library scanning
- Retrieval (stream, download, HLS)
- Searching

## Currently not supported

- Most browsing
- Annotation
- Sharing
- Podcasts
- Jukebox
- Internet radio
- Chat
- User controls
- Bookmarks
- Anything to do with video

## Left to the implementor

- Playback (skipping, current playlist, etc.)

## What needs work

- ***Documentation***
- Reach full support for modern 1.14.0
- Pre-1.14.0 support:
  - XML parsing
  - Alternatives for newer methods
  - Cargo attribute for compile-time targeting?
- Proper module splitting
- Unit testing for methods that don't require a server, and manual get-and-verify for ones that do
- Go through [checklist](https://rust-lang-nursery.github.io/api-guidelines/checklist.html) before stabilisation

# License

Licensed under either of

 * Apache License, Version 2.0
    ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
    ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
