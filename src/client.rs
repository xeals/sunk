use reqwest::Client as ReqwestClient;
use reqwest::Url;
use serde_json;

use {Album, Artist, Error, Genre, Lyrics, MusicFolder, Result, Song, UrlError,
     Version};
use media::NowPlaying;
use query::Query;
use response::Response;
use search::SearchPage;

const SALT_SIZE: usize = 36; // Minimum 6 characters.

/// A client to make requests to a Subsonic instance.
///
/// The `Client` holds an internal connection pool and stores authentication
/// details. It is highly recommended to re-use a `Client` where possible rather
/// than creating a new one each time it is required.
///
/// # Examples
///
/// Basic usage:
///
/// ```no_run
/// use sunk::Client;
/// # fn run() -> Result<(), sunk::error::Error> {
/// # let site = "http://demo.subsonic.org";
/// # let user = "guest3";
/// # let password = "guest";
///
/// let mut server = Client::new(site, user, password)?;
/// server.ping()?;
/// # Ok(())
/// # }
/// ```
///
/// # Notes
///
/// Generally, any method that requires a response from a Subsonic server will
/// require a `Client` . Any method that issues a request will have the
/// possiblity to return an error. A request will result in an error if any of
/// the following occurs:
///
/// - the `Client` is built with an unrecognised URL
/// - connecting to the Subsonic server fails
/// - the Subsonic server returns an [API error]
///
/// [API error]: ./enum.ApiError.html
#[derive(Debug)]
pub struct Client {
    url: Url,
    auth: SubsonicAuth,
    reqclient: ReqwestClient,
    pub ver: Version,
}

#[derive(Debug)]
struct SubsonicAuth {
    user: String,
    password: String,
}

impl SubsonicAuth {
    fn new(user: &str, password: &str) -> SubsonicAuth {
        SubsonicAuth {
            user: user.into(),
            password: password.into(),
        }
    }

    fn to_url(&self, ver: Version) -> String {
        // First md5 support.
        let auth = if ver >= "1.13.0".into() {
            use md5;
            use rand::{thread_rng, Rng};

            let salt: String =
                thread_rng().gen_ascii_chars().take(SALT_SIZE).collect();
            let pre_t = self.password.to_string() + &salt;
            let token = format!("{:x}", md5::compute(pre_t.as_bytes()));

            format!("u={u}&t={t}&s={s}", u = self.user, t = token, s = salt)
        } else {
            format!("u={u}&p={p}", u = self.user, p = self.password)
        };

        let format = "json";
        let crate_name = env!("CARGO_PKG_NAME");

        format!(
            "{auth}&v={v}&c={c}&f={f}",
            auth = auth,
            v = ver,
            c = crate_name,
            f = format
        )
    }
}

impl Client {
    /// Constructs a client to interact with a Subsonic instance.
    pub fn new(url: &str, user: &str, password: &str) -> Result<Client> {
        let auth = SubsonicAuth::new(user, password);
        let url = url.parse::<Url>()?;
        let ver = Version::from("1.14.0");

        let reqclient = ReqwestClient::builder().build()?;

        Ok(Client {
            url,
            auth,
            reqclient,
            ver,
        })
    }

    /// Internal helper function to construct a URL when the actual fetching is
    /// not required.
    #[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
    pub(crate) fn build_url(&self, query: &str, args: Query) -> Result<String> {
        let scheme = self.url.scheme();
        let addr = self.url
            .host_str()
            .ok_or_else(|| Error::Url(UrlError::Address))?;

        let mut url = [scheme, "://", addr, "/rest/"].concat();
        url.push_str(query);
        url.push_str("?");
        url.push_str(&self.auth.to_url(self.ver));
        url.push_str("&");
        url.push_str(&args.to_string());

        Ok(url)
    }

