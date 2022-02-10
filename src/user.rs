use query::Query;
use serde_json;
use {Client, Result};

/// A struct representing a Subsonic user.
#[derive(Debug, Deserialize)]
pub struct User {
    /// A user's name.
    pub username: String,
    /// A user's email address.
    pub email: String,
    /// A user may be limited to the bit rate of media they may stream. Any
    /// higher sampled media will be downsampled to their limit. A limit of `0`
    /// disables this.
    #[serde(rename = "maxBitRate")]
    #[serde(default)]
    pub max_bit_rate: u64,
    /// Whether the user is allowed to scrobble their songs to last.fm.
    #[serde(rename = "scrobblingEnabled")]
    pub scrobbling_enabled: bool,
    /// Whether the user is authenticated in LDAP.
    #[serde(rename = "ldapAuthenticated")]
    #[serde(default)]
    pub ldap_authenticated: bool,
    /// Whether the user is an administrator.
    #[serde(rename = "adminRole")]
    pub admin_role: bool,
    /// Whether the user is allowed to manage their own settings and change
    /// their password.
    #[serde(rename = "settingsRole")]
    pub settings_role: bool,
    /// Whether the user is allowed to download media.
    #[serde(rename = "downloadRole")]
    pub download_role: bool,
    /// Whether the user is allowed to upload media.
    #[serde(rename = "uploadRole")]
    pub upload_role: bool,
    /// Whether the user is allowed to modify or delete playlists.
    #[serde(rename = "playlistRole")]
    pub playlist_role: bool,
    /// Whether the user is allowed to change cover art and media tags.
    #[serde(rename = "coverArtRole")]
    pub cover_art_role: bool,
    /// Whether the user is allowed to create and edit comments and
    /// ratings.
    #[serde(rename = "commentRole")]
    pub comment_role: bool,
    /// Whether the user is allowed to administrate podcasts.
    #[serde(rename = "podcastRole")]
    pub podcast_role: bool,
    /// Whether the user is allowed to play media.
    #[serde(rename = "streamRole")]
    pub stream_role: bool,
    /// Whether the user is allowed to control the jukebox.
    #[serde(rename = "jukeboxRole")]
    pub jukebox_role: bool,
    /// Whether the user is allowed to share content.
    #[serde(rename = "shareRole")]
    pub share_role: bool,
    /// Whether the user is allowed to start video conversions.
    #[serde(rename = "videoConversionRole")]
    pub video_conversion_role: bool,
    /// The date the user's avatar was last changed (as an ISO8601
    /// timestamp).
    #[serde(rename = "avatarLastChanged")]
    pub avatar_last_changed: String,
    /// The list of media folders the user has access to.
    #[serde(rename = "folder")]
    pub folders: Vec<u64>,
    #[serde(default)]
    _private: bool,
}

impl User {
    /// Fetches a single user's information from the server.
    pub fn get(client: &Client, username: &str) -> Result<User> {
        let res = client.get("getUser", Query::with("username", username))?;
        Ok(serde_json::from_value::<User>(res)?)
    }

    /// Lists all users on the server.
    ///
    /// # Errors
    ///
    /// Attempting to use this method as a non-administrative user (when
    /// creating the `Client`) will result in a [`NotAuthorized`] error.
    ///
    /// [`NotAuthorized`]: ./enum.ApiError.html#variant.NotAuthorized
    pub fn list(client: &Client) -> Result<Vec<User>> {
        let user = client.get("getUsers", Query::none())?;
        Ok(get_list_as!(user, User))
    }

    /// Changes the user's password.
    ///
    /// # Errors
    ///
    /// A user may only change their own password, and only if they have the
    /// `settings_role` permission, unless they are an administrator.
    pub fn change_password(&self, client: &Client, password: &str) -> Result<()> {
        let args = Query::with("username", self.username.as_str())
            .arg("password", password)
            .build();
        client.get("changePassword", args)?;
        Ok(())
    }

    /// Returns the user's avatar image as a collection of bytes.
    ///
    /// The method makes no guarantee as to the encoding of the image, but does
    /// guarantee that it is a valid image file.
    pub fn avatar(&self, client: &Client) -> Result<Vec<u8>> {
        client.get_bytes("getAvatar", Query::with("username", self.username.as_str()))
    }

    /// Creates a new local user to be pushed to the server.
    ///
    /// See the [`UserBuilder`] struct for more details.
    ///
    /// [`UserBuilder`]: struct.UserBuilder.html
    pub fn create(username: &str, password: &str, email: &str) -> UserBuilder {
        UserBuilder::new(username, password, email)
    }

    /// Removes the user from the Subsonic server.
    pub fn delete(&self, client: &Client) -> Result<()> {
        client.get(
            "deleteUser",
            Query::with("username", self.username.as_str()),
        )?;
        Ok(())
    }

