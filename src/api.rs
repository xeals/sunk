use error::*;
use std::{convert, fmt};

#[derive(PartialEq, PartialOrd, Eq, Ord, Copy, Clone)]
pub struct Api (u8, u8, u8);

impl convert::From<String> for Api {
    fn from(s: String) -> Api {
        let mut spl = s.split('.');

        macro_rules! ver {
            ($v:ident) => (let $v = match spl.next() {
                Some(n) => n.parse::<u8>().unwrap(),
                None => 0,
            };)
        }

        ver!(major);
        ver!(minor);
        ver!(inc);

        Api (major, minor, inc)
    }
}

impl<'a> convert::From<&'a str> for Api {
    fn from(s: &'a str) -> Api {
        Api::from(s.to_string())
    }
}

impl fmt::Debug for Api {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Api: {{ {}.{}.{} }}", self.0, self.1, self.2)
    }
}

impl fmt::Display for Api {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

#[cfg(test)]
mod tests {
    use api::Api;

    #[test]
    fn test_parse_api_full() {
        let s = "1.11.0";
        let v = Api::from(s);
        assert_eq!(v.0, 1);
        assert_eq!(v.1, 11);
        assert_eq!(v.2, 0);
    }

    #[test]
    fn test_parse_api_no_inc() {
        let s = "1.12";
        let v = Api::from(s);
        assert_eq!(v.0, 1);
        assert_eq!(v.1, 12);
        assert_eq!(v.2, 0);
    }
}
