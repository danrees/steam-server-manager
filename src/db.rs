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
    pub fn list_servers(&self) -> anyhow::Result<Vec<Server>> {
        use crate::schema::servers::dsl::*;
        let results = servers.load::<Server>(&self.conn)?;
        Ok(results)
    }
}

impl ServerStorage for DB {
    fn save(&self, server: &Server) -> anyhow::Result<()> {
        use crate::schema::servers::dsl::*;
        insert_into(servers).values(server).execute(&self.conn)?;
        Ok(())
    }
    fn load(&self, id: i32) -> anyhow::Result<Server> {
        use crate::schema::servers::dsl::*;
        let server = servers.find(id).first::<Server>(&self.conn)?;
        Ok(server)
    }
    fn list(&self) -> anyhow::Result<Vec<Server>> {
        use crate::schema::servers::dsl::*;
        let results = servers.load::<Server>(&self.conn)?;
        Ok(results)
    }
}

// pub struct Storage {
//     conn: Connection,
// }

// pub fn open() -> StorageResult<Storage> {
//     let conn = Connection::open("steam_servers.db")?;

//     conn.execute(
//         "create table if not exists servers (
//             id integer primary key,
//             name text not null unique,
//             login text not null,
//             install_dir text not null,
//     )",
//         [],
//     )?;

//     Ok(Storage { conn })
// }

// impl Storage {
//     pub fn new(&self, server: &Server) -> StorageResult<()> {
//         self.conn.execute(
//             "INSERT into servers (id,name,login, install_dir) values (?1,?2,?3,?4)",
//             params![server.id, server.name, server.login, server.install_dir],
//         )?;
//         Ok(())
//     }
//     pub fn save(&self, server: &Server) -> StorageResult<()> {
//         self.conn.execute(
//             "UPDATE servers SET name = ?2, login = ?3, install_dir = ?4, where id = ?1",
//             params![server.id, server.name, server.login, server.install_dir],
//         )?;
//         Ok(())
//     }
//     pub fn get(&self, id: u32) -> StorageResult<Server> {
//         let server: Server = self.conn.query_row(
//             "SELECT id,name,login,install_dir from servers where id = ?",
//             [id],
//             |row| {
//                 Ok(Server {
//                     id: row.get(0)?,
//                     name: row.get(1)?,
//                     login: row.get(2)?,
//                     install_dir: row.get(3)?,
//                 })
//             },
//         )?;
//         Ok(server)
//     }

//     pub fn list(&self) -> StorageResult<Vec<Server>> {
//         let mut stmt = self
//             .conn
//             .prepare("SELECT id,name,login,install_dir from servers")?;
//         let results = stmt.query_map([], |row| {
//             Ok(Server {
//                 id: row.get(0)?,
//                 name: row.get(1)?,
//                 login: row.get(2)?,
//                 install_dir: row.get(3)?,
//             })
//         })?;

//         let mut servers = Vec::new();
//         for s in results {
//             servers.push(s?)
//         }
//         Ok(servers)
//     }
//     pub fn delete(&self, id: u32) -> StorageResult<()> {
//         self.conn
//             .execute("DELETE from servers where id = ?", [id])?;
//         Ok(())
//     }
// }
