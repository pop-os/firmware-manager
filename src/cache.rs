use std::{
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "failed to get XDG base directory")]
    BaseDirectory(#[error(cause)] xdg::BaseDirectoriesError),
    #[error(display = "failed to get cache directory")]
    Place(#[error(cause)] io::Error),
}

pub fn cache<P: AsRef<Path>>(file: P) -> Result<PathBuf, Error> {
    xdg::BaseDirectories::with_prefix("com.system76.FirmwareManager")
        .map_err(Error::BaseDirectory)?
        .place_cache_file(file)
        .map_err(Error::Place)
}
