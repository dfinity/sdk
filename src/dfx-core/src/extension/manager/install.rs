use crate::error::extension::{DownloadAndInstallExtensionToTempdirError, FinalizeInstallationError, FindLatestExtensionCompatibleVersionError, GetExtensionArchiveNameError, GetExtensionDownloadUrlError, InstallExtensionError};
use crate::extension::{manager::ExtensionManager, manifest::ExtensionCompatibilityMatrix};
use flate2::read::GzDecoder;
use reqwest::Url;
use semver::{BuildMetadata, Prerelease, Version};
use std::io::Cursor;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use tar::Archive;
use tempfile::{tempdir_in, TempDir};

const DFINITY_DFX_EXTENSIONS_RELEASES_URL: &str =
    "https://github.com/dfinity/dfx-extensions/releases/download";

impl ExtensionManager {
    pub fn install_extension(
        &self,
        extension_name: &str,
        install_as: Option<&str>,
        version: Option<&Version>,
    ) -> Result<(), InstallExtensionError> {
        let effective_extension_name = install_as.unwrap_or(extension_name);

        if self
            .get_extension_directory(effective_extension_name)
            .exists()
        {
            return Err(InstallExtensionError::ExtensionAlreadyInstalled(
                effective_extension_name.to_string(),
            ));
        }

        let extension_version = match version {
            Some(version) => version.clone(),
            None => self.get_extension_compatible_version(extension_name)?,
        };
        let github_release_tag = get_git_release_tag(extension_name, &extension_version);
        let extension_archive = get_extension_archive_name(extension_name)?;
        let url = get_extension_download_url(&github_release_tag, &extension_archive)?;

        let temp_dir = self.download_and_unpack_extension_to_tempdir(url)?;

        self.finalize_installation(
            extension_name,
            effective_extension_name,
            &extension_archive,
            temp_dir,
        )?;

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

    fn get_extension_compatible_version(
        &self,
        extension_name: &str,
    ) -> Result<Version, FindLatestExtensionCompatibleVersionError> {
        let manifest = ExtensionCompatibilityMatrix::fetch()?;
        let dfx_version = self.dfx_version_strip_semver();
        manifest.find_latest_compatible_extension_version(extension_name, &dfx_version)
    }

    fn download_and_unpack_extension_to_tempdir(
        &self,
        download_url: Url,
    ) -> Result<TempDir, DownloadAndInstallExtensionToTempdirError> {
        let response = reqwest::blocking::get(download_url.clone())
            .map_err(|e| DownloadAndInstallExtensionToTempdirError::ExtensionDownloadFailed(download_url.clone(), e))?;

        let bytes = response
            .bytes()
            .map_err(|e| DownloadAndInstallExtensionToTempdirError::ExtensionDownloadFailed(download_url.clone(), e))?;

        crate::fs::composite::ensure_dir_exists(&self.dir)
            .map_err(DownloadAndInstallExtensionToTempdirError::EnsureExtensionDirExistsFailed)?;

        let temp_dir = tempdir_in(&self.dir).map_err(|e| {
            DownloadAndInstallExtensionToTempdirError::CreateTemporaryDirectoryFailed(self.dir.to_path_buf(), e)
        })?;

        let mut archive = Archive::new(GzDecoder::new(Cursor::new(bytes)));
        archive
            .unpack(temp_dir.path())
            .map_err(|e| DownloadAndInstallExtensionToTempdirError::DecompressFailed(download_url, e))?;

        Ok(temp_dir)
    }

    fn finalize_installation(
        &self,
        extension_name: &str,
        effective_extension_name: &str,
        extension_unarchived_dir_name: &str,
        temp_dir: TempDir,
    ) -> Result<(), FinalizeInstallationError> {
        let effective_extension_dir = &self.get_extension_directory(effective_extension_name);
        crate::fs::rename(
            &temp_dir.path().join(extension_unarchived_dir_name),
            effective_extension_dir,
        )?;
        if extension_name != effective_extension_name {
            // rename the binary
            crate::fs::rename(
                &effective_extension_dir.join(extension_name),
                &effective_extension_dir.join(effective_extension_name),
            )?;
        }
        #[cfg(unix)]
        {
            let bin = effective_extension_dir.join(effective_extension_name);
            crate::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o500))?;
        }
        Ok(())
    }
}

fn get_extension_download_url(
    github_release_tag: &str,
    extension_archive_name: &str,
) -> Result<Url, GetExtensionDownloadUrlError> {
    let download_url = format!("{DFINITY_DFX_EXTENSIONS_RELEASES_URL}/{github_release_tag}/{extension_archive_name}.tar.gz",);
    Url::parse(&download_url)
        .map_err(|source| GetExtensionDownloadUrlError { url: download_url, source })
}

fn get_git_release_tag(extension_name: &str, extension_verion: &Version) -> String {
    format!("{extension_name}-v{extension_verion}",)
}

fn get_extension_archive_name(extension_name: &str) -> Result<String, GetExtensionArchiveNameError> {
    Ok(format!(
        "{extension_name}-{arch}-{platform}",
        platform = match std::env::consts::OS {
            "linux" => "unknown-linux-gnu",
            "macos" => "apple-darwin",
            // "windows" => "pc-windows-msvc",
            unsupported_platform =>
                return Err(GetExtensionArchiveNameError::PlatformNotSupported(
                    unsupported_platform.to_string()
                )),
        },
        arch = std::env::consts::ARCH,
    ))
}
