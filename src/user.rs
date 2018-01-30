use client::Client;
use error::Result;
use query::Query;
use serde_json;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub username: String,
    pub email: String,
    #[serde(default)]
    max_bit_rate: u64,
    scrobbling_enabled: bool,
    admin_role: bool,
    settings_role: bool,
    download_role: bool,
    upload_role: bool,
    playlist_role: bool,
    cover_art_role: bool,
    comment_role: bool,
    podcast_role: bool,
    stream_role: bool,
    jukebox_role: bool,
    share_role: bool,
    video_conversion_role: bool,
    avatar_last_changed: String,
    #[serde(rename = "folder")]
    folders: Vec<u64>,
}

impl User {
    pub fn bitrate_limit(&self) -> u64 { self.max_bit_rate }
    pub fn is_admin(&self) -> bool { self.admin_role }
    pub fn can_comment(&self) -> bool { self.comment_role }
    pub fn can_download(&self) -> bool { self.download_role }
    pub fn can_scrobble(&self) -> bool { self.scrobbling_enabled }
    pub fn can_share(&self) -> bool { self.share_role }
    pub fn can_stream(&self) -> bool { self.stream_role }
    pub fn can_upload(&self) -> bool { self.upload_role }
    pub fn can_manage_cover(&self) -> bool { self.cover_art_role }
    pub fn can_manage_jukebox(&self) -> bool { self.jukebox_role }
    pub fn can_manage_playlist(&self) -> bool { self.playlist_role }
    pub fn can_manage_podcast(&self) -> bool { self.podcast_role }
    pub fn can_manage_self(&self) -> bool { self.settings_role }

    pub fn change_password(
        &self,
        client: &mut Client,
        password: &str,
    ) -> Result<()> {
        self::change_password(client, &self.username, password)
    }

    /// Returns the user's avatar image as a collection of bytes.
    ///
    /// The method makes no guarantee as to the encoding of the image, but does guarantee that it
    /// is a valid image file.
    pub fn avatar(&self, client: &mut Client) -> Result<Vec<u8>> {
        client.get_bytes("getAvatar", Query::with("username", self.username.as_str()))
    }

    /// Creates a new local user to be pushed to the server.
    ///
    /// See the [`UserBuilder`] struct for more details.
    ///
    /// [`UserBuilder`]: struct.UserBuilder.html
    pub fn new(username: &str, password: &str, email: &str) -> UserBuilder {
        UserBuilder::new(username, password, email)
    }
}

pub fn get_user(client: &mut Client, username: &str) -> Result<User> {
    let res = client.get("getUser", Query::with("username", username))?;
    Ok(serde_json::from_value::<User>(res)?)
}

pub fn get_users(client: &mut Client) -> Result<Vec<User>> {
    let user = client.get("getUsers", Query::none())?;
    Ok(get_list_as!(user, User))
}

// TODO: Figure out how to pass fifteen possible permissions cleanly.
pub fn update_user(client: &mut Client, username: &str) -> Result<()> {
    client.get("updateUser", Query::with("username", username))?;
    Ok(())
}

pub fn delete_user(client: &mut Client, username: &str) -> Result<()> {
    client.get("deleteUser", Query::with("username", username))?;
    Ok(())
}

pub fn change_password(
    client: &mut Client,
    username: &str,
    password: &str,
) -> Result<()> {
    let args = Query::with("username", username)
        .arg("password", password)
        .build();
    client.get("changePassword", args)?;
    Ok(())
}

/// A new user to be created.
#[derive(Clone, Debug, Default)]
pub struct UserBuilder {
    #[doc(hidden)]
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
    max_bitrate: u64,
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
    /// The maximum bitrate (in Kbps) the user is allowed to stream at. Higher bitrate streams
    /// will be downsampled to their limit.
    build!(max_bitrate: u64);

    /// Pushes a defined new user to the Subsonic server.
    pub fn create(&self, client: &mut Client) -> Result<()> {
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
            .arg_list("musicFolderId", self.folders.clone())
            .arg("maxBitRate", self.max_bitrate)
            .build();
        client.get("createUser", args)?;
        Ok(())
    }

    pub fn update(&self, client: &mut Client) -> Result<()> {
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
            .arg_list("musicFolderId", self.folders.clone())
            .arg("maxBitRate", self.max_bitrate)
            .build();
        client.get("updateUser", args)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_util;

    #[test]
    fn remote_parse_user() {
        let mut srv = test_util::demo_site().unwrap();
        let guest = get_user(&mut srv, "guest3").unwrap();

        assert_eq!(guest.username, "guest3");
        assert!(guest.can_stream());
        assert!(!guest.is_admin());
    }
}
