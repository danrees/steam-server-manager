use anyhow::Error;
use config::Config;
use rocket::{
    form::Form,
    http::Status,
    response::{
        self,
        status::{self, NotFound},
    },
    serde::json::Json,
    State,
};
use service::SteamAppsService;
use std::process::Command;
use steam_apps::App;

//use serde::{Deserialize, Serialize};

mod install;
mod service;
mod steam_apps;
mod storage;
mod types;

#[derive(Debug, Responder)]
#[response(status = 500, content_type = "json")]
struct ServiceError(String);

impl From<anyhow::Error> for ServiceError {
    fn from(err: anyhow::Error) -> Self {
        ServiceError(format!("{}", err))
    }
}

#[macro_use]
extern crate rocket;

#[derive(FromForm)]
struct SearchTerms<'a> {
    term: &'a str,
    case_insensitive: bool,
}

#[post("/search", data = "<term>")]
fn search_apps(
    term: Form<SearchTerms<'_>>,
    steam_apps_service: &State<SteamAppsService>,
) -> Result<Json<Vec<App>>, ServiceError> {
    let app = steam_apps_service.search(term.term, term.case_insensitive)?;

    Ok(Json(app))
}

#[post("/generate")]
async fn generate_apps(steam_apps_service: &State<SteamAppsService>) -> Result<(), ServiceError> {
    steam_apps_service.generate().await.map_err(|e| e.into())
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings: types::ServerConfig = Config::builder()
        .add_source(config::File::with_name("./config.toml"))
        .add_source(config::Environment::with_prefix("STEAM"))
        .set_default("steamcmd_location", "./steamcmd.sh")?
        .set_default("steam_api_url", "https://api.steampowered.com")?
        .build()?
        .try_deserialize()?;

    let app_service =
        service::SteamAppsService::new(&settings.steam_api_url, "./data/applist.json");
    rocket::build()
        .manage(app_service)
        .manage(settings)
        .mount("/apps", routes![search_apps, generate_apps])
        .launch()
        .await?;
    Ok(())
}
