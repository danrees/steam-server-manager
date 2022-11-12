use anyhow::Result;
use log::debug;
use rocket::fairing::{Fairing, Info, Kind};

use crate::{
    db::{DBStorage, Db},
    handlers::Tx,
    install::{self, InstallQueueRx, Server},
    steam_apps::{self, App},
};

pub struct ServerService {
    storage: DBStorage,
}

impl ServerService {
    pub fn new(storage: DBStorage) -> Self {
        ServerService { storage }
    }

    pub async fn new_server(&self, server: &Server, db: Db) -> Result<()> {
        //let server = Server::new(id, name, login, name);
        self.storage.save(server, db).await?;
        Ok(())
    }

    pub async fn get_server(&self, id: i32, db: Db) -> Result<Server> {
        self.storage.load(id, db).await
    }

    pub async fn list_servers(&self, db: Db) -> Result<Vec<Server>> {
        self.storage.list(db).await
    }

    pub async fn delete(&self, id: i32, db: Db) -> Result<()> {
        self.storage.delete(id, db).await
    }
}

pub struct InstallService {
    client: install::Client,
    base_dir: String,
}

#[rocket::async_trait]
impl Fairing for InstallService {
    fn info(&self) -> rocket::fairing::Info {
        Info {
            name: "Run install watcher on launch",
            kind: Kind::Liftoff,
        }
    }
    async fn on_liftoff(&self, rocket: &rocket::Rocket<rocket::Orbit>) {
        let rx = rocket.state::<InstallQueueRx>().unwrap();
        let output = rocket.state::<Tx>().unwrap();
        self.run(&rx.0, &output.0).await.unwrap();
    }
}

impl InstallService {
    pub fn new(steam_cmd: &str, base_dir: &str) -> Self {
        let client = install::Client::new(steam_cmd.into());
        InstallService {
            client: client,
            base_dir: base_dir.into(),
        }
    }

    pub async fn run(
        &self,
        rx: &flume::Receiver<Server>,
        output: &flume::Sender<String>,
    ) -> Result<(), anyhow::Error> {
        for server in rx {
            debug!("Recieved request to install {:?}", server);
            if let Err(e) = self.client.install(&self.base_dir, &server, output).await {
                error!("problem installing: {}", e)
            };
        }
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