    /// Issues a request to the Subsonic server.
    ///
    /// A query should be one documented in the [official API].
    ///
    /// [official API]: http://www.subsonic.org/pages/api.jsp
    ///
    /// # Errors
    ///
    /// Will return an error if any of the following occurs:
    ///
    /// - server is built with an incomplete URL
    /// - connecting to the server fails
    /// - the server returns an API error
    pub(crate) fn get(
        &self,
        query: &str,
        args: Query,
    ) -> Result<serde_json::Value> {
        let uri: Url = self.build_url(query, args)?.parse().unwrap();

        info!("Connecting to {}", uri);
        let mut res = self.reqclient.get(uri).send()?;

        if res.status().is_success() {
            let response = res.json::<Response>()?;
            if response.is_ok() {
                Ok(match response.into_value() {
                    Some(v) => v,
                    None => serde_json::Value::Null,
                })
            } else {
                Err(response
                    .into_error()
                    .map(|e| e.into())
                    .ok_or_else(|| Error::Other("unable to retrieve error"))?)
            }
        } else {
            Err(Error::Connection(res.status()))
        }
    }

    /// Fetches an unprocessed response from the server rather than a JSON- or
    /// XML-parsed one.
    pub(crate) fn get_raw(&self, query: &str, args: Query) -> Result<String> {
        let uri: Url = self.build_url(query, args)?.parse().unwrap();
        let mut res = self.reqclient.get(uri).send()?;
        Ok(res.text()?)
    }

    /// Returns a response as a vector of bytes rather than serialising it.
    pub(crate) fn get_bytes(
        &self,
        query: &str,
        args: Query,
    ) -> Result<Vec<u8>> {
        use std::io::Read;
        let uri: Url = self.build_url(query, args)?.parse().unwrap();
        let res = self.reqclient.get(uri).send()?;
        Ok(res.bytes().map(|b| b.unwrap()).collect())
    }

    /// Tests a connection with the server.
    pub fn ping(&self) -> Result<()> {
        self.get("ping", Query::none())?;
        Ok(())
    }

    /// Get details about the software license. Note that access to the REST API
    /// requires that the server has a valid license (after a 30-day trial
    /// period). To get a license key you must upgrade to Subsonic Premium.
    ///
    /// Forks of Subsonic (Libresonic, Airsonic, etc.) do not require licenses;
    /// this method will always return a valid license and trial when attempting
    /// to connect to these services.
    pub fn check_license(&self) -> Result<License> {
        let res = self.get("getLicense", Query::none())?;
        Ok(serde_json::from_value::<License>(res)?)
    }

    /// Initiates a rescan of the media libraries.
    ///
    /// # Note
    ///
    /// This method was introduced in version 1.15.0. It will not be supported
    /// on servers with earlier versions of the Subsonic API.
    pub fn scan_library(&self) -> Result<()> {
        self.get("startScan", Query::none())?;
        Ok(())
    }

    /// Gets the status of a scan. Returns the current status for media library
    /// scanning.
    ///
    /// # Note
    ///
    /// This method was introduced in version 1.15.0. It will not be supported
    /// on servers with earlier versions of the Subsonic API.
    pub fn scan_status(&self) -> Result<(bool, u64)> {
        let res = self.get("getScanStatus", Query::none())?;

        #[derive(Deserialize)]
        struct ScanStatus {
            count: u64,
            scanning: bool,
        }
        let sc = serde_json::from_value::<ScanStatus>(res)?;

        Ok((sc.scanning, sc.count))
    }

    /// Returns all configured top-level music folders.
    pub fn music_folders(&self) -> Result<Vec<MusicFolder>> {
        #[allow(non_snake_case)]
        let musicFolder = self.get("getMusicFolders", Query::none())?;

        Ok(get_list_as!(musicFolder, MusicFolder))
    }

    /// Returns all genres.
    pub fn genres(&self) -> Result<Vec<Genre>> {
        let genre = self.get("getGenres", Query::none())?;

        Ok(get_list_as!(genre, Genre))
    }

    /// Returns all currently playing media on the server.
    pub fn now_playing(&self) -> Result<Vec<NowPlaying>> {
        let entry = self.get("getNowPlaying", Query::none())?;
        Ok(get_list_as!(entry, NowPlaying))
    }

