#[macro_use]
extern crate diesel;

use std::sync::Mutex;

use config::Config;
use handlers::{
    apps::{generate_apps, search_apps},
    server::{create_server, delete, get_server, install_events, list_servers},
    test::test_events,
    Rx, Tx,
};
use install::{InstallQueueRx, InstallQueueTx, Server};
use threadpool::ThreadPool;
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
        .set_default("workers", 4)?
        .build()?
        .try_deserialize()?;

    let thread_pool = ThreadPool::new(settings.workers);

    let app_service = service::SteamAppsService::new(&settings.steam_api_url);
    let storage = db::DBStorage {}; //FileStorage::new("./server_data");
    let server_service = service::ServerService::new(storage);
    let install_service =
        service::InstallService::new(&settings.steamcmd_location, &settings.base_dir);
    let (tx, rx) = flume::unbounded::<String>();
    let (install_tx, install_rx) = flume::unbounded::<Server>();
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
        .manage(server_service)
        .manage(Rx(rx))
        .manage(Tx(tx))
        .manage(InstallQueueRx(install_rx))
        .manage(InstallQueueTx(install_tx))
        .manage(Mutex::new(thread_pool))
        .attach(steam_apps::Db::fairing())
        .attach(db::Db::fairing())
        .attach(cors::CORS)
        .attach(install_service)
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
