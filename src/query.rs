use error::*;
use std::fmt::{self, Display};

/// An expandable query set for a call.
#[derive(Debug)]
pub struct Query<'a, D: Display> {
    inner: Vec<(&'a str, D)>
}

impl<'a, D: Display> Query<'a, D> {
    /// Use `Query::with("", "")` if no arguments are needed. This is due to a
    /// behaviour in the type inference system that I can't wrap my head around.
    pub fn new() -> Query<'a, D> {
        Query { inner: vec![] }
    }

    pub fn with(key: &'a str, val: D) -> Query<'a, D> {
        Query { inner: vec![(key, val)] }
    }

    pub fn maybe_with(key: &'a str, val: Option<D>) -> Query<'a, D> {
        if let Some(v) = val {
            self::Query::with(key, v)
        } else {
            self::Query::new()
        }
    }

    pub fn arg(&mut self, key: &'a str, val: D) -> &mut Query<'a, D> {
        self.inner.push((key, val));
        self
    }

    pub fn maybe_arg(&mut self, key: &'a str, val: Option<D>) -> &mut Query<'a, D> {
        if let Some(v) = val {
            self.arg(key, v);
        }
        self
    }

    pub fn arg_list(&mut self, key: &'a str, val: Vec<D>) -> &mut Query<'a, D> {
        for v in val {
            self.arg(key, v);
        }
        self
    }

    pub fn maybe_arg_list(&mut self, key: &'a str, val: Option<Vec<D>>) -> &mut Query<'a, D> {
        if let Some(vals) = val {
            for v in vals {
                self.arg(key, v);
            }
        }
        self
    }

    pub fn build(&mut self) -> Query<'a, D> {
        Query { inner: self.inner.drain(..).collect() }
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

impl<'a, D: Display> Default for Query<'a, D> {
    fn default() -> Query<'a, D> {
        Query::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_is_empty() {
        let q = Query::with("", "");
        assert_eq!("", &format!("{}", q))
    }

    #[test]
    fn single_query() {
        let q = Query::with("id", 64);
        assert_eq!("id=64", &format!("{}", q))
    }

    #[test]
    fn two_queries() {
        let q = Query::new()
            .arg("id", 64)
            .arg("album", 12)
            .build();
        assert_eq!("id=64&album=12", &format!("{}", q))
    }

    #[test]
    fn optional_query() {
        let mut q = Query::maybe_with("album", None);
        assert_eq!("", &format!("{}", q));
        q.maybe_arg("id", Some(64));
        assert_eq!("id=64", &format!("{}", q));
    }

    #[test]
    fn query_vec() {
        let ids = vec![1, 2, 3, 4];
        let mut q = Query::new();
        q.arg_list("id", ids);
        assert_eq!("id=1&id=2&id=3&id=4", &format!("{}", q))
    }
}
