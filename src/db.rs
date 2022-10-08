use crate::install::Server;

use diesel::{delete, insert_into, prelude::*};
use rocket_sync_db_pools::database;
// use diesel::sqlite::SqliteConnection;
//use rusqlite::{params, Connection};

// pub struct DB {
//     conn: SqliteConnection,
// }

pub struct DBStorage {}

#[database("sqlite_db")]
pub struct Db(diesel::SqliteConnection);

// impl DB {
//     pub fn establish_connection(url: &str) -> Result<DB, anyhow::Error> {
//         let conn = SqliteConnection::establish(url)?;
//         Ok(DB { conn })
//     }
// }

impl DBStorage {
    pub async fn save(&self, server: &Server, db: Db) -> anyhow::Result<()> {
        use crate::schema::servers::dsl::*;
        let save_server = server.clone();
        db.run(move |conn| insert_into(servers).values(save_server).execute(conn))
            .await?;
        Ok(())
    }
    pub async fn load(&self, server_id: i32, db: Db) -> anyhow::Result<Server> {
        use crate::schema::servers::dsl::*;
        let server = db
            .run(move |conn| servers.find(server_id).first::<Server>(conn))
            .await?;
        Ok(server)
    }
    pub async fn list(&self, db: Db) -> anyhow::Result<Vec<Server>> {
        use crate::schema::servers::dsl::*;
        let results = db.run(move |conn| servers.load::<Server>(conn)).await?;
        Ok(results)
    }

    pub async fn delete(&self, server_id: i32, db: Db) -> anyhow::Result<()> {
        use crate::schema::servers::dsl::*;
        db.run(move |conn| delete(servers.filter(id.eq(server_id))).execute(conn))
            .await?;

        Ok(())
    }
}
