// use url;
use hyper::{self, Uri, Client};
use hyper_tls::HttpsConnector;
use futures;
use tokio;
use serde;
use json;
use md5;
use rand;
use log;

use error::*;

const SALT_SIZE: usize = 36; // Minimum 6 characters.
const TARGET_API: &str = "1.14.0"; // For JSON support.

#[derive(Debug)]
pub struct Sunk {
    url: Uri,
    auth: SunkAuth,
    client: Client<HttpsConnector<hyper::client::HttpConnector>>,
    core: tokio::reactor::Core,
}

#[derive(Debug)]
struct SunkAuth {
    user: String,
    password: String,
}

impl SunkAuth {
    fn new(user: &str, password: &str) -> SunkAuth {
        SunkAuth {
            user: user.into(),
            password: password.into(),
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
    pub fn new(url: &str, user: &str, password: &str) -> Result<Sunk> {
        use std::str::FromStr;

        let auth = SunkAuth::new(user, password);
        let uri = Uri::from_str(url)?;

        let mut core = tokio::reactor::Core::new()?;
        let handle = core.handle();
        let client = Client::configure()
            .connector(HttpsConnector::new(4, &handle)
                       .map_err(|_| Error::UnknownError("TLS"))?)
            .build(&handle);

        Ok(Sunk {
            url: uri,
            auth: auth,
            client: client,
            core: core,
        })
    }

    // fn get<'de, T>(&mut self, query: &str) -> Result<(u16, T)>
    // where
    //     T: serde::Deserialize<'de>
    pub fn get<S>(&mut self, query: &str, args: Vec<(&str, S)>) -> Result<(u16, json::Value)>
    where
        S: ::std::fmt::Display + ::std::string::ToString
    {
        use futures::{Future, Stream};

        let scheme = self.url.scheme().or_else(|| {
            warn!("No scheme provided; falling back to http");
            Some("http")
        }).ok_or(Error::ServerError("Unable to determine scheme".into()))?;
        let addr = self.url.authority()
            .ok_or(Error::ServerError("No address provided".into()))?;

        let mut url = [scheme, "://", addr, "/rest/"].concat();
        url.push_str(query);
        url.push_str("?");
        url.push_str(&self.auth.as_uri());
        if !args.is_empty() {
            for a in &args {
                url.push_str("&");
                url.push_str(&format!("{}={}", a.0, a.1));
            }
        }

        let uri = url.parse().unwrap();
        debug!("uri: {}", uri);
        let work = self.client.get(uri).and_then(|res| {
            let status = res.status();
            info!("Received `{}` for request /{}?", status, query);

            res.body().concat2().and_then(move |body| {
                let v: json::Value = json::from_slice(&body).map_err(|e| {
                    use std::io;
                    io::Error::new(io::ErrorKind::Other, e)
                })?;
                Ok((status.as_u16(), v))
            })
        });

        self.core.run(work).map_err(|e| Error::HyperError(e))
    }

    /// Attempts to connect to the server with the provided credentials.
    /// Subsonic API will throw an error on any of the following:
    /// - invalid credentials
    /// - incorrect API target
    fn check_connection(&mut self) -> Result<()> {
        // let (code, _res) = self.get::<json::Value>("ping.view")?;
        let (code, _res) = self.get("ping", vec![("", "")])?;
        let res = &_res["subsonic-response"];

        macro_rules! err (
            ($e:expr) => (return Err(Error::ServerError($e)))
        );

        if code >= 400 {
            err!(format!("server not found: error {}", code))
        }

        match res["status"].as_str() {
            Some("ok") => {},
            Some("failed") => {
                if let Some(i) = res["error"].as_u64() {
                    return subsonic_err(
                        i, TARGET_API,
                        &res["version"],
                        &res["error"]["message"]
                    )
                } else {
                    err!(format!("unexpected respone: {:?}", res))
                }
            }
            _ => unreachable!()
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sunk::*;
    use std::io;
    use test_util::*;

    #[test]
    fn test_ping() {
        let (site, user, pass) = load_credentials().unwrap();
        let mut srv = Sunk::new(&site, &user, &pass)
            .expect("Failed to start client");
        debug!("{:?}", srv);
        srv.check_connection().unwrap();
        assert!(srv.check_connection().is_ok())
    }
}
