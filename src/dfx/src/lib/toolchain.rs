use crate::config::cache;
use crate::lib::dist;
use crate::lib::error::{DfxError, DfxResult};

use anyhow::bail;
use semver::{Version, VersionReq};
use std::fmt;
use std::fmt::Formatter;
use std::path::{Path, PathBuf};
use std::str::FromStr;

const TOOLCHAINS_ROOT: &str = ".dfinity/toolchains/";
const DEFAULT_PATH: &str = ".dfinity/default";

#[derive(Debug, Eq, PartialEq)]
pub enum Toolchain {
    /// Complete semver such as '0.6.21'
    CompleteVersionToolchain(Version),

    /// Partial semver such as '0.6', '1.0'
    MajorMinorToolchain(u8, u8),

    /// Tag such as 'latest'
    TagToolchain(String),
}

impl FromStr for Toolchain {
    type Err = DfxError;
    fn from_str(s: &str) -> DfxResult<Self> {
        if let Ok(v) = Version::parse(s) {
            Ok(Toolchain::CompleteVersionToolchain(v))
        } else if VersionReq::parse(s).is_ok()
            && s.chars().all(|c| c.is_ascii_digit() || c == '.')
            && s.split('.').count() == 2
        {
            let major = s.split('.').next().unwrap().parse::<u8>()?;
            let minor = s.split('.').nth(1).unwrap().parse::<u8>()?;
            Ok(Toolchain::MajorMinorToolchain(major, minor))
        } else {
            match s {
                "latest" => Ok(Toolchain::TagToolchain("latest".to_string())),
                _ => bail!("Invalid toolchain name: {}", s),
            }
        }
    }
}

impl fmt::Display for Toolchain {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::CompleteVersionToolchain(v) => write!(f, "{}", v),
            Self::MajorMinorToolchain(major, minor) => write!(f, "{0}.{1}", major, minor),
            Self::TagToolchain(t) => write!(f, "{}", t),
        }
    }
}

impl Toolchain {
    // Update the toolchain, install it if nonexisting
    pub fn update(&self) -> DfxResult<()> {
        eprintln!("Syncing toolchain: {}", self.to_string());

        let toolchain_path = self.get_path()?;

        let mut installed_version: Option<Version> = None;
        if let Ok(meta) = std::fs::symlink_metadata(&toolchain_path) {
            match meta.file_type().is_symlink() {
                true => {
                    let src = std::fs::read_link(&toolchain_path)?;
                    let src_name = src.file_name().unwrap().to_str().unwrap();
                    installed_version = Some(Version::parse(src_name).unwrap());
                    eprintln!(
                        "Toolchain {0} has been installed with SDK version {1}",
                        self, src_name
                    );
                }
                false => bail!("{:?} should be a symlink to a SDK version", toolchain_path),
            }
        }

        let resolved_version: Version = match self {
            Toolchain::CompleteVersionToolchain(v) => is_version_available(v)?,
            Toolchain::MajorMinorToolchain(major, minor) => get_compatible_version(major, minor)?,
            Toolchain::TagToolchain(t) => get_tag_version(t)?,
        };
        eprintln!("The latest compatible SDK version is {}", resolved_version);

        let status = match installed_version {
            None => "installed",
            Some(v) if v != resolved_version => "updated",
            _ => "unchanged",
        };

        if status != "unchanged" {
            match cache::is_version_installed(&resolved_version.to_string())? {
                true => eprintln!("SDK version {} already installed", resolved_version),
                false => dist::install_version(&resolved_version)?,
            };

            let cache_path = cache::get_bin_cache(&resolved_version.to_string())?;
            if toolchain_path.exists() {
                std::fs::remove_file(&toolchain_path)?;
            }
            std::os::unix::fs::symlink(cache_path, toolchain_path)?;
        }

        eprintln!(
            "Toolchain {0} {1} - SDK version {2}",
            self, status, resolved_version
        );

        Ok(())
    }

