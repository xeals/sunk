#![macro_use]

macro_rules! impl_cover_art {
    () => {
        pub fn cover_art(&self, client: &mut Client, size: Option<u64>) -> Result<String> {
            let args = Query::new()
                .arg("id", self.id)
                .arg("size", size)
                .build();
            client.build_url("getCoverArt", args)
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
