#![macro_use]

use error::*;
use json;

macro_rules! fetch {
    ($j:ident->$i:ident: $t:ident) => (
        $j[stringify!($i)].$t().ok_or(
            Error::ParseError(stringify!(failed parsing $i.$t))
        )?
    );
    ($j:ident->$i:ident: $t:ident, $u:ty) => (
        fetch!($j->$i: $t).parse::<$u>().map_err(
            |_| Error::ParseError(stringify!(not a $u))
        )?
    );
}

macro_rules! fetch_maybe {
    ($j:ident->$i:ident: $t:ident) => {
        $j[stringify!($i)].$t()
    };
    ($j:ident->$i:ident: $t:ident, $u:ty) => {
        // fetch_maybe!($j->$i: $t).map(|v| v.parse::<$u>().map_err(
        //     |_| Error::ParseError(stringify!(not a $u))
        // ))
        match fetch_maybe!($j->$i: $t).map(|v| v.parse::<$u>().map_err(
            |_| Error::ParseError(stringify!(not a $u))
        )) {
            Some(Ok(v)) => Some(v),
            _ => None
        }
    }
}

macro_rules! pointer {
    ($json:ident, $path:expr) => (
        $json.pointer($path)
            .ok_or(Error::ParseError(stringify!(nothing found at $path)))?
    )
}

/// If the value is `Some`, pushes the value to the vector with the given key.
macro_rules! push_if_some {
    ($vec:ident, $key:expr, $val:ident) => {
        if let Some(v) = $val {
            $vec.push(($key, v.to_string()))
        }
    }
}

/// If the value is `Some`, pushes the values of the argument vector to the
/// parent vector, all with the provided key.
macro_rules! push_all_if_some {
    ($vec:ident, $key:expr, $vals:ident) => {
        if let Some(vs) = $vals {
            for v in vs {
                $vec.push(($key, v.to_string()))
            }
        }
    }
}

pub(crate) fn map_str<T>(v: Option<T>) -> Option<String>
where
    T: ::std::string::ToString
{
    v.map(|v| v.to_string())
}

pub(crate) fn map_some_vec<T, U, F>(
    sv: Option<Vec<T>>, f: F
) -> Option<Vec<U>>
where
    F: FnOnce(&T) -> U,
{
    sv.map(|v| {
        v.iter().map(|n| {
            f(n)
        }).collect::<Vec<U>>()
    })
}
