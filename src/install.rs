use std::{
    io::{BufRead, BufReader, Error, Write},
    process::{Child, Command, Stdio},
};

pub struct Client {
    steamd_cmd: String,
}

pub struct Server<'a> {
    id: u32,
    login: &'a str,
    install_dir: &'a str,
}

pub struct SteamCommand<'a> {
    command: &'a str,
    args: &'a [&'a str],
}

impl Client {
    pub fn new(steamd_cmd: String) -> Self {
        Self { steamd_cmd }
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

    pub fn install(&self, server: &Server, mut w: Box<dyn Write>) -> anyhow::Result<()> {
        let install_dir = [server.install_dir];
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

        let reader =
            BufReader::new(proc.stdout.ok_or_else(|| {
                Error::new(std::io::ErrorKind::Other, "Could not capture stdout")
            })?);

        let lines = reader.lines().filter_map(|line| line.ok());
        for l in lines {
            w.write_all(l.as_bytes())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_run() -> anyhow::Result<()> {
        let client: Client = Client::new(String::from("echo"));
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
