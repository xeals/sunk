use serde::de::{Deserialize, Deserializer};
use std::result;

use client::Client;
use error::Result;
use query::Query;

#[derive(Debug)]
pub struct RadioStation {
    id: usize,
    name: String,
    stream_url: String,
    homepage_url: Option<String>,
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

impl RadioStation {
    pub fn id(&self) -> usize { self.id }
    pub fn stream_url(&self) -> &str { &self.stream_url }
    pub fn name(&self) -> &str { &self.name }
    pub fn homepage_url(&self) -> Option<&str> {
        self.homepage_url.map(|s| s.as_str())
    }

    pub fn set_name(&mut self, name: &str) { self.name = name.to_owned(); }
    pub fn set_stream(&mut self, url: &str) {
        self.stream_url = url.to_owned();
    }
    pub fn set_homepage(&mut self, url: &str) {
        self.homepage_url = Some(url.to_owned());
    }

    pub fn list(client: &mut Client) -> Result<Vec<RadioStation>> {
        #[allow(non_snake_case)]
        let internetRadioStation =
            client.get("getInternetRadioStations", Query::none())?;
        Ok(get_list_as!(internetRadioStation, RadioStation))
    }

    pub fn create(
        client: &mut Client,
        name: &str,
        url: &str,
        homepage: Option<&str>,
    ) -> Result<()> {
        let args = Query::with("name", name)
            .arg("streamUrl", url)
            .arg("homepageUrl", homepage)
            .build();
        client.get("createInternetRadioStation", args)?;
        Ok(())
    }

    pub fn update(&self, client: &mut Client) -> Result<()> {
        let args = Query::with("id", self.id)
            .arg("streamUrl", self.stream_url)
            .arg("name", self.name)
            .arg("homepageUrl", self.homepage_url)
            .build();
        client.get("updateInternetRadioStation", args)?;
        Ok(())
    }

    pub fn delete(&self, client: &mut Client) -> Result<()> {
        client.get("deleteInternetRadioStation", Query::with("id", self.id))?;
        Ok(())
    }
}
