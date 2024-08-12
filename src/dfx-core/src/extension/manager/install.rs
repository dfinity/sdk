use crate::error::extension::{
    DownloadAndInstallExtensionToTempdirError, FinalizeInstallationError,
    GetExtensionArchiveNameError, GetExtensionDownloadUrlError, GetExtensionManifestError,
    GetHighestCompatibleVersionError, GetTopLevelDirectoryError, InstallExtensionError,
};
use crate::extension::{
    catalog::ExtensionCatalog,
    manager::ExtensionManager,
    manifest::{ExtensionDependencies, ExtensionManifest},
    url::ExtensionJsonUrl,
};
use crate::http::get::get_with_retries;
use backoff::exponential::ExponentialBackoff;
use flate2::read::GzDecoder;
use reqwest::Url;
use semver::{BuildMetadata, Prerelease, Version};
use std::io::Cursor;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tar::Archive;
use tempfile::{tempdir_in, TempDir};

pub enum InstallOutcome {
    Installed(String, Version),
    AlreadyInstalled(String, Version),
}

impl ExtensionManager {
    pub async fn install_extension(
        &self,
        name: &str,
        catalog_url: Option<&Url>,
        install_as: Option<&str>,
        version: Option<&Version>,
    ) -> Result<InstallOutcome, InstallExtensionError> {
        let url = if let Ok(url) = Url::parse(name) {
            ExtensionJsonUrl::new(url)
        } else {
            ExtensionCatalog::fetch(catalog_url)
                .await?
                .lookup(name)
                .ok_or(InstallExtensionError::ExtensionNotFound(name.to_string()))?
        };

        let manifest = Self::get_extension_manifest(&url).await?;
        let extension_name: &str = &manifest.name;

        let effective_extension_name = install_as.unwrap_or(extension_name);

        if self
            .get_extension_directory(effective_extension_name)
            .exists()
        {
            let installed_manifest = ExtensionManifest::load(effective_extension_name, &self.dir)?;

            return if matches!(version, Some(v) if *v != *installed_manifest.version) {
                Err(InstallExtensionError::OtherVersionAlreadyInstalled(
                    extension_name.to_string(),
                    installed_manifest.version.clone(),
                ))
            } else {
                Ok(InstallOutcome::AlreadyInstalled(
                    extension_name.to_string(),
                    installed_manifest.version.clone(),
                ))
            };
        }

        let extension_version = match version {
            Some(version) => version.clone(),
            None => self.get_highest_compatible_version(&url).await?,
        };
        let github_release_tag = get_git_release_tag(extension_name, &extension_version);
        let extension_archive = get_extension_archive_name(extension_name)?;
        let archive_url = get_extension_download_url(
            &manifest.download_url_template(),
            &github_release_tag,
            &extension_archive,
        )?;

        let temp_dir = self
            .download_and_unpack_extension_to_tempdir(archive_url)
            .await?;

        self.finalize_installation(extension_name, effective_extension_name, temp_dir)?;

        Ok(InstallOutcome::Installed(
            extension_name.to_string(),
            extension_version,
        ))
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

    async fn get_highest_compatible_version(
        &self,
        url: &ExtensionJsonUrl,
    ) -> Result<Version, GetHighestCompatibleVersionError> {
        let dependencies = ExtensionDependencies::fetch(url).await?;
        let dfx_version = self.dfx_version_strip_semver();
        dependencies
            .find_highest_compatible_version(&dfx_version)?
            .ok_or(GetHighestCompatibleVersionError::NoCompatibleVersionFound())
    }

    async fn get_extension_manifest(
        url: &ExtensionJsonUrl,
    ) -> Result<ExtensionManifest, GetExtensionManifestError> {
        let retry_policy = ExponentialBackoff {
            max_elapsed_time: Some(Duration::from_secs(60)),
            ..Default::default()
        };
        let resp = get_with_retries(url.as_url().clone(), retry_policy)
            .await
            .map_err(GetExtensionManifestError::Get)?;

        resp.json()
            .await
            .map_err(GetExtensionManifestError::ParseJson)
    }

    async fn download_and_unpack_extension_to_tempdir(
        &self,
        download_url: Url,
    ) -> Result<TempDir, DownloadAndInstallExtensionToTempdirError> {
        let retry_policy = ExponentialBackoff {
            max_elapsed_time: Some(Duration::from_secs(60)),
            ..Default::default()
        };
        let response = get_with_retries(download_url.clone(), retry_policy)
            .await
            .map_err(DownloadAndInstallExtensionToTempdirError::ExtensionDownloadFailed)?;

        let bytes = response
            .bytes()
            .await
            .map_err(DownloadAndInstallExtensionToTempdirError::ExtensionDownloadFailed)?;

        crate::fs::composite::ensure_dir_exists(&self.dir)
            .map_err(DownloadAndInstallExtensionToTempdirError::EnsureExtensionDirExistsFailed)?;

        let temp_dir = tempdir_in(&self.dir).map_err(|e| {
            DownloadAndInstallExtensionToTempdirError::CreateTemporaryDirectoryFailed(
                self.dir.to_path_buf(),
                e,
            )
        })?;

        let mut archive = Archive::new(GzDecoder::new(Cursor::new(bytes)));
        archive.unpack(temp_dir.path()).map_err(|e| {
            DownloadAndInstallExtensionToTempdirError::DecompressFailed(download_url, e)
        })?;

        Ok(temp_dir)
    }

    fn finalize_installation(
        &self,
        extension_name: &str,
        effective_extension_name: &str,
        temp_dir: TempDir,
    ) -> Result<(), FinalizeInstallationError> {
        let effective_extension_dir = &self.get_extension_directory(effective_extension_name);
        let top_level_dir = get_top_level_directory(temp_dir.path())?;
        crate::fs::rename(&top_level_dir, effective_extension_dir)?;

        let installed_manifest = ExtensionManifest::load(effective_extension_name, &self.dir)?;

        if matches!(installed_manifest.subcommands, Some(subcommands) if !subcommands.0.is_empty())
        {
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
        }
        Ok(())
    }
}

fn get_top_level_directory(dir: &Path) -> Result<PathBuf, GetTopLevelDirectoryError> {
    // the archive will have a single top-level subdirectory
    // return that subdirectory
    Ok(crate::fs::read_dir(dir)?
        .next()
        .ok_or(GetTopLevelDirectoryError::NoTopLevelDirectoryEntry)?
        .map_err(GetTopLevelDirectoryError::ReadDirEntry)?
        .path())
}

fn get_extension_download_url(
    download_url_template: &str,
    github_release_tag: &str,
    extension_archive_name: &str,
) -> Result<Url, GetExtensionDownloadUrlError> {
    let download_url = download_url_template
        .replace("{{tag}}", github_release_tag)
        .replace("{{basename}}", extension_archive_name)
        .replace("{{archive-format}}", "tar.gz");

    Url::parse(&download_url).map_err(|source| GetExtensionDownloadUrlError {
        url: download_url,
        source,
    })
}

fn get_git_release_tag(extension_name: &str, extension_verion: &Version) -> String {
    format!("{extension_name}-v{extension_verion}",)
}

fn get_extension_archive_name(
    extension_name: &str,
) -> Result<String, GetExtensionArchiveNameError> {
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
