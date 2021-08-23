use crate::cache;

use std::{
    fs, io,
    path::PathBuf,
    time::{Duration, SystemTime},
};

/// An error that may occur when reading or writing the timestamp file.
#[derive(Debug, Error)]
pub enum Error {
    #[error("cache error")]
    Cache(#[from] cache::Error),
    #[error("failed to read last update file")]
    Read(#[source] io::Error),
    #[error("failed to write timestamp")]
    Write(#[source] io::Error),
    #[error("failed to create cache directory for timestamp file")]
    Parent(#[source] io::Error),
}

/// Fetches the timestamp that is currently stored in the cache.
pub fn last() -> Result<u64, Error> {
    let path = &*timestamp_path()?;

    fs::read_to_string(path)
        .map_err(Error::Read)
        .map(|string| string.trim().parse::<u64>().unwrap_or(0))
}

/// Refreshes the timestamp in the cache.
pub fn refresh() -> Result<(), Error> {
    trace!("refreshing the timestamp file in cache");
    let path = &*timestamp_path()?;

    let parent = path.parent().expect("timestmap file does not have a parent directory");
    fs::create_dir_all(parent).map_err(Error::Parent)?;
    fs::write(path, current().to_string()).map_err(Error::Write)
}

/// Determines if the time since the last recorded timestamp has exceeded `seconds`.
pub fn exceeded(seconds: u64) -> Result<bool, Error> {
    last().map(|last| time_exceeded(last, current(), seconds))
}

/// Convenience function for fetching the current time in seconds since the UNIX Epoch.
pub fn current() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .as_ref()
        .map(Duration::as_secs)
        .unwrap_or(0)
}

fn time_exceeded(last: u64, current: u64, limit: u64) -> bool {
    current == 0 || last > current || current - last > limit
}

/// Convenience function for fetching the path to the timestamp file.
fn timestamp_path() -> Result<PathBuf, Error> {
    cache::cache("last_refresh").map_err(Error::Cache)
}

#[cfg(test)]
mod tests {
    #[test]
    fn time_exceeded() {
        assert!(super::time_exceeded(0, 0, 500));
        assert!(super::time_exceeded(124512, 0, 500));
        assert!(super::time_exceeded(0, 124512, 500));
        assert!(super::time_exceeded(1000, 2000, 500));
        assert!(super::time_exceeded(1000, 1501, 500));
        assert!(!super::time_exceeded(1000, 1250, 500));
    }
}
