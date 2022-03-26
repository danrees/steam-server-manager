use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufWriter, Write};

#[derive(Serialize, Deserialize)]
pub struct App {
    name: String,
    appid: u32,
}

#[derive(Serialize, Deserialize)]
pub struct AppList {
    apps: Vec<App>,
}
#[derive(Serialize, Deserialize)]
pub struct Response {
    applist: AppList,
}

pub struct Client {
    url: String,
    app_list_location: String,
}

impl Client {
    pub fn new(url: String, app_list_location: String) -> Self {
        Client {
            url,
            app_list_location,
        }
    }

    pub async fn generate_applist(&self) -> anyhow::Result<()> {
        let body = reqwest::get(&self.url).await?.text().await?;
        let jsonified = serde_json::to_string_pretty(&body)?;
        let mut output = File::create(&self.app_list_location)?;
        output.write_all(jsonified.as_bytes())?;
        Ok(())
    }

    pub fn search(&self, name: String) -> anyhow::Result<Option<App>> {
        let matcher = Regex::new(&name)?;
        let app_file = File::open(&self.app_list_location)?;
        let apps: Response = serde_json::from_reader(&app_file)?;
        let app = apps
            .applist
            .apps
            .into_iter()
            .find(|a| matcher.is_match(a.name.as_str()));

        Ok(app)
    }
}
