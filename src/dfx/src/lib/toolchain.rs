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
    TagToolchain(Tag),
}

#[derive(Debug, Eq, PartialEq)]
pub enum Tag {
    Latest,
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
                "latest" => Ok(Toolchain::TagToolchain(Tag::Latest)),
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
            Self::TagToolchain(t) => write!(f, "{:?}", t),
        }
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
            Toolchain::TagToolchain(Tag::Latest)
        );
    }

    #[test]
    fn test_bad_toolchain_name() {
        assert!(Toolchain::from_str("0.06.21").is_err());
        assert!(Toolchain::from_str("0.06").is_err());
        assert!(Toolchain::from_str("unknown").is_err());
    }
}
