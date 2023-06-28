use crate::lib::dist;
use crate::lib::error::{DfxError, DfxResult};
use dfx_core::config::cache;

use anyhow::{bail, Context};
use directories_next::BaseDirs;
use fn_error_context::context;
use semver::{Version, VersionReq};
use std::fmt;
use std::fmt::Formatter;
use std::path::PathBuf;
use std::str::FromStr;

const TOOLCHAINS_ROOT: &str = ".dfinity/toolchains/";
const DEFAULT_PATH: &str = ".dfinity/default";

#[derive(Debug, Eq, PartialEq)]
pub enum Toolchain {
    /// Complete semver such as '0.6.21'
    CompleteVersion(Version),

    /// Partial semver such as '0.6', '1.0'
    MajorMinor(u8, u8),

    /// Tag such as 'latest'
    Tag(String),
}

impl FromStr for Toolchain {
    type Err = DfxError;
    fn from_str(s: &str) -> DfxResult<Self> {
        if let Ok(v) = Version::parse(s) {
            Ok(Toolchain::CompleteVersion(v))
        } else if VersionReq::parse(s).is_ok()
            && s.chars().all(|c| c.is_ascii_digit() || c == '.')
            && s.split('.').count() == 2
        {
            let major = s.split('.').next().unwrap().parse::<u8>()?;
            let minor = s.split('.').nth(1).unwrap().parse::<u8>()?;
            Ok(Toolchain::MajorMinor(major, minor))
        } else {
            match s {
                "latest" => Ok(Toolchain::Tag("latest".to_string())),
                _ => bail!("Invalid toolchain name: {}", s),
            }
        }
    }
}

impl fmt::Display for Toolchain {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::CompleteVersion(v) => write!(f, "{}", v),
            Self::MajorMinor(major, minor) => write!(f, "{0}.{1}", major, minor),
            Self::Tag(t) => write!(f, "{}", t),
        }
    }
}

impl Toolchain {
    // Update the toolchain, install it if nonexisting
    #[context("Failed to update toolchain.")]
    pub fn update(&self) -> DfxResult<()> {
        eprintln!("Syncing toolchain: {}", self);

        let toolchain_path = self.get_path()?;

        let mut installed_version: Option<Version> = None;
        if let Ok(meta) = std::fs::symlink_metadata(&toolchain_path) {
            match meta.file_type().is_symlink() {
                true => {
                    let src = std::fs::read_link(&toolchain_path).with_context(|| {
                        format!(
                            "Failed to read symlink {}.",
                            toolchain_path.to_string_lossy()
                        )
                    })?;
                    let src_name = src.file_name().unwrap().to_str().unwrap();
                    installed_version = Some(Version::parse(src_name).unwrap());
                    eprintln!(
                        "Toolchain {0} has been installed with SDK version {1}",
                        self, src_name
                    );
                }
                false => bail!(
                    "{} should be a symlink to a SDK version",
                    toolchain_path.to_string_lossy()
                ),
            }
        }

        let resolved_version: Version = match self {
            Toolchain::CompleteVersion(v) => is_version_available(v)?,
            Toolchain::MajorMinor(major, minor) => get_compatible_version(major, minor)?,
            Toolchain::Tag(t) => get_tag_version(t)?,
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
                std::fs::remove_file(&toolchain_path).with_context(|| {
                    format!("Failed to remove {}.", toolchain_path.to_string_lossy())
                })?;
            }
            #[cfg(unix)]
            std::os::unix::fs::symlink(&cache_path, &toolchain_path).with_context(|| {
                format!(
                    "Failed to create symlink from {} to {}.",
                    toolchain_path.display(),
                    cache_path.display(),
                )
            })?;
            // On Windows, symlinks require admin permission or developer mode,
            // and junctions are preferable anyway as the filesystem parses them instead of the OS.
            #[cfg(windows)]
            junction::create(&cache_path, &toolchain_path).with_context(|| {
                format!(
                    "Failed to create junction from {} to {}.",
                    toolchain_path.display(),
                    cache_path.display(),
                )
            })?;
        }

        eprintln!(
            "Toolchain {0} {1} - SDK version {2}",
            self, status, resolved_version
        );

