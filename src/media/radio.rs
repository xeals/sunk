//! Radio APIs.

use std::result;

use serde::de::{Deserialize, Deserializer};

use crate::id::Id;
use crate::query::Query;
use crate::{Client, Result};

#[allow(missing_docs)]
#[derive(Debug)]
#[readonly::make]
pub struct RadioStation {
    pub id: Id,
    pub name: String,
    pub stream_url: String,
    pub homepage_url: Option<String>,
}

impl<'de> Deserialize<'de> for RadioStation {
    fn deserialize<D>(de: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct _Station {
            id: String,
            name: String,
            stream_url: String,
            homepage_url: Option<String>,
        }
        let raw = _Station::deserialize(de)?;
        Ok(RadioStation {
            id: raw.id.parse().unwrap(),
            name: raw.name,
            stream_url: raw.stream_url,
            homepage_url: raw.homepage_url,
        })
    }
}

#[allow(missing_docs)]
impl RadioStation {
    pub fn id(&self) -> Id {
        self.id.clone()
    }

    pub fn list(client: &Client) -> Result<Vec<RadioStation>> {
        #[allow(non_snake_case)]
        let internetRadioStation = client.get("getInternetRadioStations", Query::none())?;
        Ok(get_list_as!(internetRadioStation, RadioStation))
    }

    pub fn create(client: &Client, name: &str, url: &str, homepage: Option<&str>) -> Result<()> {
        let args = Query::with("name", name)
            .arg("streamUrl", url)
            .arg("homepageUrl", homepage)
            .build();
        client.get("createInternetRadioStation", args)?;
        Ok(())
    }

    pub fn update(&self, client: &Client) -> Result<()> {
        let args = Query::with("id", self.id.clone())
            .arg("streamUrl", self.stream_url.as_str())
            .arg("name", self.name.as_str())
            .arg("homepageUrl", self.homepage_url.as_deref())
            .build();
        client.get("updateInternetRadioStation", args)?;
        Ok(())
    }

    pub fn delete(&self, client: &Client) -> Result<()> {
        client.get("deleteInternetRadioStation", Query::with("id", self.id.clone()))?;
        Ok(())
    }
}
