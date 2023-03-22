use std::iter;

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use reqwest::Client as ReqwestClient;
use reqwest::Url;

use crate::media::NowPlaying;
use crate::query::Query;
use crate::response::Response;
use crate::search::{SearchPage, SearchResult};
use crate::{Error, Genre, Hls, Lyrics, MusicFolder, Result, UrlError, Version};

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
/// # fn run() -> sunk::Result<()> {
/// # let site = "http://demo.subsonic.org";
/// # let user = "guest3";
/// # let password = "guest";
///
/// let client = Client::new(site, user, password)?;
/// client.ping()?;
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
    /// Version that the `Client` supports.
    pub ver: Version,
    /// Version that the `Client` is targeting; currently only has an effect on
    /// the authentication method.
    pub target_ver: Version,
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
            let mut rng = thread_rng();
            let salt: String = iter::repeat(())
                .take(SALT_SIZE)
                .map(|_| char::from(rng.sample(Alphanumeric)))
                .collect();
            let pre_t = self.password.to_string() + &salt;
            let token = format!("{:x}", md5::compute(pre_t.as_bytes()));

            format!("u={u}&t={t}&s={s}", u = self.user, t = token, s = salt)
        } else {
            format!("u={u}&p={p}", u = self.user, p = self.password)
        };

        let format = "json";
        let crate_name = env!("CARGO_PKG_NAME");

        format!("{auth}&v={ver}&c={crate_name}&f={format}")
    }
}

impl Client {
    /// Constructs a client to interact with a Subsonic instance.
    pub fn new(url: &str, user: &str, password: &str) -> Result<Client> {
        let auth = SubsonicAuth::new(user, password);
        let url = url
            .parse::<Url>()
            .map_err(<url::ParseError as Into<UrlError>>::into)?;
        let ver = Version::from("1.14.0");
        let target_ver = ver;

        let reqclient = ReqwestClient::builder().build()?;

        Ok(Client {
            url,
            auth,
            reqclient,
            ver,
            target_ver,
        })
    }

    /// Adjusts the client to target a specific version.
    ///
    /// By default, the client will target version 1.14.0, as built by `sunk`.
    /// However, this means that any servers that don't implement advanced
    /// features that `sunk` does automatically, such as token-based
    /// authentication, will be incompatible. The target version allows setting
    /// an override on these features by making the client limit itself to
    /// features that the target will support.
    ///
    /// Note that (currently) the client does not provide any sanity-checking
    /// on which methods are called; attempting to access an endpoint not
    /// supported by the server will fail after the call, not before.
    pub fn with_target(self, ver: Version) -> Client {
        let mut cli = self;
        cli.target_ver = ver;
        cli
    }

