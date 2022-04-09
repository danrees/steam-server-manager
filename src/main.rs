use config::Config;
use std::process::Command;

//use serde::{Deserialize, Serialize};

mod install;
mod steam_apps;
mod storage;
mod types;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings: types::ServerConfig = Config::builder()
        .add_source(config::File::with_name("./config.toml"))
        .add_source(config::Environment::with_prefix("STEAM"))
        .set_default("steamcmd_location", "./steamcmd.sh")?
        .build()?
        .try_deserialize()?;

    let steamcmd_cmd = settings.steamcmd_location;
    let output = Command::new(steamcmd_cmd).arg("--help").output()?;
    String::from_utf8(output.stdout)?
        .lines()
        .for_each(|l| println!("{}", l));

    // let client = Client::new("https://api.steampowered.com".into());
    // let mut output = File::create("output.json")?;

    // client.generate_applist(&mut output).await?;
    Ok(())
}
