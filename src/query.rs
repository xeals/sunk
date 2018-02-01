use std::{fmt, iter};

/// An expandable query set for an API call.
#[derive(Debug, PartialEq, PartialOrd)]
pub struct Query {
    inner: Vec<(String, Arg)>,
}

impl Query {
    /// Creates an empty query set to be built on.
    pub fn new() -> Query { Query { inner: Vec::new() } }

    /// A blank query to be used where an API call doesn't require additional
    /// arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sunk::query::Query;
    /// let query = Query::none();
    /// assert_eq!(query.to_string(), "");
    /// ```
    pub fn none() -> Query {
        Query {
            inner: vec![("".into(), Arg(None))],
        }
    }

    /// Creates a new query with an initial argument.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sunk::query::Query;
    /// let query = Query::new()
    ///                 .arg("key", "value")
    ///                 .build();
    ///
    /// assert_eq!(query, Query::with("key", "value"));
    /// ```
    pub fn with<A: IntoArg>(key: &str, value: A) -> Query {
        Query {
            inner: vec![(key.to_string(), value.into_arg())],
        }
    }

    /// Adds an argument to the query.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sunk::query::Query;
    /// let query_with = Query::with("key", "value");
    ///
    /// let mut builder = Query::new();
    /// assert_ne!(query_with, builder);
    ///
    /// builder.arg("key", "value");
    /// assert_eq!(query_with, builder);
    /// ```
    pub fn arg<A: IntoArg>(&mut self, key: &str, value: A) -> &mut Query {
        self.inner.push((key.to_string(), value.into_arg()));
        self
    }

    /// Adds a list of arguments to the query, all with the provided key.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sunk::query::Query;
    /// let list = &[0, 1, 2];
    ///
    /// let query_list = Query::new().arg_list("index", list).build();
    ///
    /// let query_manual = Query::new()
    ///                        .arg("index", 0)
    ///                        .arg("index", 1)
    ///                        .arg("index", 2)
    ///                        .build();
    ///
    /// assert_eq!(query_list, query_manual);
    /// ```
    pub fn arg_list<A: IntoArg + Clone>(
        &mut self,
        key: &str,
        values: &[A],
    ) -> &mut Query {
        for v in values.to_owned() {
            self.inner.push((key.to_string(), v.into_arg()))
        }
        self
    }

    /// Consumes the query builder and returns a completed query.
    pub fn build(&mut self) -> Query {
        Query {
            inner: self.inner.drain(..).collect(),
        }
    }
}

impl iter::Extend<(String, Arg)> for Query {
    fn extend<T: IntoIterator<Item = (String, Arg)>>(&mut self, iter: T) {
        self.inner.extend(iter);
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (n, a) in self.inner.iter().enumerate() {
            if a.1.is_some() {
                write!(f, "{}={}", a.0, a.1)?;
                if n + 1 < self.inner.len() {
                    write!(f, "&")?;
                }
            }
        }
        Ok(())
    }
}

impl Default for Query {
    fn default() -> Query { Query::new() }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Arg(Option<String>);

impl Arg {
    fn is_some(&self) -> bool { self.0.is_some() }
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_some() {
            write!(f, "{}", self.0.clone().unwrap())
        } else {
            write!(f, "")
        }
    }
}

pub trait IntoArg {
    fn into_arg(self) -> Arg;
}

impl<T> IntoArg for Option<T>
where
    T: IntoArg,
{
    fn into_arg(self) -> Arg {
        match self {
            Some(a) => a.into_arg(),
            None => Arg(None),
        }
    }
}

impl IntoArg for Arg {
    fn into_arg(self) -> Arg { self }
}

macro_rules! impl_arg {
    ($t:ty) => {impl IntoArg for $t {
        fn into_arg(self) -> Arg { Arg(Some(self.to_string())) }
    }};
}

impl_arg!(i8);
impl_arg!(i16);
impl_arg!(i32);
impl_arg!(i64);
impl_arg!(isize);
impl_arg!(f32);
impl_arg!(f64);
impl_arg!(u8);
impl_arg!(u16);
impl_arg!(u32);
impl_arg!(u64);
impl_arg!(usize);
impl_arg!(bool);

impl<'a> IntoArg for &'a str {
    fn into_arg(self) -> Arg { Arg(Some(self.to_owned())) }
}

impl IntoArg for String {
    fn into_arg(self) -> Arg { Arg(Some(self)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_is_empty() {
        let q = Query::none();
        assert_eq!("", &format!("{}", q))
    }

    #[test]
    fn single_query() {
        let q = Query::with("id", 64);
        assert_eq!("id=64", &format!("{}", q))
    }

    #[test]
    fn two_queries() {
        let q = Query::new().arg("id", 64).arg("album", 12).build();
        assert_eq!("id=64&album=12", &format!("{}", q))
    }

    #[test]
    fn optional_query() {
        let mut q = Query::with("album", Arg(None));
        assert_eq!("", &format!("{}", q));
        q.arg("id", 64);
        assert_eq!("id=64", &format!("{}", q));
    }

    #[test]
    fn query_vec() {
        let ids = &[1, 2, 3, 4];
        let mut q = Query::new();
        q.arg_list("id", ids);
        assert_eq!("id=1&id=2&id=3&id=4", &format!("{}", q))
    }
}
