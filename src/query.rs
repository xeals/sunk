use error::*;
use std::fmt::{self, Display};

/// An expandable query set for a call.
pub struct Query<'a, D: Display> {
    inner: Vec<(&'a str, D)>
}

impl<'a, D: Display> Query<'a, D> {
    pub fn new() -> Query<'a, D> {
        Query { inner: vec![] }
    }

    /// Short-hand function to get an empty set of queries without having to
    /// declare types.
    pub fn empty() -> Query<'a, String> {
        self::Query::new()
    }

    pub fn from(key: &'a str, val: D) -> Query<'a, D> {
        let mut q = self::Query::new();
        q.push(key, val);
        q
    }

    pub fn from_some(key: &'a str, val: Option<D>) -> Query<'a, D> {
        let mut q = self::Query::new();
        if let Some(v) = val {
            q.push(key, v);
        }
        q
    }

    pub fn push(&mut self, key: &'a str, val: D) {
        self.inner.push((key, val))
    }

    pub fn push_some(&mut self, key: &'a str, val: Option<D>) {
        if let Some(v) = val {
            self.push(key, v)
        }
    }

    pub fn push_all(&mut self, key: &'a str, val: Vec<D>) {
        for v in val {
            self.push(key, v)
        }
    }

    pub fn push_all_some(&mut self, key: &'a str, val: Option<Vec<D>>) {
        if let Some(vals) = val {
            for v in vals {
                self.push(key, v)
            }
        }
    }
}

impl<'a, D: Display> Display for Query<'a, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, &(ref k, ref v)) in self.inner.iter().enumerate() {
            write!(f, "{}={}", k, v)?;
            if i + 1 < self.inner.len() {
                write!(f, "&")?;
            }
        }
        Ok(())
    }
}
