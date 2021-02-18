use crate::lib::dist;
use crate::lib::error::{DfxError, DfxResult};

use anyhow::bail;
use semver::{Version, VersionReq};
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

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
            && s.split(".").count() == 2
        {
            let major = s.split(".").nth(0).unwrap().parse::<u8>()?;
            let minor = s.split(".").nth(1).unwrap().parse::<u8>()?;
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
    pub fn install(&self) -> DfxResult<()> {
        eprintln!("Installing toolchain: {}", self);
        let resolved_version: Version = match self {
            Toolchain::CompleteVersionToolchain(v) => v.clone(),
            Toolchain::MajorMinorToolchain(major, minor) => get_compatible_version(major, minor)?,
            Toolchain::TagToolchain(t) => get_tag_version(t)?,
        };
        eprintln!("Compatible SDK version found: {}", resolved_version);
        dist::install_version(&resolved_version)?;
        Ok(())
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
