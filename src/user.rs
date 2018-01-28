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
pub fn create_user(
    client: &mut Client,
    username: &str,
    password: &str,
    email: &str,
) -> Result<()> {
    let args = Query::with("username", username)
        .arg("password", password)
        .arg("email", email)
        .build();
    warn!("Full permission set not yet supported");
    client.get("createUser", args)?;
    Ok(())
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
