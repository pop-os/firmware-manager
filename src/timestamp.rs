use crate::cache;

use std::{
    fs, io,
    path::PathBuf,
    time::{Duration, SystemTime},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "cache error")]
    Cache(#[error(cause)] cache::Error),
    #[error(display = "failed to read last update file")]
    LastUpdate(#[error(cause)] io::Error),
    #[error(display = "failed to write timestamp")]
    LastUpdateWrite(#[error(cause)] io::Error),
    #[error(display = "failed to create cache directory for timestamp file")]
    Parent(#[error(cause)] io::Error),
}

pub fn last() -> Result<u64, Error> {
    let path = &*timestamp_path()?;

    fs::read_to_string(path)
        .map_err(Error::LastUpdate)
        .map(|string| string.trim().parse::<u64>().unwrap_or(0))
}

pub fn refresh() -> Result<(), Error> {
    let path = &*timestamp_path()?;

    let parent = path.parent().expect("timestmap file does not have a parent directory");
    fs::create_dir_all(parent).map_err(Error::Parent)?;
    fs::write(path, current().to_string()).map_err(Error::LastUpdateWrite)
}

pub fn exceeded(seconds: u64) -> Result<bool, Error> {
    last().map(|last| {
        let current = current();
        current == 0 || last > current || current - last > seconds
    })
}

pub fn current() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .as_ref()
        .map(Duration::as_secs)
        .unwrap_or(0)
}

fn timestamp_path() -> Result<PathBuf, Error> { cache::cache("timestamp").map_err(Error::Cache) }

// TODO: Add unit tests.
