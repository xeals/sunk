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
