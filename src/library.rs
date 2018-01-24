use serde_json;
use error::*;

#[derive(Debug)]
pub struct MusicFolder {
    pub id: usize,
    pub name: String,
}

impl MusicFolder {
    pub fn try_from(json: serde_json::Value) -> Result<MusicFolder> {
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
    pub fn try_from(json: serde_json::Value) -> Result<Genre> {
        let serde: GenreSerde = serde_json::from_value(json)?;
        Ok(Genre {
            name: serde.value,
            song_count: serde.songCount,
            album_count: serde.albumCount,
        })
    }
}

pub mod search {
    use std::fmt;

    pub const ALL: SearchPage = SearchPage { count: 500, offset: 0 };

    #[derive(Debug, Copy, Clone)]
    pub struct SearchPage {
        pub count: usize,
        pub offset: usize
    }

    impl SearchPage {
        pub fn new() -> SearchPage {
            SearchPage { offset: 0, count: 0 }
        }

        pub fn at_page(offset: usize) -> SearchPage {
            SearchPage { offset, count: 0 }
        }

        pub fn with_size(self, count: usize) -> SearchPage {
            SearchPage { offset: self.offset, count }
        }
    }

    impl Default for SearchPage {
        fn default() -> SearchPage {
            SearchPage::new()
        }
    }

    impl fmt::Display for SearchPage {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "search range {}-{}", self.count * self.offset, (self.count + 1) * self.offset - 1)
        }
    }
}
