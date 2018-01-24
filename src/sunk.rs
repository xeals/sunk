// use url;
use hyper::{self, Client, Uri};
use hyper_tls::HttpsConnector;
use serde_json;
use tokio;

use api::Api;
use error::*;
use query::Query;
use library;
use album;
use artist;
use song;

const SALT_SIZE: usize = 36; // Minimum 6 characters.

#[derive(Debug)]
pub struct Sunk {
    url: Uri,
    auth: SunkAuth,
    client: Client<HttpsConnector<hyper::client::HttpConnector>>,
    core: tokio::reactor::Core,
    api: Api,
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
    fn as_uri(&self, api: Api) -> String {
        // First md5 support.
        let auth = if api >= "1.13.0".into() {
            use md5;
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
        let format = if api >= "1.14.0".into() {
            "json"
        } else {
            "xml"
        };

        let crate_name = ::std::env::var("CARGO_PKG_NAME").unwrap();

        format!(
            "{auth}&v={v}&c={c}&f={f}",
            auth = auth,
            v = api,
            c = crate_name,
            f = format
        )
    }
}

impl Sunk {
    pub fn new(url: &str, user: &str, password: &str) -> Result<Sunk> {
        use std::str::FromStr;

        let auth = SunkAuth::new(user, password);
        let uri =
            Uri::from_str(url).map_err(|e| Error::Uri(UriError::Hyper(e)))?;
        let api = Api::from("1.14.0");

        let core = tokio::reactor::Core::new()?;
        let handle = core.handle();
        let client = Client::configure()
            .connector(HttpsConnector::new(4, &handle)
                .map_err(|_| Error::Other("Unable to use secure conection"))?)
            .build(&handle);

        Ok(Sunk {
            url: uri,
            auth: auth,
            client: client,
            core: core,
            api: api,
        })
    }

    /// Internal helper function to construct a URL when the actual fetching is
    /// not required.
    ///
    /// Formats arguments in a standard HTTP format, using information from the
    /// `Sunk`; for example:
    ///
    /// ```rust,norun
    /// # use sunk::Sunk;
    /// # use error::*;
    /// # use query::Query;
    ///
    /// let sunk = Sunk::new("demo.subsonic.com", "guest3", "guest")?;
    /// let url = sunk.build_url("stream", Query::with("id", 222))?;
    ///
    /// assert_eq!(
    ///     url.as_str(),
    ///     "http://demo.subsonic.com/rest/stream \
    ///         &u=guest3&t=XXXX&s=XXXXX&v=1.14.0&id=222"))
    /// ```
    ///
    /// Most usage of this function will be through `Sunk::get()`.
    #[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
    fn build_url<'a, D>(
        &self,
        query: &str,
        args: Query<'a, D>,
    ) -> Result<String>
    where
        D: ::std::fmt::Display,
    {
        let scheme = self.url
            .scheme()
            .or_else(|| {
                warn!("No scheme provided; falling back to http");
                Some("http")
            })
            .ok_or_else(|| Error::Uri(UriError::Scheme))?;
        let addr = self.url
            .authority()
            .ok_or_else(|| Error::Uri(UriError::Address))?;

        let mut url = [scheme, "://", addr, "/rest/"].concat();
        url.push_str(query);
        url.push_str("?");
        url.push_str(&self.auth.as_uri(self.api));
        url.push_str("&");
        url.push_str(&args.to_string());

        Ok(url)
    }

