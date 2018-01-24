use std::io::{self, BufRead, BufReader};

use sunk;
use error;

pub fn load_credentials() -> io::Result<(String, String, String)> {
    let mut file = ::std::fs::File::open("credentials")?;
    let mut it = BufReader::new(file).lines();
    let site = it.next().unwrap()?;
    let user = it.next().unwrap()?;
    let pass = it.next().unwrap()?;
    Ok((site, user, pass))
}

pub fn demo_site() -> error::Result<sunk::Sunk> {
    let site = "http://demo.subsonic.org";
    let user = "guest3";
    let password = "guest";
    sunk::Sunk::new(site, user, password)
}
