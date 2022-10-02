use crate::{db, install::Server, service::InstallService};
use rocket::{
    response::stream::{Event, EventStream},
    serde::json::Json,
    State,
};

use super::{Rx, ServiceError, Tx};

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
    tx: &State<Tx>,
) -> Result<Json<Server>, ServiceError> {
    tx.0.send_async(String::from("got")).await.unwrap();
    install_service
        .get_server(id, db)
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
    let tx_clone = &tx.0;
    tx_clone
        .send_async(String::from("A first message"))
        .await
        .unwrap();
    let s = install_service.install(id, &tx_clone, db);
    debug!("installing {}", id);
    s.await?;
    Ok(())
}

#[get("/install/events")]
pub fn install_events(rx: &State<Rx>) -> EventStream![Event + '_] {
    let receiver = &rx.0;
    debug!("events called");
    EventStream! {
        while let Ok(msg) = receiver.recv_async().await {
            debug!("received: {}", msg);
            yield Event::data(msg);
        }
    }
}
