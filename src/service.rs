use std::io::Write;

use anyhow::Result;
use log::debug;

use crate::{
    db::{DBStorage, Db},
    install::{self, Server, ServerStorage},
    steam_apps::{self, App},
};

pub struct InstallService {
    client: install::Client,
    storage: DBStorage,
}

impl InstallService {
    pub fn new(steam_cmd: &str, storage: DBStorage) -> Self {
        InstallService {
            client: install::Client::new(steam_cmd),
            storage,
        }
    }

    pub async fn new_server(&self, server: &Server, db: Db) -> Result<()> {
        //let server = Server::new(id, name, login, name);
        self.storage.save(&server, db).await?;
        Ok(())
    }

    pub async fn get_server(&self, id: i32, db: Db) -> Result<Server> {
        self.storage.load(id, db).await
    }

    pub async fn list_servers(&self, db: Db) -> Result<Vec<Server>> {
        self.storage.list(db).await
    }

    pub async fn install(
        &self,
        id: i32,
        send: std::sync::mpsc::Sender<String>,
        db: Db,
    ) -> Result<()> {
        let server = self.storage.load(id, db).await?;
        self.client.install(&server, send).await?;
        debug!("installed appliction with id: {}", id);
        Ok(())
    }
}

pub struct SteamAppsService {
    client: steam_apps::Client,
}

impl SteamAppsService {
    pub fn new(url: &str) -> Self {
        SteamAppsService {
            client: steam_apps::Client::new(url),
        }
    }
    pub async fn generate(&self, db: steam_apps::Db) -> Result<()> {
        self.client.generate_applist(db).await
    }

    pub async fn search(&self, name: &str, db: steam_apps::Db) -> Result<Vec<App>> {
        self.client.search(name, db).await
    }
}
