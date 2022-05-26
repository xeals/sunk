//! Methods and containers for searching and search results.
//!
//! The Subsonic API works on the concept of paging, something not uncommon in
//! RESTful APIs. A search will return a number of results up to a
//! specification. The client then has a virtual "page" number they will send,
//! to offset a search by a multiple of the return number.
//!
//! # Example
//!
//! Suppose a Subsonic server has 50 albums stored on it.
//!
//! ```no_run
//! extern crate sunk;
//! use sunk::{Album, Client, ListType};
//! use sunk::search::{self, SearchPage};
//!
//! # fn run() -> sunk::Result<()> {
//! # let site = "https://demo.subsonic.org";
//! # let username = "guest3";
//! # let password = "guest";
//! let client = Client::new(site, username, password)?;
//! let mut page = SearchPage::new();
//! let list = ListType::default();
//!
//! let results = Album::list(&client, list, page, 0)?;
//! assert_eq!(results.len(), 20);
//! #
//! # page.next();
//! # let more_results = Album::list(&client, list, page, 0)?;
//! # assert_eq!(more_results.len(), 20);
//! #
//! # page.next();
//! # let last_results = Album::list(&client, list, page, 0)?;
//! # assert_eq!(last_results.len(), 10);
//! #
//! # let exact = SearchPage::new().with_size(50);
//! # let exact_results = Album::list(&client, list, exact, 0)?;
//! # assert_eq!(exact_results.len(), 50);
//! #
//! # let all = search::ALL;
//! # let all_results = Album::list(&client, list, all, 0)?;
//! # assert_eq!(all_results.len(), 50);
//! #
//! # Ok(())
//! # }
//! # fn main() { }
//! ```
//!
//! How do we get the remaining 30 songs from the server? By paging.
//!
//! ```no_run
//! # extern crate sunk;
//! # use sunk::{Album, Client, ListType};
//! # use sunk::search::{self, SearchPage};
//! #
//! # fn run() -> sunk::Result<()> {
//! # let site = "https://demo.subsonic.org";
//! # let username = "guest3";
//! # let password = "guest";
//! # let client = Client::new(site, username, password)?;
//! # let mut page = SearchPage::new();
//! # let list = ListType::default();
//! #
//! # let results = Album::list(&client, list, page, 0)?;
//! # assert_eq!(results.len(), 20);
//! #
//! page.next();
//! let more_results = Album::list(&client, list, page, 0)?;
//! assert_eq!(more_results.len(), 20);
//!
//! page.next();
//! let last_results = Album::list(&client, list, page, 0)?;
//! assert_eq!(last_results.len(), 10);
//! #
//! # let exact = SearchPage::new().with_size(50);
//! # let exact_results = Album::list(&client, list, exact, 0)?;
//! # assert_eq!(exact_results.len(), 50);
//! #
//! # let all = search::ALL;
//! # let all_results = Album::list(&client, list, all, 0)?;
//! # assert_eq!(all_results.len(), 50);
//! #
//! # Ok(())
//! # }
//! # fn main() { }
//! ```
//!
//! Notice that the last set of results only returns *up to* the count in the
//! `SearchPage`.
//!
//! Of course, if we knew beforehand how many results there would be, we could
//! request exactly fifty albums.
//!
//! ```no_run
//! # extern crate sunk;
//! # use sunk::{Album, Client, ListType};
//! # use sunk::search::{self, SearchPage};
//! #
//! # fn run() -> sunk::Result<()> {
//! # let site = "https://demo.subsonic.org";
//! # let username = "guest3";
//! # let password = "guest";
//! # let client = Client::new(site, username, password)?;
//! # let mut page = SearchPage::new();
//! # let list = ListType::default();
//! #
//! # let results = Album::list(&client, list, page, 0)?;
//! # assert_eq!(results.len(), 20);
//! #
//! # page.next();
//! # let more_results = Album::list(&client, list, page, 0)?;
//! # assert_eq!(more_results.len(), 20);
//! #
//! # page.next();
//! # let last_results = Album::list(&client, list, page, 0)?;
//! # assert_eq!(last_results.len(), 10);
//! #
//! let exact = SearchPage::new().with_size(50);
//! let exact_results = Album::list(&client, list, exact, 0)?;
//! assert_eq!(exact_results.len(), 50);
//! #
//! # let all = search::ALL;
//! # let all_results = Album::list(&client, list, all, 0)?;
//! # assert_eq!(all_results.len(), 50);
//! #
//! # Ok(())
//! # }
//! # fn main() { }
//! ```
//!
//! However, if we didn't, there's a convinent constant in place to return up
//! to 500 results. This is set at 500 because most Subsonic functions only
//! accept up to returning 500 results. It's still possible to page through
//! results if you have to.
//!
//! ```no_run
//! # extern crate sunk;
//! # use sunk::{Album, Client, ListType};
//! # use sunk::search::{self, SearchPage};
//! #
//! # fn run() -> sunk::Result<()> {
//! # let site = "https://demo.subsonic.org";
//! # let username = "guest3";
//! # let password = "guest";
//! # let client = Client::new(site, username, password)?;
//! # let mut page = SearchPage::new();
//! # let list = ListType::default();
//! #
//! # let results = Album::list(&client, list, page, 0)?;
//! # assert_eq!(results.len(), 20);
//! #
//! # page.next();
//! # let more_results = Album::list(&client, list, page, 0)?;
//! # assert_eq!(more_results.len(), 20);
//! #
//! # page.next();
//! # let last_results = Album::list(&client, list, page, 0)?;
//! # assert_eq!(last_results.len(), 10);
//! #
//! # let exact = SearchPage::new().with_size(50);
//! # let exact_results = Album::list(&client, list, exact, 0)?;
//! # assert_eq!(exact_results.len(), 50);
//! #
//! let all = search::ALL;
//! let all_results = Album::list(&client, list, all, 0)?;
//! assert_eq!(all_results.len(), 50);
//! #
//! # Ok(())
//! # }
//! # fn main() { }
//! ```

