table! {
    servers (id) {
        id -> Integer,
        name -> Text,
        login -> Text,
        install_dir -> Text,
    }
}

table! {
    steam_apps (appid) {
        appid -> Integer,
        name -> Text,
    }
}

allow_tables_to_appear_in_same_query!(servers, steam_apps,);
