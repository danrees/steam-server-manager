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

pub mod apps;
pub mod server;
pub mod test;

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
