use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    pub steamcmd_location: String,
    pub steam_api_url: String,
    pub database_url: String,
    pub base_dir: String,
}
