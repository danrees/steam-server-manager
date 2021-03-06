use rocket::response::stream::TextStream;
use rocket::{form::Form, serde::json::Json, State};

use crate::db::DB;
use crate::{
    install::Server,
    service::{InstallService, SteamAppsService},
    steam_apps::App,
    //storage::FileStorage,
};

use std::io::BufWriter;
use std::sync::{Mutex, PoisonError};

#[derive(Debug, Responder)]
#[response(status = 500, content_type = "json")]
pub struct ServiceError(String);

#[derive(FromForm)]
pub struct SearchTerms<'a> {
    term: &'a str,
    case_insensitive: bool,
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
pub fn search_apps(
    term: Form<SearchTerms<'_>>,
    steam_apps_service: &State<SteamAppsService>,
) -> Result<Json<Vec<App>>, ServiceError> {
    let app = steam_apps_service.search(term.term, term.case_insensitive)?;

    Ok(Json(app))
}

#[post("/generate")]
pub async fn generate_apps(
    steam_apps_service: &State<SteamAppsService>,
) -> Result<(), ServiceError> {
    steam_apps_service.generate().await.map_err(|e| e.into())
}

#[post("/", data = "<server>")]
pub fn create_server(
    server: Json<Server>,
    // TODO: How can I still do this generically
    install_service: &State<Mutex<InstallService<DB>>>,
) -> Result<(), ServiceError> {
    let service = install_service.lock()?;
    service.new_server(&server)?;
    Ok(())
}

#[get("/<id>")]
pub fn get_server(
    id: i32,
    install_service: &State<Mutex<InstallService<DB>>>,
) -> Result<Json<Server>, ServiceError> {
    install_service
        .lock()?
        .get_server(id)
        .map(|m| Json(m))
        .map_err(|e| e.into())
}

#[get("/")]
pub fn list_servers(
    install_service: &State<Mutex<InstallService<DB>>>,
) -> Result<Json<Vec<Server>>, ServiceError> {
    install_service
        .lock()?
        .list_servers()
        .map(|m| Json(m))
        .map_err(|e| e.into())
}

#[post("/install/<id>")]
pub fn install(
    id: i32,
    install_service: &State<Mutex<InstallService<DB>>>,
) -> Result<TextStream![String], ServiceError> {
    let service = install_service.lock()?;
    //let buf = Vec::new();
    let mut w = BufWriter::new(Vec::new());
    service.install(id, &mut w)?;

    let ts = TextStream! {
        for b in w.into_inner() {
            yield String::from_utf8(b).unwrap();
        }
    };
    Ok(ts)
}
