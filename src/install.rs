use std::{
    io::prelude::*,
    io::{BufReader, Error},
    process::{Child, Command, Stdio},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Server {
    id: u32,
    pub name: String,
    login: String,
    install_dir: String,
}

impl Server {
    pub fn new(id: u32, name: &str, login: &str, install_dir: &str) -> Self {
        return Server {
            id,
            name: name.into(),
            login: login.into(),
            install_dir: install_dir.into(),
        };
    }
}

pub trait ServerStorage {
    fn save(&self, server: &Server) -> Result<()>;
    fn load(&self, name: &str) -> Result<Server>;
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
        p.stdout(Stdio::piped()).stderr(Stdio::piped());
        for c in commands {
            p.arg(c.command).args(c.args);
        }
        let output = p.spawn()?;
        //let handle = p.spawn()?;

        return Ok(output);
    }

    pub fn install<W: Write>(&self, server: &Server, mut writer: W) -> anyhow::Result<()> {
        let install_dir = [server.install_dir.as_str()];
        let app_update = [&server.id.to_string(), "validate"];
        let commands = vec![
            SteamCommand {
                command: "+login",
                args: &["anonymous"],
            },
            SteamCommand {
                command: "+force_install_dir",
                args: &install_dir,
            },
            SteamCommand {
                command: "+app_update",
                args: &app_update,
            },
            SteamCommand {
                command: "+quit",
                args: &[],
            },
        ];
        let proc = self.run(&commands)?;

        let mut reader =
            BufReader::new(proc.stdout.ok_or_else(|| {
                Error::new(std::io::ErrorKind::Other, "Could not capture stdout")
            })?);
        //let writer = BufWriter::new(sender);
        //let lines = reader.lines().filter_map(|line| line.ok());
        std::io::copy(&mut reader, &mut writer)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_run() -> anyhow::Result<()> {
        let client: Client = Client::new("echo");
        let commands = vec![SteamCommand {
            command: "hello",
            args: &["world"],
        }];
        let output = client.run(&commands)?.wait_with_output()?;
        let output_str = String::from_utf8(output.stdout)?;

        assert_eq!(String::from("hello world"), output_str.trim_end());
        Ok(())
    }
}
