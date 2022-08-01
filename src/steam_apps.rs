use crate::schema::steam_apps;
use diesel::{insert_into, query_dsl::methods::FilterDsl};
use diesel::{Insertable, Queryable};
use diesel::{RunQueryDsl, TextExpressionMethods};
use rocket_sync_db_pools::{database, diesel};

use serde::{Deserialize, Serialize};

#[database("sqlite_db")]
pub struct Db(diesel::SqliteConnection);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Queryable, Insertable)]
#[table_name = "steam_apps"]
pub struct App {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct AppList {
    pub apps: Vec<App>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Response {
    pub applist: AppList,
}

#[derive(Default)]
pub struct Client {
    url: String,
}

impl Client {
    pub fn new(url: &str) -> Self {
        Client { url: url.into() }
    }

    pub async fn generate_applist(&self, db: Db) -> anyhow::Result<()> {
        use crate::schema::steam_apps::dsl::*;
        let path = "/ISteamApps/GetAppList/v2/";

        let body = reqwest::get(format!("{}{}", &self.url, path))
            .await?
            .text()
            .await?;

        let records: AppList = serde_json::from_str(&body)?;
        let apps = records.apps.clone();
        db.run(move |conn| {
            apps.iter().for_each(|a| {
                insert_into(steam_apps)
                    .values(a)
                    .on_conflict()
                    .do_nothing()
                    .execute(conn)
            });
            insert_into(steam_apps)
                .values(&apps)
                // .on_conflict(id)
                // .do_update()
                // .set(&apps)
                .execute(conn)
        })
        .await?;
        Ok(())
    }

    pub async fn search(&self, app_name: &str, db: Db) -> anyhow::Result<Vec<App>> {
        use crate::schema::steam_apps::dsl::*;
        let use_name: String = app_name.into();
        let apps = db
            .run(move |conn| steam_apps.filter(name.like(use_name)).load(conn))
            .await?;

        Ok(apps)
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;

    use super::*;
    use anyhow::Ok;
    use mockito::mock;
    use tempfile::NamedTempFile;

    //static test_body: &'static str = r#"{ "applist": { "apps": [ { "appid": 612440, "name": "The Cable Center - Virtual Archive" }, { "appid": 612470, "name": "Bio Inc. Redemption" }, { "appid": 612480, "name": "Arma 3 DLC Bundle 2" } ] } }"#;

    #[tokio::test]
    async fn test_generate_applist() -> Result<(), anyhow::Error> {
        let _m = mock("GET", "/ISteamApps/GetAppList/v2/")
            .with_status(200)
            .with_body_from_file(Path::new("./test_data/applist.json"))
            .create();

        let mut output_file = NamedTempFile::new()?;
        let mut output_file2 = output_file.reopen()?;

        let client = Client::new(&mockito::server_url());
        {
            //client.generate_applist().await?;
        }

        let mut output = String::new();
        output_file2.read_to_string(&mut output)?;
        //let o = unescape(&output).expect("wasn't escaped");
        let output: Response = serde_json::from_str(output.as_str())?;
        let expected: Response =
            serde_json::from_reader(File::open(Path::new("./test_data/applist.json"))?)?;
        //println!("{}", expected);
        assert_eq!(expected, output, "Expected certain contents");
        Ok(())
    }

    #[tokio::test]
    async fn test_search_found() -> Result<(), anyhow::Error> {
        let mut input = File::open(Path::new("./test_data/applist.json"))?;
        let client = Client::new("https://example.com".into());

        // TODO: Reimplement this test
        // let answer = client.search("Arma 3", db).await?;

        // assert_eq!(answer.len(), 1, "Should be one answer to the search");
        // assert_eq!(answer.get(0).unwrap().name, "Arma 3 DLC Bundle 2");

        Ok(())
    }

    #[tokio::test]
    async fn test_search_not_found() -> Result<(), anyhow::Error> {
        let mut input = File::open(Path::new("./test_data/applist.json"))?;
        let client = Client::new("https://example.com".into());

        // TODO: Reimplement this
        // let answer = client.search("Doesn't exist", false).await?;

        // assert_eq!(answer.len(), 0, "Should not be an answer to the search");

        Ok(())
    }
}
