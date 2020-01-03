use std::fmt;
use std::io;
use std::path::PathBuf;

/// An error happened during build.
#[derive(Debug)]
pub enum CacheErrorKind {
    CannotFindUserHomeDirectory(),
    CannotCreateCacheDirectory(PathBuf, io::Error),
    CacheShouldBeADirectory(PathBuf),
    UnknownDfxVersion(String),
}

impl fmt::Display for CacheErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CacheErrorKind::*;

        match self {
            CannotFindUserHomeDirectory() => f.write_str("Cannot find the home directory."),
            CannotCreateCacheDirectory(path, io_err) => f.write_fmt(format_args!(
                r#"Could not create the cache folder at "{}". Error: {}"#,
                path.display(),
                io_err,
            )),

            CacheShouldBeADirectory(path) => f.write_fmt(format_args!(
                r#"Cache folder "{}" should be a directory or a symlink to a directory."#,
                path.display(),
            )),

            UnknownDfxVersion(version) => f.write_fmt(format_args!("Unknown version: {}", version)),
        }
    }
}
