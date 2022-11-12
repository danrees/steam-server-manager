use std::{path::Path, process::Stdio};

use crate::schema::*;
use anyhow::Result;
use diesel::Queryable;
use log::debug;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader, Error};
use tokio::process::{Child, Command};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Queryable, Insertable)]
pub struct Server {
    pub id: i32,
    pub name: String,
    pub login: String,
    pub install_dir: String,
}

pub struct InstallQueueTx(pub flume::Sender<Server>);
pub struct InstallQueueRx(pub flume::Receiver<Server>);

impl Server {
    pub fn new(id: i32, name: &str, login: &str, install_dir: &str) -> Self {
        Server {
            id,
            name: name.into(),
            login: login.into(),
            install_dir: install_dir.into(),
        }
    }
}

pub trait ServerStorage {
    fn save(&self, server: &Server) -> Result<()>;
    fn load(&self, server_id: i32) -> Result<Server>;
    fn list(&self) -> Result<Vec<Server>>;
}

pub struct SteamCommand<'a> {
    command: &'a str,
    args: &'a [&'a str],
}
pub struct Client {
    steamd_cmd: String,
}

impl Client {
    pub fn new(steamd_cmd: &str) -> Self {
        Self {
            steamd_cmd: steamd_cmd.into(),
        }
    }

    fn run(&self, commands: &Vec<SteamCommand>) -> anyhow::Result<Child> {
        let mut p = Command::new(&self.steamd_cmd);
        p.kill_on_drop(true);
        p.stdout(Stdio::piped()).stderr(Stdio::piped());
        for c in commands {
            p.arg(format!("{} {}", c.command, c.args.join(" ")));
        }

        //debug!("{:?}", p.);
        let output = p.spawn()?;
        //let handle = p.spawn()?;

        Ok(output)
    }

    pub async fn install(
        &self,
        base_dir: &str,
        server: &Server,
        sender: &flume::Sender<String>,
    ) -> anyhow::Result<()> {
        let path = Path::new(base_dir)
            .join(server.install_dir.as_str())
            .display()
            .to_string();

        let install_dir = [path.as_str()];
        let app_update = [&server.id.to_string(), "validate"];
        let commands = vec![
            SteamCommand {
                command: "+force_install_dir",
                args: &install_dir,
            },
            SteamCommand {
                command: "+login",
                args: &["anonymous"],
            },
            SteamCommand {
                command: "+app_update",
                args: &app_update,
            },
            SteamCommand {
                command: "+exit",
                args: &[],
            },
        ];
        debug!("running steamcmd install for application {}", server.name);
        let proc = self.run(&commands)?;

        let output = proc
            .stdout
            .ok_or_else(|| Error::new(std::io::ErrorKind::Other, "Could not capture stdout"))?;

        // let reader =
        //     BufReader::new(proc.stdout.ok_or_else(|| {
        //         Error::new(std::io::ErrorKind::Other, "Could not capture stdout")
        //     })?);
        //let writer = BufWriter::new(sender);
        //let lines = reader.lines().filter_map(|line| line.ok());
        let mut reader = BufReader::new(output).lines();

        // tokio::spawn(async {
        //     let status = proc.wait().await.expect("couldn't process exit status");
        //     debug!("child status was {}", status);
        // });

        while let Some(l) = reader.next_line().await? {
            log::debug!("line: {}", l);
            sender.send_async(l).await?;
        }
        // for line in reader.lines() {
        //     match line {
        //         Ok(l) => {
        //             log::debug!("line: {}", l);
        //             sender.send_async(l).await?
        //         }
        //         Err(e) => return Err(anyhow::anyhow!(e)),
        //     }
        // }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_run() -> anyhow::Result<()> {
        let client: Client = Client::new("echo");
        let commands = vec![SteamCommand {
            command: "hello",
            args: &["world"],
        }];
        let output = client.run(&commands)?.wait_with_output().await?;
        let output_str = String::from_utf8(output.stdout)?;

        assert_eq!(String::from("hello world"), output_str.trim_end());
        Ok(())
    }
}