    /// Pushes any changes made to the user to the server.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// extern crate sunk;
    /// use sunk::{Client, User};
    ///
    /// # fn run() -> sunk::Result<()> {
    /// let client = Client::new("http://demo.subsonic.org", "guest3", "guest")?;
    /// let mut user = User::get(&client, "guest")?;
    ///
    /// // Update email
    /// user.email = "user@example.com".to_string();
    /// // Disable commenting
    /// user.comment_role = false;
    /// // Update on server
    /// user.update(&client)?;
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// #     run().unwrap();
    /// # }
    /// ```
    pub fn update(&self, client: &Client) -> Result<()> {
        let args = Query::with("username", self.username.as_ref())
            .arg("email", self.email.as_ref())
            .arg("ldapAuthenticated", self.ldap_authenticated)
            .arg("adminRole", self.admin_role)
            .arg("settingsRole", self.settings_role)
            .arg("streamRole", self.stream_role)
            .arg("jukeboxRole", self.jukebox_role)
            .arg("downloadRole", self.download_role)
            .arg("uploadRole", self.upload_role)
            .arg("coverArt_role", self.cover_art_role)
            .arg("commentRole", self.comment_role)
            .arg("podcastRole", self.podcast_role)
            .arg("shareRole", self.share_role)
            .arg("videoConversionRole", self.video_conversion_role)
            .arg_list("musicFolderId", &self.folders.clone())
            .arg("maxBitRate", self.max_bit_rate)
            .build();
        client.get("updateUser", args)?;
        Ok(())
    }
}

/// A new user to be created.
#[derive(Clone, Debug, Default)]
pub struct UserBuilder {
    username: String,
    password: String,
    email: String,
    ldap_authenticated: bool,
    admin_role: bool,
    settings_role: bool,
    stream_role: bool,
    jukebox_role: bool,
    download_role: bool,
    upload_role: bool,
    cover_art_role: bool,
    comment_role: bool,
    podcast_role: bool,
    share_role: bool,
    video_conversion_role: bool,
    folders: Vec<u64>,
    max_bit_rate: u64,
}

macro_rules! build {
    ($f:ident: $t:ty) => {
        pub fn $f(&mut self, $f: $t) -> &mut UserBuilder {
            self.$f = $f.into();
            self
        }
    };
}

impl UserBuilder {
    /// Begins creating a new user.
    fn new(username: &str, password: &str, email: &str) -> UserBuilder {
        UserBuilder {
            username: username.to_string(),
            password: password.to_string(),
            email: email.to_string(),
            ..UserBuilder::default()
        }
    }

    /// Sets the user's username.
    build!(username: &str);
    /// Sets the user's password.
    build!(password: &str);
    /// Set's the user's email.
    build!(email: &str);
    /// Enables LDAP authentication for the user.
    build!(ldap_authenticated: bool);
    /// Bestows admin rights onto the user.
    build!(admin_role: bool);
    /// Allows the user to change personal settings and their own password.
    build!(settings_role: bool);
    /// Allows the user to play files.
    build!(stream_role: bool);
    /// Allows the user to play files in jukebox mode.
    build!(jukebox_role: bool);
    /// Allows the user to download files.
    build!(download_role: bool);
    /// Allows the user to upload files.
    build!(upload_role: bool);
    /// Allows the user to change cover art and tags.
    build!(cover_art_role: bool);
    /// Allows the user to create and edit comments and ratings.
    build!(comment_role: bool);
    /// Allows the user to administrate podcasts.
    build!(podcast_role: bool);
    /// Allows the user to share files with others.
    build!(share_role: bool);
    /// Allows the user to start video coversions.
    build!(video_conversion_role: bool);
    /// IDs of the music folders the user is allowed to access.
    build!(folders: &[u64]);
    /// The maximum bit rate (in Kbps) the user is allowed to stream at. Higher
    /// bit rate streams will be downsampled to their limit.
    build!(max_bit_rate: u64);

    /// Pushes a defined new user to the Subsonic server.
    pub fn create(&self, client: &Client) -> Result<()> {
        let args = Query::with("username", self.username.as_ref())
            .arg("password", self.password.as_ref())
            .arg("email", self.email.as_ref())
            .arg("ldapAuthenticated", self.ldap_authenticated)
            .arg("adminRole", self.admin_role)
            .arg("settingsRole", self.settings_role)
            .arg("streamRole", self.stream_role)
            .arg("jukeboxRole", self.jukebox_role)
            .arg("downloadRole", self.download_role)
            .arg("uploadRole", self.upload_role)
            .arg("coverArt_role", self.cover_art_role)
            .arg("commentRole", self.comment_role)
            .arg("podcastRole", self.podcast_role)
            .arg("shareRole", self.share_role)
            .arg("videoConversionRole", self.video_conversion_role)
            .arg_list("musicFolderId", &self.folders)
            .arg("maxBitRate", self.max_bit_rate)
            .build();
        client.get("createUser", args)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use test_util;

    use super::*;

    #[test]
    fn remote_parse_user() {
        let mut srv = test_util::demo_site().unwrap();
        let guest = User::get(&mut srv, "guest3").unwrap();

        assert_eq!(guest.username, "guest3");
        assert!(guest.stream_role);
        assert!(!guest.admin_role);
    }
}
