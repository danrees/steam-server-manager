use rocket::{form::Form, serde::json::Json, State};

use crate::{
    service::SteamAppsService,
    steam_apps::{self, App},
};

use super::{SearchTerms, ServiceError};

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
