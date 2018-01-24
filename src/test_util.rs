use sunk;
use error;

pub fn demo_site() -> error::Result<sunk::Sunk> {
    let site = "demo.subsonic.org";
    let user = "guest3";
    let password = "guest";
    sunk::Sunk::new(site, user, password)
}
