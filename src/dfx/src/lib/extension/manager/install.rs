use crate::lib::error::extension::ExtensionError;
use crate::lib::extension::{manager::ExtensionManager, manifest::ExtensionCompatibilityMatrix};

use flate2::read::GzDecoder;
use reqwest::Url;
use semver::{BuildMetadata, Prerelease, Version};
use tar::Archive;
use tempfile::{tempdir_in, TempDir};

use std::io::Cursor;
#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;

impl ExtensionManager {
    pub fn install_extension(&self, extension_name: &str) -> Result<(), ExtensionError> {
        if self.get_extension_directory(extension_name).exists() {
            return Err(ExtensionError::ExtensionAlreadyInstalled(
                format!("extension {} already installed", extension_name), // TODO: --force
            ));
        }

        let url = self.get_extension_download_url(extension_name)?;

        let temp_dir = self.download_and_unpack_extension_to_tempdir(url)?;

        self.finalize_installation(extension_name, temp_dir)?;

        Ok(())
    }

    /// Removing the prerelease tag and build metadata, because they should
    /// not be allowed in extension manifests, and semver crate won't match
    /// a semver with a prerelease tag or build metadata against a semver without.
    fn dfx_version_strip_semver(&self) -> Version {
        let mut dfx_version = self.dfx_version.clone();
        dfx_version.pre = Prerelease::EMPTY;
        dfx_version.build = BuildMetadata::EMPTY;
        dfx_version
    }

    fn get_extension_download_url(&self, extension_name: &str) -> Result<Url, ExtensionError> {
        let manifest = ExtensionCompatibilityMatrix::fetch()?;
        let dfx_version = self.dfx_version_strip_semver();
        let extension_version =
            manifest.find_latest_compatible_extension_version(extension_name, dfx_version)?;
        let download_url = format!(
            "https://github.com/dfinity/dfx-extensions/releases/download/{name}-{version}/{name}-{version}-{platform}-{arch}.tar.gz",
            name = extension_name,
            version = extension_version,
            platform = std::env::consts::OS,
            arch = std::env::consts::ARCH,
        );
        Url::parse(&download_url).map_err(|e| ExtensionError::MalformedExtensionDownloadUrl(e))
    }

    fn download_and_unpack_extension_to_tempdir(
        &self,
        download_url: Url,
    ) -> Result<TempDir, ExtensionError> {
        let response = reqwest::blocking::get(download_url.clone())
            .map_err(|_e| ExtensionError::ExtensionDownloadFailed(download_url.clone()))?;

        let bytes = response
            .bytes()
            .map_err(|_e| ExtensionError::ExtensionDownloadFailed(download_url.clone()))?;

        let temp_dir = tempdir_in(&self.dir).map_err(|_e|
            ExtensionError::CreateTemporaryDirectoryFailed(
                self.dir.to_path_buf()
            )
        )?;

        let mut archive = Archive::new(GzDecoder::new(Cursor::new(bytes)));

        if let Err(e) = archive.unpack(temp_dir.path()) {
            return Err(ExtensionError::DecompressFailed(download_url, e));
        }

        Ok(temp_dir)
    }

    fn finalize_installation(
        &self,
        extension_name: &str,
        temp_dir: TempDir,
    ) -> Result<(), ExtensionError> {
        #[cfg(not(target_os = "windows"))]
        {
            let bin = temp_dir.path().join(extension_name);
            let f = std::fs::File::open(&bin).map_err(|_e|
                ExtensionError::InsufficientPermissionsToOpenExtensionBinaryInWriteMode(
                    extension_name.to_string()
                )
            )?;

            if let Err(e) = f.set_permissions(std::fs::Permissions::from_mode(0o777)) {
                return Err(ExtensionError::ChangeFilePermissionsFailed(bin, e));
            }
        }

        let extension_dir = self.dir.join(extension_name);
        if let Err(e) = std::fs::rename(temp_dir, extension_dir) {
            return Err(ExtensionError::RenameDirectoryFailed(e));
        }

        Ok(())
    }
}