    /// Issues a request to the `sunk` server.
    ///
    /// A query should be one documented in the [official API].
    ///
    /// [official API]: http://www.subsonic.org/pages/api.jsp
    ///
    /// # Errors
    ///
    /// Will return an error if any of the following occurs:
    ///
    /// - server is build with an incomplete URL
    /// - connecting to the server fails
    /// - the server returns an API error
    pub fn get<'a, D>(
        &mut self,
        query: &str,
        args: Query<'a, D>,
    ) -> Result<serde_json::Value>
    where
        D: ::std::fmt::Display,
    {
        use futures::{Future, Stream};

        let uri = self.build_url(query, args)?.parse().unwrap();

        info!("Connecting to {}", uri);
        let work = self.client.get(uri).and_then(|res| {
            let status = res.status();
            info!("Received `{}` for request /{}?", status, query);

            res.body().concat2().and_then(move |body| {
                let v: serde_json::Value = serde_json::from_slice(&body).map_err(|e| {
                    use std::io;
                    io::Error::new(io::ErrorKind::Other, e)
                })?;
                Ok((status, v))
            })
        });

        let (status, res): (hyper::StatusCode, serde_json::Value) =
            self.core.run(work)?;
        if status.is_success() {
            if let Some(out) = res.get("subsonic-response") {
                match out["status"].as_str() {
                    Some("ok") => {
                        if query == "ping" {
                            return Ok(serde_json::Value::Null)
                        }

                        let out = out.as_object().unwrap();
                        for (k, v) in out {
                            if k != "status" && k != "version" {
                                return Ok(v.clone())
                            }
                        }
                        unreachable!()
                    }
                    Some("failed") => {
                        return Err(Error::Api(ApiError::try_from(out)?))
                    }
                    _ => panic!(),
                }
            } else {
                panic!()
            }
        } else {
            Err(Error::ConnectionError(status))
        }
    }

    /// Attempts to connect to the `Sunk` with the provided query and args.
    ///
    /// Returns the constructed, attempted URL on success, or an error if the
    /// Subsonic instance refuses the connection (i.e., returns a failure
    /// response).
    ///
    /// Specifically, it will succeed if `serde_json::from_slice()` fails due to not
    /// receiving a valid JSON stream. It's assumed that the stream will be
    /// binary in this case.
    pub fn try_binary<'a, D>(
        &mut self,
        query: &str,
        args: Query<'a, D>,
    ) -> Result<String>
    where
        D: ::std::fmt::Display,
    {
        use futures::{Future, Stream};

        let raw_uri = self.build_url(query, args)?;
        let uri = raw_uri.parse().unwrap();

        info!("Connecting to {}", uri);
        let work = self.client.get(uri).and_then(|res| {
            res.body().concat2().and_then(move |b| {
                let valid_json = serde_json::from_slice::<serde_json::Value>(&b).is_ok();
                if !valid_json {
                    Ok(raw_uri)
                } else {
                    Err(hyper::Error::Method)
                }
            })
        });

        Ok(self.core.run(work)?)
    }

    pub fn get_raw<'a, D>(
        &mut self,
        query: &str,
        args: Query<'a, D>,
    ) -> Result<String>
    where
        D: ::std::fmt::Display,
    {
        use futures::{Future, Stream};

        let uri = self.build_url(query, args)?.parse().unwrap();

        info!("Connecting to {}", uri);
        let work = self.client.get(uri).and_then(|res| res.body().concat2());

        let get = self.core.run(work)?;
        String::from_utf8(get.to_vec())
            .map_err(|_| Error::Other("Unable to parse stream as UTF-8"))
    }

    /// Attempts to connect to the server with the provided credentials.
    fn check_connection(&mut self) -> Result<()> {
        self.get("ping", Query::with("", "")).map(|_| ())
    }

    fn check_license(&mut self) -> Result<License> {
        serde_json::from_value::<License>(self.get("getLicense", Query::with("", ""))?)
            .map_err(|e| e.into())
    }

    /// Starts a library scan.
    pub fn scan_library(&mut self) -> Result<()> {
        self.get("startScan", Query::with("", ""))?;
        Ok(())
    }

    /// Gets the status of a scan. Returns whether or not the scan is currently
    /// running, and the number of media items found.
    pub fn scan_status(&mut self) -> Result<(bool, u64)> {
        let res = self.get("getScanStatus", Query::with("", ""))?;

        println!("{}", res);
        if let Some(status) = res["scanning"].as_bool() {
            if let Some(count) = res["count"].as_u64() {
                Ok((status, count))
            } else {
                unreachable!()
            }
        } else {
            unreachable!()
        }
    }

    pub fn music_folders(&mut self) -> Result<Vec<library::MusicFolder>> {
        let res = self.get("musicFolders", Query::with("", ""))?;
        let mut folders = Vec::new();
        if let Some(Some(list)) = res.get("musicFolder").map(|r| r.as_array()) {
            for folder in list {
                folders.push(serde_json::from_value::<library::MusicFolder>(folder.clone())?);
            }
        }

        Ok(folders)
    }

    pub fn genres(&mut self) -> Result<Vec<library::Genre>> {
        let res = self.get("getGenres", Query::with("", ""))?;
        let mut genres = Vec::new();
        if let Some(Some(list)) = res.get("genres").map(|r| r.as_array()) {
            for genre in list {
                genres.push(serde_json::from_value::<library::Genre>(genre.clone())?);
            }
        }

        Ok(genres)
    }

    pub fn search(
        &mut self,
        query: &str,
        artist_page: library::search::SearchPage,
        album_page: library::search::SearchPage,
        song_page: library::search::SearchPage,
    ) -> Result<(Vec<artist::Artist>, Vec<album::Album>, Vec<song::Song>)>
    {
        // FIXME There has to be a way to make this nicer.
        let args = Query::with("query", query.to_string())
            .arg("artistCount", artist_page.count.to_string())
            .arg("artistOffset", artist_page.offset.to_string())
            .arg("albumCount", album_page.count.to_string())
            .arg("albumOffset", album_page.offset.to_string())
            .arg("songCount", song_page.count.to_string())
            .arg("songOffset", song_page.offset.to_string())
            .build();

        // TODO `search` was deprecated in 1.4.0 in favour of `search2`, and
        // `search3` organises using ID3 tags over `search2`, implemented in
        // 1.8.0. `search` uses a different query set to to other two calls, and
        // `search2` and `search3` return different fields for artists and
        // albums. This should be supported eventually using a conditional
        // compilation, probably on a search module.
        let res = self.get("search3", args)?;

        macro_rules! vec_of {
            ($t:ident, $str:ident) => ({
                let mut v = Vec::new();
                if let Some(Some(list)) = res.get(stringify!($t))
                    .map(|v| v.as_array())
                {
                    for item in list {
                        v.push(serde_json::from_value::<$t::$str>(item.clone())?);
                    }
                }
                v
            })
        };

        let artists = vec_of!(artist, Artist);
        let albums = vec_of!(album, Album);
        let songs = vec_of!(song, Song);

        Ok((artists, albums, songs))
    }
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct License {
    valid: bool,
    email: String,
    licenseExpires: String
}

#[cfg(test)]
mod tests {
    use sunk::*;
    use test_util;

    #[test]
    fn demo_ping() {
        let mut srv = ::test_util::demo_site().unwrap();
        srv.check_connection().unwrap();
    }

    #[test]
    fn demo_try_binary() {
        let mut srv = test_util::demo_site().unwrap();
        let res = srv.try_binary("stream", Query::with("id", 189));
        assert!(res.is_ok())
    }

    #[test]
    fn demo_scan_status() {
        let mut srv = test_util::demo_site().unwrap();
        let (status, n) = srv.scan_status().unwrap();
        assert_eq!(status, false);
        assert_eq!(n, 521);
    }

    #[test]
    fn demo_search() {
        use library::search;

        let mut srv = test_util::demo_site().unwrap();
        let s = search::SearchPage::new().with_size(1);
        let (art, alb, son) = srv.search("dada", s, s, s).unwrap();

        assert_eq!(art[0].id, 14);
        assert_eq!(art[0].name, String::from("The Dada Weatherman"));
        assert_eq!(art[0].album_count, 4);

        assert_eq!(alb[0].id, 23);
        assert_eq!(alb[0].name, String::from("The Green Waltz"));

        assert_eq!(son[0].id, 222);

        // etc.
    }
}
