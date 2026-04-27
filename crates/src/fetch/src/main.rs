use std::{
    env, fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use flate2::read::GzDecoder;
use scraper::{Html, Selector};
use tar::Archive;

const USER_AGENT: &str = concat!(
    "squire-sqlite3-src-fetch/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/silverlyra/squire)"
);

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result {
    let mut args = env::args().skip(1);
    let version = args
        .next()
        .ok_or("usage: squire-sqlite3-src-fetch <version> (e.g. 3.53.0)")?;

    let releases = index()?;
    let release = releases
        .into_iter()
        .find(|r| r.version == version)
        .ok_or_else(|| format!("no release found for version {version}"))?;

    let dest = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("crate manifest dir has no parent")?
        .join("sqlite");

    download(&release, &dest)?;

    eprintln!("extracted {} to {}", release.version, dest.display());
    Ok(())
}

fn index() -> Result<Vec<Release>> {
    let body: String = ureq::get("https://sqlite.org/chronology.html")
        .header("User-Agent", USER_AGENT)
        .call()?
        .body_mut()
        .read_to_string()?;

    let html = Html::parse_document(&body);
    let selector = Selector::parse("table#chrontab > tbody > tr")?;

    let releases = html.select(&selector).filter_map(|row| {
        let mut children = row.child_elements();
        match (children.next(), children.next()) {
            (Some(date), Some(rel)) => {
                let date = date.text().collect::<String>();
                let year = date.split_once('-')?.0;
                let year: i32 = year.parse().ok()?;

                let version = rel.text().collect::<String>().trim().to_owned();

                Some(Release { year, version })
            }
            _ => None,
        }
    });

    Ok(releases.collect())
}

fn download(release: &Release, dest: &Path) -> Result {
    let url = release.download_url()?;
    eprintln!("downloading {url} to {}", dest.display());

    let mut response = ureq::get(&url).header("User-Agent", USER_AGENT).call()?;
    let reader = response.body_mut().as_reader();

    if dest.exists() {
        fs::remove_dir_all(dest)?;
    }
    fs::create_dir_all(dest)?;

    let mut archive = Archive::new(GzDecoder::new(reader));
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let relative_path = path.components().skip(1).collect::<PathBuf>();
        if relative_path.as_os_str().is_empty() {
            continue;
        }

        entry.unpack(dest.join(relative_path))?;
    }

    fs::File::options()
        .write(true)
        .open(dest.join("VERSION"))?
        .set_modified(SystemTime::now())?;

    Ok(())
}

#[derive(Clone, Debug)]
struct Release {
    year: i32,
    version: String,
}

impl Release {
    /// The `https://sqlite.org` URL where this [`Release`] is hosted.
    fn download_url(&self) -> Result<String> {
        let (year, version) = (self.year, &self.version);

        let mut parts = version.split('.');
        let major: u32 = parts.next().ok_or("missing major")?.parse()?;
        let minor: u32 = parts.next().ok_or("missing minor")?.parse()?;
        let patch: u32 = parts.next().map(str::parse).transpose()?.unwrap_or(0);
        let branch: u32 = parts.next().map(str::parse).transpose()?.unwrap_or(0);
        if parts.next().is_some() {
            return Err(format!("unexpected version shape: {}", version).into());
        }
        if major != 3 {
            return Err(format!("only 3.x versions are supported, got {}", version).into());
        }

        Ok(format!(
            "https://sqlite.org/{year}/sqlite-autoconf-{major}{minor:02}{patch:02}{branch:02}.tar.gz"
        ))
    }
}
