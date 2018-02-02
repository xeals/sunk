# sunk

A library for interfacing with the Subsonic API.

The library is designed to be as ergonomic and feel as natural as to Rust as possible.

It aims to support any version of the Subsonic API from 1.8.0 onwards.

## Quick usage

```rust
extern crate sunk;

let username = "guest3";
let password = "guest";
let site = "http://demo.subsonic.org";

let client = sunk::Client::new(site, username, password).unwrap();

// Update the library.
client.ping().unwrap()
client.scan_library().unwrap();

// Fetch some songs and play them.
let mut random = sunk::song::Song::random(&client, 20).unwrap();
for song in random {
    song.set_max_bit_rate(96);
    let bytes: Vec<u8> = song.stream(&client);
    // Pass `bytes` to an audio library to actually play the song.
}
```

# To-do

- Still unsupported (as yet):
    - Chat
    - Bookmarks
    - Most functionality for podcasts
- Not planned to be supported:
     - Shares; the system does not conform with the standard operating method of the rest of the library, and would require heavy refactoring or specialisation.
- Documentation!
- Unit testing, particularly for operations that require a server!

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
