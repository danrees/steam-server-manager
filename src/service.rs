use std::io::Write;

use anyhow::Result;

use crate::{
    install::{self, Server, ServerStorage},
    steam_apps::{self, App, Db},
};

pub struct InstallService<S: ServerStorage> {
    client: install::Client,
    storage: S,
}

impl<S: ServerStorage> InstallService<S> {
    pub fn new(steam_cmd: &str, storage: S) -> Self {
        InstallService {
            client: install::Client::new(steam_cmd),
            storage,
        }
    }

    pub fn new_server(&self, server: &Server) -> Result<()> {
        //let server = Server::new(id, name, login, name);
        self.storage.save(&server)?;
        Ok(())
    }

    pub fn get_server(&self, id: i32) -> Result<Server> {
        self.storage.load(id)
    }

    pub fn list_servers(&self) -> Result<Vec<Server>> {
        self.storage.list()
    }

    pub fn install<W: Write>(&self, id: i32, send: W) -> Result<()> {
        let server = self.storage.load(id)?;
        self.client.install(&server, send)?;
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
    pub async fn generate(&self, db: Db) -> Result<()> {
        self.client.generate_applist(db).await
    }

    pub async fn search(&self, name: &str, db: Db) -> Result<Vec<App>> {
        self.client.search(name, db).await
    }
}
