#[macro_use]
extern crate diesel;

use std::sync::Mutex;

use config::Config;
use db::DB;
use handlers::{create_server, generate_apps, get_server, list_servers, search_apps};
//use storage::FileStorage;
//use serde::{Deserialize, Serialize};

mod db;
mod handlers;
mod install;
mod schema;
mod service;
mod steam_apps;
//mod storage;
mod types;

#[macro_use]
extern crate rocket;

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings: types::ServerConfig = Config::builder()
        .add_source(config::File::with_name("./config.toml"))
        .add_source(config::Environment::with_prefix("STEAM"))
        .set_default("steamcmd_location", "./steamcmd.sh")?
        .set_default("steam_api_url", "https://api.steampowered.com")?
        .build()?
        .try_deserialize()?;

    let app_service = service::SteamAppsService::new(&settings.steam_api_url);
    let storage = DB::establish_connection(&settings.database_url)?; //FileStorage::new("./server_data");
    let install_service = service::InstallService::new(&settings.steamcmd_location, storage);
    rocket::build()
        .manage(app_service)
        .manage(settings)
        .manage(Mutex::new(install_service))
        .attach(steam_apps::Db::fairing())
        .mount("/apps", routes![search_apps, generate_apps])
        .mount(
            "/server",
            routes![
                create_server,
                get_server,
                list_servers,
                crate::handlers::install
            ],
        )
        .launch()
        .await?;
    Ok(())
}