    pub fn uninstall(&self) -> DfxResult<()> {
        eprintln!("Uninstalling toolchain: {}", self);
        let toolchain_path = self.get_path()?;
        if toolchain_path.exists() {
            std::fs::remove_file(toolchain_path)?;
            eprintln!("Toolchain {} uninstalled", self);
        } else {
            eprintln!("Toolchain {} has not been installed", self);
        }
        Ok(())
    }

    pub fn get_path(&self) -> DfxResult<PathBuf> {
        let home = std::env::var("HOME")?;
        let home = Path::new(&home);
        let toolchains_dir = home.join(TOOLCHAINS_ROOT);
        std::fs::create_dir_all(&toolchains_dir)?;
        Ok(toolchains_dir.join(self.to_string()))
    }

    pub fn set_default(&self) -> DfxResult<()> {
        self.update()?;
        let default_path = get_default_path()?;
        let toolchain_path = self.get_path()?;
        if default_path.exists() {
            std::fs::remove_file(&default_path)?;
        }
        std::os::unix::fs::symlink(toolchain_path, default_path)?;
        println!("Default toolchain set to {}", self);
        Ok(())
    }
}

pub fn list_installed_toolchains() -> DfxResult<Vec<Toolchain>> {
    let home = std::env::var("HOME")?;
    let home = Path::new(&home);
    let toolchains_dir = home.join(TOOLCHAINS_ROOT);
    let mut toolchains = vec![];
    for entry in std::fs::read_dir(toolchains_dir)? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            toolchains.push(name.parse::<Toolchain>()?);
        }
    }
    Ok(toolchains)
}

pub fn get_default_toolchain() -> DfxResult<Toolchain> {
    let default_path = get_default_path()?;
    if !default_path.exists() {
        bail!("Default toolchain not set");
    }
    let toolchain_path = std::fs::read_link(&default_path)?;
    let toolchain_name = toolchain_path.file_name().unwrap().to_str().unwrap();
    toolchain_name.parse::<Toolchain>()
}

fn get_default_path() -> DfxResult<PathBuf> {
    let home = std::env::var("HOME")?;
    let home = Path::new(&home);
    let default_path = home.join(DEFAULT_PATH);
    let parent = default_path.parent().unwrap();
    std::fs::create_dir_all(parent)?;
    Ok(default_path)
}

fn is_version_available(v: &Version) -> DfxResult<Version> {
    let manifest = dist::get_manifest()?;
    let versions = manifest.get_versions();
    match versions.contains(v) {
        true => Ok(v.clone()),
        false => bail!("SDK Version {} is not available from the server", v),
    }
}

fn get_compatible_version(major: &u8, minor: &u8) -> DfxResult<Version> {
    let manifest = dist::get_manifest()?;
    let versions = manifest.get_versions();
    let req = VersionReq::parse(&format!("{}.{}", major, minor)).unwrap();
    let compatible_version = versions.iter().filter(|v| req.matches(v)).max();
    match compatible_version {
        Some(v) => Ok(v.clone()),
        None => bail!(
            "Failed to get compatible SDK version for {}.{}",
            major,
            minor
        ),
    }
}

fn get_tag_version(tag: &str) -> DfxResult<Version> {
    let manifest = dist::get_manifest()?;
    let tag_version = manifest.get_tag_version(tag);
    match tag_version {
        Some(v) => Ok(v.clone()),
        None => bail!("Failed to get compatible SDK version for tag {}", tag),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_toolchain_name() {
        assert_eq!(
            Toolchain::from_str("0.6.21").unwrap(),
            Toolchain::CompleteVersionToolchain(Version::new(0, 6, 21))
        );
        assert_eq!(
            Toolchain::from_str("0.6").unwrap(),
            Toolchain::MajorMinorToolchain(0, 6)
        );
        assert_eq!(
            Toolchain::from_str("latest").unwrap(),
            Toolchain::TagToolchain("latest".to_string())
        );
    }

    #[test]
    fn test_bad_toolchain_name() {
        assert!(Toolchain::from_str("0.06.21").is_err());
        assert!(Toolchain::from_str("0.06").is_err());
        assert!(Toolchain::from_str("unknown").is_err());
    }
}
