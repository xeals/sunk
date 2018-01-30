use serde::de::{Deserialize, Deserializer};
use std::result;

#[derive(Debug)]
pub struct MusicFolder {
    pub id: usize,
    pub name: String,
}

impl MusicFolder {
    fn from(id: usize, name: String) -> MusicFolder { MusicFolder { id, name } }
}

impl<'de> Deserialize<'de> for MusicFolder {
    fn deserialize<D>(de: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct _MusicFolder {
            id: String,
            name: String,
        }

        let raw = _MusicFolder::deserialize(de)?;
        Ok(MusicFolder {
            id: raw.id.parse().unwrap(),
            name: raw.name,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Genre {
    pub name: String,
    pub song_count: u64,
    pub album_count: u64,
}

pub mod search {
    use std::fmt;

    pub const ALL: SearchPage = SearchPage {
        count: 500,
        offset: 0,
    };

    pub const NONE: SearchPage = SearchPage {
        count: 0,
        offset: 0,
    };

    #[derive(Debug, Copy, Clone)]
    pub struct SearchPage {
        pub count: usize,
        pub offset: usize,
    }

    impl SearchPage {
        pub fn new() -> SearchPage {
            SearchPage {
                offset: 0,
                count: 20,
            }
        }

        pub fn at_page(offset: usize) -> SearchPage {
            SearchPage { offset, count: 20 }
        }

        pub fn with_size(self, count: usize) -> SearchPage {
            SearchPage {
                offset: self.offset,
                count,
            }
        }
    }

    impl Default for SearchPage {
        fn default() -> SearchPage { SearchPage::new() }
    }

    impl fmt::Display for SearchPage {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "search range {}-{}",
                self.count * self.offset,
                (self.count + 1) * self.offset - 1
            )
        }
    }
}
