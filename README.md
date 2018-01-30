# sunk

A library for interfacing with the Subsonic API.

## Example

```rust
extern crate sunk;

let username = "guest3";
let password = "guest";
let site = "http://demo.subsonic.org";

let mut server = sunk::Client::new(site, username, password).unwrap();

// Take a look at the documentation for what you can do with a `Client`.

// Update the library.
server.scan_library().unwrap();

// Fetch some songs and play them.
let mut random = sunk::song::get_random_songs(&mut server, 20).unwrap();
for song in random {
    song.set_max_bit_rate(96);
    let bytes: Vec<u8> = song.stream(&mut server);
    // Pass `bytes` to an audio library to actually play the song.
}
```

## Currently supported

- Playlist controls
- Library scanning
- Retrieval (stream, download, HLS)
- Searching and browsing/pivoting on results
- User controls
- Annotation
- Jukebox

## Currently not supported

- Sharing; no plans currently due to clash in design
- Podcasts (fully)
- Internet radio
- Chat
- Bookmarks

## Left to the implementor

- Playback (skipping, current playlist, etc.)

## What needs work

- ***Documentation***
- Reach full support for 1.14.0
- Proper module splitting
- Consistency in method naming
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
