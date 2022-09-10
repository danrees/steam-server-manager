use log::debug;
use rocket::response::stream::{Event, EventStream};
use rocket::{form::Form, serde::json::Json, State};

use crate::steam_apps::App;
use crate::{db, steam_apps};
use crate::{
    install::Server,
    service::{InstallService, SteamAppsService},
    //storage::FileStorage,
};

use std::sync::PoisonError;

pub struct Tx(pub flume::Sender<String>);
pub struct Rx(pub flume::Receiver<String>);

#[derive(Debug, Responder)]
#[response(status = 500, content_type = "json")]
pub struct ServiceError(String);

#[derive(FromForm)]
pub struct SearchTerms<'a> {
    term: &'a str,
}

impl From<anyhow::Error> for ServiceError {
    fn from(err: anyhow::Error) -> Self {
        ServiceError(format!("{}", err))
    }
}

impl<S> From<PoisonError<S>> for ServiceError {
    fn from(err: PoisonError<S>) -> Self {
        ServiceError(format!("{}", err))
    }
}

#[post("/search", data = "<term>")]
pub async fn search_apps(
    term: Form<SearchTerms<'_>>,
    db: steam_apps::Db,
    steam_apps_service: &State<SteamAppsService>,
) -> Result<Json<Vec<App>>, ServiceError> {
    let app = steam_apps_service.search(term.term, db).await?;

    Ok(Json(app))
}

#[post("/generate")]
pub async fn generate_apps(
    db: steam_apps::Db,
    steam_apps_service: &State<SteamAppsService>,
) -> Result<(), ServiceError> {
    steam_apps_service.generate(db).await.map_err(|e| e.into())
}

#[post("/", data = "<server>")]
pub async fn create_server(
    server: Json<Server>,
    // TODO: How can I still do this generically
    install_service: &State<InstallService>,
    db: db::Db,
) -> Result<(), ServiceError> {
    let service = install_service;
    service.new_server(&server, db).await?;
    Ok(())
}

#[get("/<id>")]
pub async fn get_server(
    id: i32,
    install_service: &State<InstallService>,
    db: db::Db,
) -> Result<Json<Server>, ServiceError> {
    install_service
        .get_server(id, db)
        .await
        .map(|m| Json(m))
        .map_err(|e| e.into())
}

#[get("/")]
pub async fn list_servers(
    install_service: &State<InstallService>,
    db: db::Db,
) -> Result<Json<Vec<Server>>, ServiceError> {
    install_service
        .list_servers(db)
        .await
        .map(|m| Json(m))
        .map_err(|e| e.into())
}

#[post("/install/<id>")]
pub async fn install(
    id: i32,
    install_service: &State<InstallService>,
    db: db::Db,
    tx: &State<Tx>,
) -> Result<(), ServiceError> {
    let tx_clone = tx.0.clone();
    let s = install_service.install(id, tx_clone, db);
    debug!("installing {}", id);

    s.await?;
    Ok(())
}

#[get("/install/events")]
pub async fn install_events(rx: &State<Rx>) -> EventStream![Event + '_] {
    EventStream! {
        while let Ok(msg) = rx.0.recv() {
            debug!("{}", msg);
            yield Event::data(msg);
        }
    }
}
