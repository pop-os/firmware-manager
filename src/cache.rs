use std::{
    io,
    path::{Path, PathBuf},
};

/// An error that may occur when attempting to get the cache directory.
#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to get XDG base directory")]
    BaseDirectory(#[from] xdg::BaseDirectoriesError),
    #[error("failed to get cache directory")]
    Place(#[from] io::Error),
}

/// Fetches the XDG cache directory for com.system76.FirmwareManager
pub fn cache<P: AsRef<Path>>(file: P) -> Result<PathBuf, Error> {
    xdg::BaseDirectories::with_prefix("com.system76.FirmwareManager")
        .map_err(Error::BaseDirectory)?
        .place_cache_file(file)
        .map_err(Error::Place)
}
