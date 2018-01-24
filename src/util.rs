#![macro_use]

use std::string;

macro_rules! impl_cover_art {
    () => {
        pub fn cover_art(&self, sunk: &mut Sunk, size: Option<u64>) -> Result<String> {
            let args = Query::new()
                .arg("id", self.id)
                .maybe_arg("size", size)
                .build();
            sunk.build_url("getCoverArt", args)
        }
    }
}

macro_rules! get_list_as {
    ($f:ident, $t:ident) => ({
        #[derive(Deserialize)]
        #[allow(non_snake_case)]
        struct List {
            $f: Vec<$t>
        }
        serde_json::from_value::<List>($f)?.$f
    });
}

pub(crate) fn map_str<T>(v: Option<T>) -> Option<String>
where
    T: string::ToString,
{
    v.map(|v| v.to_string())
}

fn map_some_vec<T, U, F>(sv: Option<Vec<T>>, mut f: F) -> Option<Vec<U>>
where
    F: FnMut(&T) -> U,
{
    sv.map(|v| v.iter().map(|n| f(n)).collect::<Vec<U>>())
}

pub(crate) fn map_vec_string<T>(sv: Option<Vec<T>>) -> Option<Vec<String>>
where
    T: string::ToString,
{
    map_some_vec(sv, T::to_string)
}
