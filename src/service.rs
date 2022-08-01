use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::Result;

use crate::{
    install::{self, Server, ServerStorage},
    steam_apps::{self, App},
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
    file_name: String,
}

impl SteamAppsService {
    pub fn new(url: &str, file_name: &str) -> Self {
        SteamAppsService {
            client: steam_apps::Client::new(url.to_string()),
            file_name: file_name.to_string(),
        }
    }
    pub async fn generate(&self) -> Result<()> {
        let path = Path::new(&self.file_name);
        let parent = path
            .parent()
            .ok_or(anyhow::anyhow!("path of parent does not exist: {:?}", path))?;
        fs::create_dir_all(parent)?;
        let mut file = File::create(path)?;
        self.client.generate_applist().await
    }

    pub fn search(&self, name: &str, case_insensitive: bool) -> Result<Vec<App>> {
        let mut file = File::open(&self.file_name)?;
        self.client.search(&mut file, name, case_insensitive)
    }
}
