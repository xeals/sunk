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
