use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;

use crate::lib::error::DfxResult;
use crate::lib::extension::manager::DfxError;
use crate::lib::extension::manager::ExtensionError;
use crate::lib::extension::manifest::compatibility_matrix::ExtensionsCompatibilityMatrix;
use flate2::read::GzDecoder;
use reqwest::Url;
use semver::BuildMetadata;
use semver::Prerelease;
use tar::Archive;
use tempfile::{tempdir_in, TempDir};

use super::ExtensionsManager;

// possible errors:
// - extension already installed
// - dfx version not found in compatibility.json
// - extension not found
// - extension download failed
// - extension version not found
// - extension version already installed
// - extension version not compatible with
//   - dfx version
//   - platform
//   - architecture
// - maformed manifest
//   - malformed extension version
//   - malformed extension download url

impl ExtensionsManager {
    pub fn install_extension(&self, extension_name: &str) -> DfxResult<()> {
        if self.dir.join(extension_name).exists() {
            return Err(DfxError::new(ExtensionError::ExtensionError(
                format!("extension {} already installed", extension_name), // TODO: --force
            )));
        }

        let mut dfx_version = self.dfx_version.clone();
        // Removing the prerelease tag and build metadata, because they should
        // not be allowed in extension manifests, and semver crate won't match
        // a semver with a prerelease tag or build metadata against a semver without.
        dfx_version.pre = Prerelease::EMPTY;
        dfx_version.build = BuildMetadata::EMPTY;

        let manifest = ExtensionsCompatibilityMatrix::fetch()?;
        let extension_version =
            manifest.find_latest_compatible_extension_version(extension_name, dfx_version)?;
        let download_url = format!(
            // "https://github.com/dfinity/dfx-extensions/releases/download/{tag}/{name}-{version}-{platform}-{arch}.tar.gz",
            "https://github.com/smallstepman/dfx-extensions/releases/download/{name}-{version}/{name}-{version}-{platform}-{arch}.tar.gz",
            name = extension_name,
            version = extension_version,
            platform = std::env::consts::OS,
            arch = std::env::consts::ARCH,
        );
        let url = Url::parse(&download_url)?;

        let temp_dir = self.download_and_unpack_extension_to_tempdir(url)?;
        std::fs::File::open(temp_dir.path().join(extension_name))?
            .set_permissions(std::fs::Permissions::from_mode(0o777))?;

        let extension_dir = self.dir.join(extension_name);
        std::fs::rename(temp_dir, extension_dir)?;

        Ok(())
    }

    fn download_and_unpack_extension_to_tempdir(&self, download_url: Url) -> DfxResult<TempDir> {
        let response = reqwest::blocking::get(download_url)
            .map_err(|e| anyhow::anyhow!("Failed to download extension: {}", e))?;

        let mut archive = Archive::new(GzDecoder::new(Cursor::new(response.bytes()?)));

        let temp_dir = tempdir_in(&self.dir)?;
        archive.unpack(temp_dir.path())?;

        Ok(temp_dir)
    }
}
