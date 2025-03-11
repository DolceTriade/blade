use std::io::Write;
use std::{fs::OpenOptions, net::TcpListener};

use anyhow::anyhow;

pub struct PgHarness {
    data_path: std::path::PathBuf,
    postgres: std::process::Child,
    db_name: String,
    port: u16,
}

impl PgHarness {
    pub fn uri(&self) -> String {
        format!("postgres://localhost:{}/{}", self.port, self.db_name)
    }

    pub fn data_path(&self) -> std::path::PathBuf {
        self.data_path.clone()
    }

    pub fn close(&mut self) -> anyhow::Result<()> {
        self.postgres.wait()?;
        Ok(())
    }
}

impl Drop for PgHarness {
    fn drop(&mut self) {
        rustix::process::kill_process(
            rustix::process::Pid::from_child(&self.postgres),
            rustix::process::Signal::Term,
        )
        .unwrap();
        self.close().unwrap();
    }
}

#[allow(unused_assignments)]
pub fn new(p: &str) -> anyhow::Result<PgHarness> {
    let r = runfiles::Runfiles::create()?;
    let bin_path = r.rlocation("postgresql-bin/bin").unwrap();
    let initdb_path = bin_path.join("initdb").read_link().unwrap();
    let initdb = std::process::Command::new(initdb_path).arg(p).status()?;
    if !initdb.success() {
        return Err(anyhow!("failed to init db: {:#?}", initdb));
    }
    let config_path = std::path::PathBuf::from(p).join("postgresql.conf");
    let mut port = 0;

    {
        let lis = TcpListener::bind("127.0.0.1:0")?;
        let addr = lis.local_addr()?;

        let mut f = OpenOptions::new().append(true).open(config_path)?;
        port = addr.port();
        writeln!(f, "\nport = {}", addr.port())?;
    }
    let postgres_path = bin_path.join("postgres").read_link().unwrap();
    let harness = PgHarness {
        data_path: p.into(),
        postgres: std::process::Command::new(postgres_path)
            .arg("-D")
            .arg(p)
            .arg("-k")
            .arg(p)
            .spawn()?,
        db_name: "blade".into(),
        port,
    };
    let mut attempts = 3;
    let createdb_path = bin_path.join("createdb");
    while attempts > 0 {
        if std::process::Command::new(createdb_path.clone())
            .arg("-h")
            .arg("127.0.0.1")
            .arg("-p")
            .arg(harness.port.to_string())
            .arg("blade")
            .status()?
            .success()
        {
            break;
        }
        attempts -= 1;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    Ok(harness)
}

#[cfg(test)]
mod test {

    #[test]
    fn test_new() {
        let r = runfiles::Runfiles::create().unwrap();
        let bin_path = r.rlocation("postgresql-bin/bin").unwrap();
        let psql_path = bin_path.join("psql").read_link().unwrap();
        let tmp = tempdir::TempDir::new("test_new").unwrap();
        let path = tmp.path().to_str().unwrap();
        let h = super::new(path).unwrap();
        let uri = h.uri();
        let u = uri.split('/').collect::<Vec<_>>();
        let db_name = u.last().unwrap();
        let port = u[2].split(':').last().unwrap();
        let o = std::process::Command::new(psql_path)
            .arg("-l")
            .arg("-p")
            .arg(port)
            .arg("-d")
            .arg(db_name)
            .output()
            .unwrap();
        println!("{o:#?}");
    }
}
