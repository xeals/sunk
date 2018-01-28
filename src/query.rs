use std::fmt;

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
    pub fn none() -> Query {
        Query {
            inner: vec![("".into(), Arg(None))],
        }
    }

    /// Creates a new query with an initial argument.
    ///
    /// Shorthand for the following:
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
    /// let list = vec![0, 1, 2];
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
    pub fn arg_list<A: IntoArg>(
        &mut self,
        key: &str,
        values: Vec<A>,
    ) -> &mut Query {
        for v in values {
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

#[derive(Debug, PartialEq, PartialOrd)]
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

impl IntoArg for u8 {
    fn into_arg(self) -> Arg { Arg(Some(self.to_string())) }
}

impl IntoArg for u16 {
    fn into_arg(self) -> Arg { Arg(Some(self.to_string())) }
}

impl IntoArg for u32 {
    fn into_arg(self) -> Arg { Arg(Some(self.to_string())) }
}

impl IntoArg for u64 {
    fn into_arg(self) -> Arg { Arg(Some(self.to_string())) }
}

impl IntoArg for usize {
    fn into_arg(self) -> Arg { Arg(Some(self.to_string())) }
}

impl IntoArg for i8 {
    fn into_arg(self) -> Arg { Arg(Some(self.to_string())) }
}

impl IntoArg for i16 {
    fn into_arg(self) -> Arg { Arg(Some(self.to_string())) }
}

impl IntoArg for i32 {
    fn into_arg(self) -> Arg { Arg(Some(self.to_string())) }
}

impl IntoArg for i64 {
    fn into_arg(self) -> Arg { Arg(Some(self.to_string())) }
}

impl IntoArg for isize {
    fn into_arg(self) -> Arg { Arg(Some(self.to_string())) }
}

impl<'a> IntoArg for &'a str {
    fn into_arg(self) -> Arg { Arg(Some(self.to_owned())) }
}

impl IntoArg for String {
    fn into_arg(self) -> Arg { Arg(Some(self)) }
}

impl IntoArg for bool {
    fn into_arg(self) -> Arg { Arg(Some(self.to_string())) }
}

pub trait IntoStrArg {
    fn into_str_arg(self) -> Arg;
}

impl IntoStrArg for String {
    fn into_str_arg(self) -> Arg { self.into_arg() }
}

impl<'a> IntoStrArg for &'a str {
    fn into_str_arg(self) -> Arg { self.into_arg() }
}

pub trait IntoNumArg {
    fn into_num_arg(self) -> Arg;
}

macro_rules! impl_num {
    ($n:ty) => {
        impl IntoNumArg for Option<$n> {
            fn into_num_arg(self) -> Arg {
                self.into_arg()
            }
        }

        impl IntoNumArg for $n {
            fn into_num_arg(self) -> Arg {
                self.into_arg()
            }
        }
    };
}

impl_num!(u8);
impl_num!(u16);
impl_num!(u32);
impl_num!(u64);
impl_num!(usize);

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
        let ids = vec![1, 2, 3, 4];
        let mut q = Query::new();
        q.arg_list("id", ids);
        assert_eq!("id=1&id=2&id=3&id=4", &format!("{}", q))
    }
}
