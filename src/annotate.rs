//! Annotation APIs.

use crate::query::Query;
use crate::{Album, Artist, Client, Error, Result, Song};

/// Allows starring, rating, and scrobbling media.
#[async_trait::async_trait]
pub trait Annotatable {
    /// Attaches a star to the content.
    async fn star(&self, client: &Client) -> Result<()>;

    /// Removes a star from the content.
    async fn unstar(&self, client: &Client) -> Result<()>;

    /// Sets the rating for the content.
    async fn set_rating(&self, client: &Client, rating: u8) -> Result<()>;

    /// Registers the local playback of the content. Typically used when playing
    /// media that is cached on the client. This operation includes the
    /// following:
    ///
    /// - "Scrobbles" the media files on last.fm if the user has configured
    /// their last.fm credentials on the Subsonic server.
    /// - Updates the play count and last played timestamp for the content.
    /// - Makes the content appear in the "Now Playing" page in the web app,
    /// and appear in the list of songs returned by
    /// [`Client::now_playing()`] (since API version 1.11.0).
    ///
    /// [`Client::now_playing()`]: ./struct.Client.html#method.now_playing
    ///
    /// `time` should be a valid ISO8601 timestamp. In the future, this will be
    /// validated.
    async fn scrobble<'a, B: Send, T: Send>(
        &self,
        client: &Client,
        time: T,
        now_playing: B,
    ) -> Result<()>
    where
        B: Into<Option<bool>>,
        T: Into<Option<&'a str>>;
}

#[async_trait::async_trait]
impl Annotatable for Artist {
    async fn star(&self, client: &Client) -> Result<()> {
        client.get("star", Query::with("artistId", self.id)).await?;
        Ok(())
    }

    async fn unstar(&self, client: &Client) -> Result<()> {
        client
            .get("unstar", Query::with("artistId", self.id))
            .await?;
        Ok(())
    }

    async fn set_rating(&self, client: &Client, rating: u8) -> Result<()> {
        if rating > 5 {
            return Err(Error::Other("rating must be between 0 and 5 inclusive"));
        }

        let args = Query::with("id", self.id).arg("rating", rating).build();
        client.get("setRating", args).await?;
        Ok(())
    }

    async fn scrobble<'a, B, T>(&self, client: &Client, time: T, now_playing: B) -> Result<()>
    where
        B: Into<Option<bool>> + Send,
        T: Into<Option<&'a str>> + Send,
    {
        let args = Query::with("id", self.id)
            .arg("time", time.into())
            .arg("submission", now_playing.into().map(|b| !b))
            .build();
        client.get("scrobble", args).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Annotatable for Album {
    async fn star(&self, client: &Client) -> Result<()> {
        client.get("star", Query::with("albumId", self.id)).await?;
        Ok(())
    }

    async fn unstar(&self, client: &Client) -> Result<()> {
        client
            .get("unstar", Query::with("albumId", self.id))
            .await?;
        Ok(())
    }

    async fn set_rating(&self, client: &Client, rating: u8) -> Result<()> {
        if rating > 5 {
            return Err(Error::Other("rating must be between 0 and 5 inclusive"));
        }

        let args = Query::with("id", self.id).arg("rating", rating).build();
        client.get("setRating", args).await?;
        Ok(())
    }

    async fn scrobble<'a, B, T>(&self, client: &Client, time: T, now_playing: B) -> Result<()>
    where
        B: Into<Option<bool>> + Send,
        T: Into<Option<&'a str>> + Send,
    {
        let args = Query::with("id", self.id)
            .arg("time", time.into())
            .arg("submission", now_playing.into().map(|b| !b))
            .build();
        client.get("scrobble", args).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Annotatable for Song {
    async fn star(&self, client: &Client) -> Result<()> {
        client.get("star", Query::with("id", self.id)).await?;
        Ok(())
    }

    async fn unstar(&self, client: &Client) -> Result<()> {
        client.get("unstar", Query::with("id", self.id)).await?;
        Ok(())
    }

    async fn set_rating(&self, client: &Client, rating: u8) -> Result<()> {
        if rating > 5 {
            return Err(Error::Other("rating must be between 0 and 5 inclusive"));
        }

        let args = Query::with("id", self.id).arg("rating", rating).build();
        client.get("setRating", args).await?;
        Ok(())
    }

    async fn scrobble<'a, B, T>(&self, client: &Client, time: T, now_playing: B) -> Result<()>
    where
        B: Into<Option<bool>> + Send,
        T: Into<Option<&'a str>> + Send,
    {
        let args = Query::with("id", self.id)
            .arg("time", time.into())
            .arg("submission", now_playing.into().map(|b| !b))
            .build();
        client.get("scrobble", args).await?;
        Ok(())
    }
}