use std::fmt;

use crate::song::Song;
use crate::{Album, Artist};

/// The maximum number of results most searches will accept.
pub const ALL: SearchPage = SearchPage {
    count: 500,
    offset: 0,
};

/// Effectively makes a search ignore the field.
pub const NONE: SearchPage = SearchPage {
    count: 0,
    offset: 0,
};

/// A holding struct for a search configuration.
///
/// See the [module-level documentation](./index.html) for more information.
#[derive(Debug, Copy, Clone)]
pub struct SearchPage {
    /// The number of results to return.
    pub count: usize,
    /// The page offset.
    pub offset: usize,
}

impl SearchPage {
    /// Creates a new search page configuration.
    pub fn new() -> SearchPage {
        SearchPage {
            offset: 0,
            count: 20,
        }
    }

    /// Creates the configuration at the provided page.
    pub fn at_page(offset: usize) -> SearchPage {
        SearchPage { offset, count: 20 }
    }

    /// Sets the configuration to the given size.
    pub fn with_size(self, count: usize) -> SearchPage {
        SearchPage {
            offset: self.offset,
            count,
        }
    }

    /// Advances the page.
    pub fn next(&mut self) {
        self.offset += 1;
    }

    /// Decrements the page.
    pub fn prev(&mut self) {
        self.offset -= 1;
    }
}

impl Default for SearchPage {
    fn default() -> SearchPage {
        SearchPage::new()
    }
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

/// A holder struct for a search result.
#[derive(Debug, Deserialize, Clone)]
pub struct SearchResult {
    /// Artists found in the search.
    #[serde(rename = "artist")]
    #[serde(default)]
    pub artists: Vec<Artist>,
    /// Albums found in the search.
    #[serde(rename = "album")]
    #[serde(default)]
    pub albums: Vec<Album>,
    /// Songs found in the search.
    #[serde(rename = "song")]
    #[serde(default)]
    pub songs: Vec<Song>,
}