    /// Searches for lyrics matching the artist and title. Returns `None` if no
    /// lyrics are found.
    pub fn lyrics<'a, S>(&self, artist: S, title: S) -> Result<Option<Lyrics>>
    where
        S: Into<Option<&'a str>>,
    {
        let args = Query::with("artist", artist.into())
            .arg("title", title.into())
            .build();
        let res = self.get("getLyrics", args)?;

        if res.get("value").is_some() {
            Ok(Some(serde_json::from_value(res)?))
        } else {
            Ok(None)
        }
    }

    // TODO Sort out what's happening with the library stuff
    // TODO Make a SearchResult struct to handle this properly
    /// Returns albums, artists and songs matching the given search criteria.
    /// Supports paging through the result.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_run
    /// use sunk::Client;
    /// use sunk::library::search;
    ///
    /// # fn run() -> sunk::error::Result<()> {
    /// # let site = "http://demo.subsonic.org";
    /// # let user = "guest3";
    /// # let password = "guest";
    /// let mut server = Client::new(site, user, password)?;
    ///
    /// let search_size = search::SearchPage::new();
    /// let ignore = search::NONE;
    ///
    /// let (artists, albums, songs) = server.search("smile", ignore, ignore, search_size)?;
    ///
    /// assert!(artists.is_empty());
    /// assert!(albums.is_empty());
    /// assert!(!songs.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Notes
    ///
    /// The current implementation uses the `search3` method, introduced in
    /// version 1.8.0. This supports organising results by their ID3 tags,
    /// and paging through results.
    pub fn search(
        &self,
        query: &str,
        artist_page: SearchPage,
        album_page: SearchPage,
        song_page: SearchPage,
    ) -> Result<(Vec<Artist>, Vec<Album>, Vec<Song>)> {
        // FIXME There has to be a way to make this nicer.
        let args = Query::with("query", query)
            .arg("artistCount", artist_page.count)
            .arg("artistOffset", artist_page.offset)
            .arg("albumCount", album_page.count)
            .arg("albumOffset", album_page.offset)
            .arg("songCount", song_page.count)
            .arg("songOffset", song_page.offset)
            .build();

        let res = self.get("search3", args)?;

        #[derive(Deserialize)]
        struct Output {
            artist: Vec<Artist>,
            album: Vec<Album>,
            song: Vec<Song>,
        }

        let result = serde_json::from_value::<Output>(res)?;
        Ok((result.artist, result.album, result.song))
    }

    /// Returns a list of all starred artists, albums, and songs.
    pub fn starred<U>(
        &self,
        folder_id: U,
    ) -> Result<(Vec<Artist>, Vec<Album>, Vec<Song>)>
    where
        U: Into<Option<usize>>,
    {
        let res = self.get(
            "getStarred",
            Query::with("musicFolderId", folder_id.into()),
        )?;

        #[derive(Deserialize)]
        struct Output {
            artist: Vec<Artist>,
            album: Vec<Album>,
            song: Vec<Song>,
        }

        let result = serde_json::from_value::<Output>(res)?;
        Ok((result.artist, result.album, result.song))
    }
}

/// A representation of a license associated with a server.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct License {
    /// Whether the license is valid or not.
    pub valid: bool,
    /// The email associated with the email.
    pub email: String,
    /// An ISO8601 timestamp of the server's trial expiry.
    pub trial_expires: Option<String>,
    /// An ISO8601 timestamp of the server's license expiry. Servers still in
    /// the trial phase typically will not have this field.
    pub license_expires: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_util;

    #[test]
    fn demo_ping() {
        let mut srv = test_util::demo_site().unwrap();
        srv.ping().unwrap();
    }

    #[test]
    fn demo_license() {
        let mut srv = test_util::demo_site().unwrap();
        let license = srv.check_license().unwrap();

        assert!(license.valid);
        assert_eq!(license.email, String::from("demo@subsonic.org"));
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
