use crate::install::{Server, ServerStorage};
use anyhow::Result;
use std::{
    fs::{self, File},
    path::Path,
};

pub struct FileStorage {
    dir: String,
}

impl FileStorage {
    pub fn new(dir: &str) -> Self {
        fs::create_dir_all(dir).unwrap();
        FileStorage { dir: dir.into() }
    }
}

impl ServerStorage for FileStorage {
    fn save(&self, app: &Server) -> Result<()> {
        let path = Path::new(&self.dir).join(&app.name).with_extension("json");
        println!("{:?}", path);
        let file = File::create(path)?;
        serde_json::to_writer(file, app)?;
        Ok(())
    }

    fn load(&self, name: &str) -> Result<Server> {
        let file = File::open(Path::new(&self.dir).join(name).with_extension("json"))?;
        let app: Server = serde_json::from_reader(file)?;
        Ok(app)
    }
}

#[cfg(test)]
mod test {
    use std::fs::{self, create_dir_all};

    use super::*;
    use anyhow::Result;

    fn cleanup(dir: &str) {
        fs::remove_dir_all(dir).unwrap()
    }

    #[test]
    fn test_save() -> Result<()> {
        let dir = "./testdir";
        create_dir_all(dir)?;
        let storage = FileStorage::new(dir);
        let app = Server::new(1, "test-app", "anonymous", "test-app");

        storage.save(&app)?;

        let result = File::open(Path::new(dir).join(&app.name).with_extension("json"))?;
        let result_app: Server = serde_json::from_reader(result)?;

        assert_eq!(app, result_app);
        cleanup(dir);
        Ok(())
    }

    #[test]
    fn test_load() -> Result<()> {
        let dir = "test-dir";
        let app_name = "test-app";
        create_dir_all(dir)?;

        let path = Path::new(dir).join(app_name).with_extension("json");

        fs::write(
            path,
            r#"{"name": "test-app", "id": 1, "login": "anonymous" ,"install_dir": "test-app"}"#,
        )?;

        let storage = FileStorage::new(dir);

        let app = storage.load(app_name)?;

        assert_eq!(app_name, app.name);

        cleanup(dir);
        Ok(())
    }
}
