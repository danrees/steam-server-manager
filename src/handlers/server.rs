use super::{Rx, ServiceError, Tx};
use crate::{
    db,
    install::{InstallQueueTx, Server},
    service::ServerService,
};
use rocket::{
    response::stream::{Event, EventStream},
    serde::json::Json,
    State,
};

#[get("/")]
pub async fn list_servers(
    server_service: &State<ServerService>,
    db: db::Db,
) -> Result<Json<Vec<Server>>, ServiceError> {
    server_service
        .list_servers(db)
        .await
        .map(Json)
        .map_err(|e| e.into())
}

#[post("/", data = "<server>")]
pub async fn create_server(
    server: Json<Server>,
    // TODO: How can I still do this generically
    server_service: &State<ServerService>,
    db: db::Db,
) -> Result<(), ServiceError> {
    let service = server_service;
    service.new_server(&server, db).await?;
    Ok(())
}

#[get("/<id>")]
pub async fn get_server(
    id: i32,
    server_service: &State<ServerService>,
    db: db::Db,
    tx: &State<Tx>,
) -> Result<Json<Server>, ServiceError> {
    tx.0.send_async(String::from("got")).await.unwrap();
    server_service
        .get_server(id, db)
        .await
        .map(Json)
        .map_err(|e| e.into())
}

#[delete("/<id>")]
pub async fn delete(
    id: i32,
    server_service: &State<ServerService>,
    db: db::Db,
) -> Result<(), ServiceError> {
    server_service.delete(id, db).await.map_err(|e| e.into())
}

#[post("/install/<id>")]
pub async fn install(
    id: i32,
    server_service: &State<ServerService>,
    install_queue: &State<InstallQueueTx>,
    db: db::Db,
) -> Result<(), ServiceError> {
    let server = server_service.get_server(id, db).await?;
    let queue = &install_queue.0;

    queue
        .send(server)
        .map_err(|e| ServiceError(format!("{}", e)))?;

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
