use error::*;
use std::fmt::{self, Display};

/// An expandable query set for a call.
pub struct Query<'a, D: Display> {
    inner: Vec<(&'a str, D)>
}

impl<'a, D: Display> Query<'a, D> {
    /// Use `Query::from("", "")` if no arguments are needed. This is due to a
    /// behaviour in the type inference system that I can't wrap my head around.
    pub fn new() -> Query<'a, D> {
        Query { inner: vec![] }
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
            if k.is_empty() {
                break;
            }
            write!(f, "{}={}", k, v)?;
            if i + 1 < self.inner.len() {
                write!(f, "&")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_is_empty() {
        let q = Query::from("", "");
        assert_eq!("", &format!("{}", q))
    }

    #[test]
    fn single_query() {
        let q = Query::from("id", 64);
        assert_eq!("id=64", &format!("{}", q))
    }

    #[test]
    fn two_queries() {
        let mut q = Query::new();
        q.push("id", 64);
        q.push("album", 12);
        assert_eq!("id=64&album=12", &format!("{}", q))
    }

    #[test]
    fn optional_query() {
        let mut q = Query::new();
        q.push_some("album", None);
        assert_eq!("", &format!("{}", q));
        q.push_some("id", Some(64));
        assert_eq!("id=64", &format!("{}", q));
    }

    #[test]
    fn query_vec() {
        let mut q = Query::new();
        let ids = vec![1, 2, 3, 4];
        q.push_all("id", ids);
        assert_eq!("id=1&id=2&id=3&id=4", &format!("{}", q))
    }
}
