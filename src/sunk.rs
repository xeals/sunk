// use url;
use hyper::{self, Uri};
use futures;
use tokio;

use md5;
use rand;

use error::*;

const SALT_SIZE: usize = 36; // Minimum 6 characters.
const TARGET_API: &str = "1.14.0"; // For JSON support.

pub struct Sunk {
    url: String,
    auth: SunkAuth,
    client: hyper::Client<hyper::client::HttpConnector>,
}

struct SunkAuth {
    user: String,
    password: String,
    // salt: String,
    // token: String,
}

impl SunkAuth {
    fn new(user: &str, password: &str) -> SunkAuth {
        SunkAuth {
            user: user.into(),
            password: password.into(),
            // salt: salt,
            // token: token,
        }
    }

    // TODO Actual version comparison support
    fn as_uri(&self) -> String {
        // First md5 support.
        let auth = if TARGET_API >= "1.13.0" {
            use rand::{thread_rng, Rng};

            let salt: String = thread_rng().gen_ascii_chars().take(SALT_SIZE).collect();
            let pre_t = self.password.to_string() + &salt;
            let token = format!("{:x}", md5::compute(pre_t.as_bytes()));

            // As detailed in http://www.subsonic.org/pages/api.jsp
            format!("u={u}&t={t}&s={s}",
                    u = self.user,
                    t = token,
                    s = salt)
        } else {
            format!("u={u}&p={p}",
                    u = self.user,
                    p = self.password)
        };

        // Prefer JSON.
        let format = if TARGET_API >= "1.14.0" {
            "json"
        } else {
            "xml"
        };

        let crate_name = ::std::env::var("CARGO_PKG_NAME").unwrap();

        format!("{auth}&v={v}&c={c}&f={f}",
                auth = auth,
                v = TARGET_API,
                c = crate_name,
                f = format)
    }
}

impl Sunk {
    fn new(url: &str, user: &str, password: &str) -> Result<Sunk> {
        use std::str::FromStr;

        let auth = SunkAuth::new(user, password);
        let uri = Uri::from_str(url)?;

        let mut core = tokio::reactor::Core::new()?;
        let client = hyper::Client::new(&core.handle());

        Ok(Sunk {
            url: uri.authority().unwrap().to_string(),
            auth: auth,
            client: client,
        })
    }
}

#[cfg(test)]
mod tests {
    use sunk::*;
}