    /// Internal helper function to construct a URL when the actual fetching is
    /// not required.
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_pass_by_value))]
    pub(crate) fn build_url(&self, query: &str, args: Query) -> Result<String> {
        let scheme = self.url.scheme();
        let addr = self.url.host_str().ok_or(Error::Url(UrlError::Address))?;
        let path = self.url.path();

        let mut url = [scheme, "://", addr, path, "/rest/"].concat();
        url.push_str(query);
        url.push('?');
        url.push_str(&self.auth.to_url(self.target_ver));
        url.push('&');
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
    pub(crate) async fn get(&self, query: &str, args: Query) -> Result<serde_json::Value> {
        let uri: Url = self.build_url(query, args)?.parse().unwrap();

        info!("Connecting to {}", uri);
        let res = self.reqclient.get(uri).send().await?;

        if res.status().is_success() {
            let response = res.json::<Response>().await?;
            if response.is_ok() {
                Ok(match response.into_value() {
                    Some(v) => v,
                    None => serde_json::Value::Null,
                })
            } else {
                Err(response
                    .into_error()
                    .map(|e| e.into())
                    .ok_or(Error::Other("unable to retrieve error"))?)
            }
        } else {
            Err(Error::Connection(res.status()))
        }
    }

    /// Fetches an unprocessed response from the server rather than a JSON- or
    /// XML-parsed one.
    pub(crate) async fn get_raw(&self, query: &str, args: Query) -> Result<String> {
        let uri: Url = self.build_url(query, args)?.parse().unwrap();
        let res = self.reqclient.get(uri).send().await?;
        Ok(res.text().await?)
    }

    /// Returns a response as a vector of bytes rather than serialising it.
    pub(crate) async fn get_bytes(&self, query: &str, args: Query) -> Result<Vec<u8>> {
        let uri: Url = self.build_url(query, args)?.parse().unwrap();
        let res = self.reqclient.get(uri).send().await?;
        Ok(res.bytes().await?.to_vec())
    }

    /// Returns the raw bytes of a HLS slice.
    pub async fn hls_bytes(&self, hls: &Hls) -> Result<Vec<u8>> {
        let url: Url = self
            .url
            .join(&hls.url)
            .map_err(<url::ParseError as Into<UrlError>>::into)?;
        let res = self.reqclient.get(url).send().await?;
        Ok(res.bytes().await?.to_vec())
    }

    /// Tests a connection with the server.
    pub async fn ping(&self) -> Result<()> {
        self.get("ping", Query::none()).await?;
        Ok(())
    }

    /// Get details about the software license. Note that access to the REST API
    /// requires that the server has a valid license (after a 30-day trial
    /// period). To get a license key you must upgrade to Subsonic Premium.
    ///
    /// Forks of Subsonic (Libresonic, Airsonic, etc.) do not require licenses;
    /// this method will always return a valid license and trial when attempting
    /// to connect to these services.
    pub async fn check_license(&self) -> Result<License> {
        let res = self.get("getLicense", Query::none()).await?;
        Ok(serde_json::from_value::<License>(res)?)
    }

    /// Initiates a rescan of the media libraries.
    ///
    /// # Note
    ///
    /// This method was introduced in version 1.15.0. It will not be supported
    /// on servers with earlier versions of the Subsonic API.
    pub async fn scan_library(&self) -> Result<()> {
        self.get("startScan", Query::none()).await?;
        Ok(())
    }

    /// Gets the status of a scan. Returns the current status for media library
    /// scanning.
    ///
    /// # Note
    ///
    /// This method was introduced in version 1.15.0. It will not be supported
    /// on servers with earlier versions of the Subsonic API.
    pub async fn scan_status(&self) -> Result<(bool, u64)> {
        let res = self.get("getScanStatus", Query::none()).await?;

        #[derive(Deserialize)]
        struct ScanStatus {
            count: u64,
            scanning: bool,
        }
        let sc = serde_json::from_value::<ScanStatus>(res)?;

        Ok((sc.scanning, sc.count))
    }

    /// Returns all configured top-level music folders.
    pub async fn music_folders(&self) -> Result<Vec<MusicFolder>> {
        #[allow(non_snake_case)]
        let music_folder = self.get("getMusicFolders", Query::none()).await?;

        Ok(get_list_as!(music_folder, MusicFolder))
    }

    /// Returns all genres.
    pub async fn genres(&self) -> Result<Vec<Genre>> {
        let genre = self.get("getGenres", Query::none()).await?;

        Ok(get_list_as!(genre, Genre))
    }

    /// Returns all currently playing media on the server.
    pub async fn now_playing(&self) -> Result<Vec<NowPlaying>> {
        let entry = self.get("getNowPlaying", Query::none()).await?;
        Ok(get_list_as!(entry, NowPlaying))
    }

    /// Searches for lyrics matching the artist and title. Returns `None` if no
    /// lyrics are found.
    pub async fn lyrics<'a, S>(&self, artist: S, title: S) -> Result<Option<Lyrics>>
    where
        S: Into<Option<&'a str>>,
    {
        let args = Query::with("artist", artist.into())
            .arg("title", title.into())
            .build();
        let res = self.get("getLyrics", args).await?;

        if res.get("value").is_some() {
            Ok(Some(serde_json::from_value(res)?))
        } else {
            Ok(None)
        }
    }

    /// Returns albums, artists and songs matching the given search criteria.
    /// Supports paging through the result. See the [search module] for
    /// documentation.
    ///
    /// [search module]: ./search/index.html
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_run
    /// use sunk::search::{self, SearchPage};
    /// use sunk::Client;
    ///
    /// # fn run() -> sunk::Result<()> {
    /// # let site = "http://demo.subsonic.org";
    /// # let user = "guest3";
    /// # let password = "guest";
    /// let client = Client::new(site, user, password)?;
    ///
    /// let search_size = SearchPage::new();
    /// let ignore = search::NONE;
    ///
    /// let result = client.search("smile", ignore, ignore, search_size)?;
    ///
    /// assert!(result.artists.is_empty());
    /// assert!(result.albums.is_empty());
    /// assert!(!result.songs.is_empty());
    /// # Ok(())
    /// # }
    /// # fn main() { }
    /// ```
    pub async fn search(
        &self,
        query: &str,
        artist_page: SearchPage,
        album_page: SearchPage,
        song_page: SearchPage,
    ) -> Result<SearchResult> {
        // FIXME There has to be a way to make this nicer.
        let args = Query::with("query", query)
            .arg("artistCount", artist_page.count)
            .arg("artistOffset", artist_page.offset)
            .arg("albumCount", album_page.count)
            .arg("albumOffset", album_page.offset)
            .arg("songCount", song_page.count)
            .arg("songOffset", song_page.offset)
            .build();

        let res = self.get("search3", args).await?;
        Ok(serde_json::from_value::<SearchResult>(res)?)
    }

    /// Returns a list of all starred artists, albums, and songs.
    pub async fn starred<U>(&self, folder_id: U) -> Result<SearchResult>
    where
        U: Into<Option<usize>>,
    {
        let res = self
            .get("getStarred", Query::with("musicFolderId", folder_id.into()))
            .await?;
        Ok(serde_json::from_value::<SearchResult>(res)?)
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
    use crate::test_util;

    #[test]
    fn test_token_auth() {
        let cli = test_util::demo_site().unwrap();
        let token_addr = cli.build_url("ping", Query::none()).unwrap();
        let legacy_cli = cli.with_target("1.8.0".into());
        let legacy_addr = legacy_cli.build_url("ping", Query::none()).unwrap();

        assert!(token_addr != legacy_addr);
        assert_eq!(
            legacy_addr,
            "http://demo.subsonic.org/rest/ping?u=guest3&p=guest&v=1.8.0&c=sunk&f=json&"
        );
    }

    #[test]
    fn demo_ping() {
        let cli = test_util::demo_site().unwrap();
        tokio_test::block_on(async {
            cli.ping().await.unwrap();
        });
    }

    #[test]
    fn demo_license() {
        let cli = test_util::demo_site().unwrap();
        let license = tokio_test::block_on(async { cli.check_license().await.unwrap() });

        assert!(license.valid);
        assert_eq!(license.email, String::from("demo@subsonic.org"));
    }

    #[test]
    fn demo_scan_status() {
        let cli = test_util::demo_site().unwrap();
        let (status, n) = tokio_test::block_on(async { cli.scan_status().await.unwrap() });
        assert!(!status);
        assert_eq!(n, 525);
    }

    #[test]
    fn demo_search() {
        let cli = test_util::demo_site().unwrap();
        let s = SearchPage::new().with_size(1);
        let r = tokio_test::block_on(async { cli.search("dada", s, s, s).await.unwrap() });

        assert_eq!(r.artists[0].id, 14);
        assert_eq!(r.artists[0].name, String::from("The Dada Weatherman"));
        assert_eq!(r.artists[0].album_count, 4);

        assert_eq!(r.albums[0].id, 23);
        assert_eq!(r.albums[0].name, String::from("The Green Waltz"));

        assert_eq!(r.songs[0].id, 222);

        // etc.
    }
}
