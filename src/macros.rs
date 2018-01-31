macro_rules! get_list_as {
    ($f:ident, $t:ident) => ({
        #[derive(Deserialize)]
        #[allow(non_snake_case)]
        struct List {
            $f: Vec<$t>
        }
        ::serde_json::from_value::<List>($f)?.$f
    });
}
