use error;
use sunk;

pub fn demo_site() -> error::Result<sunk::Sunk> {
    let site = "http://demo.subsonic.org";
    let user = "guest3";
    let password = "guest";
    sunk::Sunk::new(site, user, password)
}
