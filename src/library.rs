use json;
use error::*;

#[derive(Debug)]
pub struct MusicFolder {
    pub id: usize,
    pub name: String,
}

impl MusicFolder {
    pub fn try_from(json: json::Value) -> Result<MusicFolder> {
        Ok(MusicFolder {
            id: json["id"].as_str().unwrap().parse()?,
            name: json["name"].as_str().unwrap().to_string(),
        })
    }

    fn from(id: usize, name: String) -> MusicFolder {
        MusicFolder { id, name }
    }
}

#[derive(Debug)]
pub struct Genre {
    pub name: String,
    pub song_count: u64,
    pub album_count: u64,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GenreSerde {
    songCount: u64,
    albumCount: u64,
    value: String
}

impl Genre {
    pub fn try_from(json: json::Value) -> Result<Genre> {
        let serde: GenreSerde = json::from_value(json)?;
        Ok(Genre {
            name: serde.value,
            song_count: serde.songCount,
            album_count: serde.albumCount,
        })
    }
}
