// use url;
use futures;
use hyper::{self, Client, Uri};
use hyper_tls::HttpsConnector;
use json;
use log;
use md5;
use rand;
use serde;
use tokio;

use error::*;

const SALT_SIZE: usize = 36; // Minimum 6 characters.
const TARGET_API: &str = "1.14.0"; // For JSON support.

#[derive(Debug)]
pub struct Sunk {
    url:    Uri,
    auth:   SunkAuth,
    client: Client<HttpsConnector<hyper::client::HttpConnector>>,
    core:   tokio::reactor::Core,
}

#[derive(Debug)]
struct SunkAuth {
    user:     String,
    password: String,
}

impl SunkAuth {
    fn new(user: &str, password: &str) -> SunkAuth {
        SunkAuth {
            user:     user.into(),
            password: password.into(),
        }
    }

    // TODO Actual version comparison support
    fn as_uri(&self) -> String {
        // First md5 support.
        let auth = if TARGET_API >= "1.13.0" {
            use rand::{thread_rng, Rng};

            let salt: String =
                thread_rng().gen_ascii_chars().take(SALT_SIZE).collect();
            let pre_t = self.password.to_string() + &salt;
            let token = format!("{:x}", md5::compute(pre_t.as_bytes()));

            // As detailed in http://www.subsonic.org/pages/api.jsp
            format!("u={u}&t={t}&s={s}", u = self.user, t = token, s = salt)
        } else {
            format!("u={u}&p={p}", u = self.user, p = self.password)
        };

        // Prefer JSON.
        let format = if TARGET_API >= "1.14.0" {
            "json"
        } else {
            "xml"
        };

        let crate_name = ::std::env::var("CARGO_PKG_NAME").unwrap();

        format!(
            "{auth}&v={v}&c={c}&f={f}",
            auth = auth,
            v = TARGET_API,
            c = crate_name,
            f = format
        )
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
            url:    uri,
            auth:   auth,
            client: client,
            core:   core,
        })
    }

    /// Internal helper function to construct a URL when the actual fetching is
    /// not required.
    ///
    /// Formats arguments in a standard HTTP format, using information from the
    /// `Sunk`; for example:
    ///
    /// ```rust
    /// use sunk::Sunk::*;
    /// use error::*;
    ///
    /// let sunk = Sunk::new("subsonic.example.com", "user", "password")?;
    /// let url = sunk.build_url("stream", vec![("id", 1), ("bitrate", 96)])?;
    ///
    /// assert_eq!(
    ///     url,
    ///     "https://subsonic.example.com/rest/stream \
    ///         &u=user&p=password&v=1.14.0&id=1&bitrate=96".to_string()
    /// )
    /// ```
    ///
    /// Most usage of this function will be through `Sunk::get()`.
    fn build_url<S>(&self, query: &str, args: Vec<(&str, S)>) -> Result<String>
    where
        S: ::std::fmt::Display,
    {
        let scheme = self.url
            .scheme()
            .or_else(|| {
                warn!("No scheme provided; falling back to http");
                Some("http")
            })
            .ok_or(Error::ServerError("Unable to determine scheme".into()))?;
        let addr = self.url
            .authority()
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

        Ok(url)
    }

    // fn get<'de, T>(&mut self, query: &str) -> Result<(u16, T)>
    // where
    //     T: serde::Deserialize<'de>
    pub fn get<S>(
        &mut self,
        query: &str,
        args: Vec<(&str, S)>,
    ) -> Result<(u16, json::Value)>
    where
        S: ::std::fmt::Display, // + ::std::string::ToString
    {
        use futures::{Future, Stream};

        let uri = self.build_url(query, args)?.parse().unwrap();
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

    /// Attempts to connect to the `Sunk` with the provided query and args.
    ///
    /// Returns the constructed, attempted URL on success, or an error if the
    /// Subsonic instance refuses the connection (i.e., returns a failure
    /// response).
    ///
    /// Specifically, it will succeed if `json::from_slice()` fails due to not
    /// receiving a valid JSON stream. It's assumed that the stream will be
    /// binary in this case.
    pub fn try_binary<S>(
        &mut self,
        query: &str,
        args: Vec<(&str, S)>,
    ) -> Result<String>
    where
        S: ::std::fmt::Display,
    {
        use futures::{Future, Stream};

        let raw_uri = self.build_url(query, args)?;
        let uri = raw_uri.parse().unwrap();
        let work = self.client.get(uri).and_then(|res| {
            res.body().concat2().and_then(move |b| {
                let valid_json = json::from_slice::<json::Value>(&b).is_ok();
                if !valid_json {
                    return Ok(raw_uri)
                } else {
                    return Err(hyper::Error::Method)
                }
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
            Some("ok") => {}
            Some("failed") => {
                if let Some(i) = res["error"].as_u64() {
                    return subsonic_err(
                        i,
                        TARGET_API,
                        &res["version"],
                        &res["error"]["message"],
                    )
                } else {
                    err!(format!("unexpected respone: {:?}", res))
                }
            }
            _ => unreachable!(),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use sunk::*;
    use test_util::*;

    #[test]
    fn test_try() {
        let (site, user, pass) = load_credentials().unwrap();
        let mut srv = Sunk::new(&site, &user, &pass).unwrap();
        let resp = srv.try_binary("stream", vec![("id", 0)]);
        assert!(resp.is_ok())
    }

    #[test]
    fn test_ping() {
        let (site, user, pass) = load_credentials().unwrap();
        let mut srv =
            Sunk::new(&site, &user, &pass).expect("Failed to start client");
        debug!("{:?}", srv);
        srv.check_connection().unwrap();
        assert!(srv.check_connection().is_ok())
    }
}
