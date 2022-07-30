use crate::install::{Server, ServerStorage};

use diesel::{insert_into, prelude::*, Connection};
// use diesel::sqlite::SqliteConnection;
//use rusqlite::{params, Connection};

pub struct DB {
    conn: SqliteConnection,
}

impl DB {
    pub fn establish_connection(url: &str) -> Result<DB, anyhow::Error> {
        let conn = SqliteConnection::establish(url)?;
        Ok(DB { conn })
    }
}

impl ServerStorage for DB {
    fn save(&self, server: &Server) -> anyhow::Result<()> {
        use crate::schema::servers::dsl::*;
        insert_into(servers).values(server).execute(&self.conn)?;
        Ok(())
    }
    fn load(&self, server_id: i32) -> anyhow::Result<Server> {
        use crate::schema::servers::dsl::*;
        let server = servers.find(server_id).first::<Server>(&self.conn)?;
        Ok(server)
    }
    fn list(&self) -> anyhow::Result<Vec<Server>> {
        use crate::schema::servers::dsl::*;
        let results = servers.load::<Server>(&self.conn)?;
        Ok(results)
    }
}
