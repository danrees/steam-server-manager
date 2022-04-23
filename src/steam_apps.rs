use regex::RegexBuilder;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct App {
    pub name: String,
    pub appid: u32,
}

impl App {
    pub fn new(name: String, appid: u32) -> Self {
        App { name, appid }
    }
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
    pub fn new(url: String) -> Self {
        Client { url }
    }

    pub async fn generate_applist<W: std::io::Write>(&self, w: &mut W) -> anyhow::Result<()> {
        let path = "/ISteamApps/GetAppList/v2/";

        let body = reqwest::get(format!("{}{}", &self.url, path))
            .await?
            .text()
            .await?;
        //let jsonified = serde_json::to_string(&body)?;
        w.write_all(body.as_bytes())?;
        w.flush()?;
        Ok(())
    }

    pub fn search<R: std::io::Read>(
        &self,
        r: &mut R,
        name: &str,
        case_insensitive: bool,
    ) -> anyhow::Result<Vec<App>> {
        //let matcher = Regex::new(&name)?;
        let matcher = RegexBuilder::new(name)
            .case_insensitive(case_insensitive)
            .build()?;
        //let app_file = File::open(&self.app_list_location)?;
        let apps: Response = serde_json::from_reader(r)?;
        let app = apps
            .applist
            .apps
            .into_iter()
            .filter(|a| matcher.is_match(a.name.as_str()))
            .collect();

        Ok(app)
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

        let client = Client::new(mockito::server_url());
        {
            client
                .generate_applist::<File>(&mut output_file.as_file_mut())
                .await?;
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

    #[test]
    fn test_search_found() -> Result<(), anyhow::Error> {
        let mut input = File::open(Path::new("./test_data/applist.json"))?;
        let client = Client::new("https://example.com".into());

        let answer = client.search(&mut input, "Arma 3", false)?;

        assert_eq!(answer.len(), 1, "Should be one answer to the search");
        assert_eq!(answer.get(0).unwrap().name, "Arma 3 DLC Bundle 2");

        Ok(())
    }

    #[test]
    fn test_search_not_found() -> Result<(), anyhow::Error> {
        let mut input = File::open(Path::new("./test_data/applist.json"))?;
        let client = Client::new("https://example.com".into());

        let answer = client.search(&mut input, "Doesn't exist", false)?;

        assert_eq!(answer.len(), 0, "Should not be an answer to the search");

        Ok(())
    }
}