        Ok(())
    }

    #[context("Failed to uninistall toolchain {}.", self)]
    pub fn uninstall(&self) -> DfxResult<()> {
        eprintln!("Uninstalling toolchain: {}", self);
        let toolchain_path = self.get_path()?;
        if toolchain_path.exists() {
            std::fs::remove_file(&toolchain_path).with_context(|| {
                format!("Failed to remove {}.", toolchain_path.to_string_lossy())
            })?;
            eprintln!("Toolchain {} uninstalled", self);
        } else {
            eprintln!("Toolchain {} has not been installed", self);
        }
        Ok(())
    }

    #[context("Failed to get toolchain path.")]
    pub fn get_path(&self) -> DfxResult<PathBuf> {
        let dirs = BaseDirs::new().context("Failed to resolve home dir.")?;
        let toolchains_dir = dirs.home_dir().join(TOOLCHAINS_ROOT);
        std::fs::create_dir_all(&toolchains_dir).with_context(|| {
            format!(
                "Failed to create toolchain dir {}.",
                toolchains_dir.to_string_lossy()
            )
        })?;
        Ok(toolchains_dir.join(self.to_string()))
    }

    #[context("Failed to set '{}' as default toolchain.", self)]
    pub fn set_default(&self) -> DfxResult<()> {
        self.update()?;
        let default_path = get_default_path()?;
        let toolchain_path = self.get_path()?;
        if default_path.exists() {
            std::fs::remove_file(&default_path).with_context(|| {
                format!(
                    "Failed to remove default toolchain path {}.",
                    default_path.to_string_lossy()
                )
            })?;
        }
        #[cfg(unix)]
        std::os::unix::fs::symlink(&toolchain_path, &default_path).with_context(|| {
            format!(
                "Failed to create symlink from {} to {}.",
                toolchain_path.display(),
                default_path.display(),
            )
        })?;
        // On Windows, symlinks require admin permission or developer mode,
        // and junctions are preferable anyway as the filesystem parses them instead of the OS.
        #[cfg(windows)]
        junction::create(&toolchain_path, &default_path).with_context(|| {
            format!(
                "Failed to create junction from {} to {}.",
                toolchain_path.display(),
                default_path.display(),
            )
        })?;
        println!("Default toolchain set to {}", self);
        Ok(())
    }
}

#[context("Failed to get installed toolchains.")]
pub fn list_installed_toolchains() -> DfxResult<Vec<Toolchain>> {
    let dirs = BaseDirs::new().context("Failed to resolve home dir.")?;
    let toolchains_dir = dirs.home_dir().join(TOOLCHAINS_ROOT);
    let mut toolchains = vec![];
    for entry in std::fs::read_dir(&toolchains_dir).with_context(|| {
        format!(
            "Failed to read toolchain dir {}.",
            toolchains_dir.to_string_lossy()
        )
    })? {
        let entry = entry.with_context(|| {
            format!(
                "Failed to read a directory entry in {}.",
                toolchains_dir.to_string_lossy()
            )
        })?;
        if let Some(name) = entry.file_name().to_str() {
            toolchains.push(
                name.parse::<Toolchain>()
                    .with_context(|| format!("Failed to add {} to toolchains.", name))?,
            );
        }
    }
    Ok(toolchains)
}

#[context("Failed to get default toolchain.")]
pub fn get_default_toolchain() -> DfxResult<Toolchain> {
    let default_path = get_default_path()?;
    if !default_path.exists() {
        bail!("Default toolchain not set");
    }
    let toolchain_path = std::fs::read_link(&default_path).with_context(|| {
        format!(
            "Failed to read default toolchain symlink at {}.",
            default_path.to_string_lossy()
        )
    })?;
    let toolchain_name = toolchain_path.file_name().unwrap().to_str().unwrap();
    toolchain_name.parse::<Toolchain>()
}

#[context("Failed to get default toolchain path.")]
fn get_default_path() -> DfxResult<PathBuf> {
    let dirs = BaseDirs::new().context("Failed to resolve home dir.")?;
    let default_path = dirs.home_dir().join(DEFAULT_PATH);
    let parent = default_path.parent().unwrap();
    std::fs::create_dir_all(parent)
        .with_context(|| format!("Failed to create dir {}.", parent.to_string_lossy()))?;
    Ok(default_path)
}

#[context("Failed to determine if version {} is available.", v)]
fn is_version_available(v: &Version) -> DfxResult<Version> {
    let manifest = dist::get_manifest()?;
    let versions = manifest.get_versions();
    match versions.contains(v) {
        true => Ok(v.clone()),
        false => bail!("SDK Version {} is not available from the server", v),
    }
}

#[context(
    "Failed to get compatible version for major {} and minor {}.",
    major,
    minor
)]
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

#[context("Failed to get tag version '{}'.", tag)]
fn get_tag_version(tag: &str) -> DfxResult<Version> {
    let manifest = dist::get_manifest()?;
    let tag_version = manifest.get_tag_version(tag);
    match tag_version {
        Some(v) => Ok(v.clone()),
        None => bail!("Failed to get compatible SDK version for tag {}.", tag),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_toolchain_name() {
        assert_eq!(
            Toolchain::from_str("0.6.21").unwrap(),
            Toolchain::CompleteVersion(Version::new(0, 6, 21))
        );
        assert_eq!(
            Toolchain::from_str("0.6").unwrap(),
            Toolchain::MajorMinor(0, 6)
        );
        assert_eq!(
            Toolchain::from_str("latest").unwrap(),
            Toolchain::Tag("latest".to_string())
        );
    }

    #[test]
    fn test_bad_toolchain_name() {
        assert!(Toolchain::from_str("0.06.21").is_err());
        assert!(Toolchain::from_str("0.06").is_err());
        assert!(Toolchain::from_str("unknown").is_err());
    }
}
