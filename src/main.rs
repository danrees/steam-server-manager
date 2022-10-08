#[macro_use]
extern crate diesel;

use config::Config;
use handlers::{
    apps::{generate_apps, search_apps},
    server::{create_server, delete, get_server, install_events, list_servers},
    test::test_events,
    Rx, Tx,
};
//use storage::FileStorage;
//use serde::{Deserialize, Serialize};

mod db;
mod handlers;
mod install;
mod schema;
mod service;
mod steam_apps;
//mod storage;
mod cors;
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
    let storage = db::DBStorage {}; //FileStorage::new("./server_data");
    let install_service =
        service::InstallService::new(&settings.steamcmd_location, &settings.base_dir, storage);
    let (tx, rx) = flume::unbounded::<String>();
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()?;

    let _r = rocket::build()
        .manage(app_service)
        .manage(settings)
        .manage(install_service)
        .manage(Rx(rx))
        .manage(Tx(tx))
        .attach(steam_apps::Db::fairing())
        .attach(db::Db::fairing())
        .attach(cors::CORS)
        .mount("/apps", routes![search_apps, generate_apps])
        .mount(
            "/server",
            routes![
                create_server,
                get_server,
                list_servers,
                crate::handlers::server::install,
                install_events,
                delete,
            ],
        )
        .mount("/test", routes![test_events])
        .launch()
        .await?;

    Ok(())
}
