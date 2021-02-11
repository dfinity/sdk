use crate::lib::error::{DfxError, DfxResult};

use anyhow::bail;
use semver::Version;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq)]
pub enum ToolchainDesc {
    /// Complete semver such as '0.6.21'
    CompleteVersionToolchainDesc(Version),

    /// Partial semver such as '0.6', '1.0'
    MajorMinorToolchainDesc(u8, u8),

    /// Tag such as 'latest'
    TagToolchainDesc(Tag),
}

#[derive(Debug, Eq, PartialEq)]
pub enum Tag {
    Latest,
}

impl FromStr for ToolchainDesc {
    type Err = DfxError;
    fn from_str(s: &str) -> DfxResult<Self> {
        if let Ok(v) = Version::parse(s) {
            Ok(ToolchainDesc::CompleteVersionToolchainDesc(v))
        } else if s.chars().all(|c| c.is_ascii_digit() || c == '.') && s.split(".").count() == 2 {
            let major = s.split(".").nth(0).unwrap().parse::<u8>()?;
            let minor = s.split(".").nth(1).unwrap().parse::<u8>()?;
            Ok(ToolchainDesc::MajorMinorToolchainDesc(major, minor))
        } else {
            match s {
                "latest" => Ok(ToolchainDesc::TagToolchainDesc(Tag::Latest)),
                _ => bail!("Invalid toolchain name: {}", s),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_desc() {
        assert_eq!(
            ToolchainDesc::from_str("0.6.21").unwrap(),
            ToolchainDesc::CompleteVersionToolchainDesc(Version::new(0, 6, 21))
        );

        assert_eq!(
            ToolchainDesc::from_str("0.6").unwrap(),
            ToolchainDesc::MajorMinorToolchainDesc(0, 6)
        );
    }
}
