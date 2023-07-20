use crate::error::archive::ArchiveError;
use crate::error::extension::ExtensionError;
use crate::extension::manifest::extension::ExtensionBinariesDescriptor;
use crate::extension::manifest::{ExtensionManifest, ExternalExtensionManifest};
use crate::extension::{manager::ExtensionManager, manifest::ExtensionCompatibilityMatrix};

use bytes::Bytes;
use flate2::read::GzDecoder;
use reqwest::Url;
use semver::{BuildMetadata, Prerelease, Version};
use sha2::{Digest, Sha256};
use tar::Archive;
use tempfile::{tempdir_in, TempDir};

use std::io::Cursor;
#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

const DFINITY_DFX_EXTENSIONS_RELEASES_URL: &str =
    "https://github.com/dfinity/dfx-extensions/releases/download";

impl ExtensionManager {
    pub fn install_extension(
        &self,
        extension_name: &str,
        external_registry_manifest_url: Option<&str>,
        install_as: Option<&str>,
    ) -> Result<(), ExtensionError> {
        let effective_extension_name = install_as.unwrap_or(extension_name);

        if self
            .get_extension_directory(effective_extension_name)
            .exists()
        {
            return Err(ExtensionError::ExtensionAlreadyInstalled(
                effective_extension_name.to_string(),
            ));
        }

        let dfx_version = strip_semver(self.dfx_version.clone());

        let (
            extension_download_url,
            extension_archive_filename,
            checksum_source,
            extension_manifest,
        ) = match external_registry_manifest_url {
            Some(url) => process_external_registry_manifest(url, extension_name, &dfx_version)?,
            None => process_dfinity_registry_manifest(extension_name, &dfx_version)?,
        };

        self.process_extension_download(
            extension_download_url,
            &get_checksum(checksum_source)?,
            extension_name,
            effective_extension_name,
            &extension_archive_filename,
            extension_manifest,
        )
    }

    fn process_extension_download(
        &self,
        extension_download_url: Url,
        extension_archive_checksum: &str,
        extension_name: &str,
        effective_extension_name: &str,
        extension_archive_filename: &str,
        extension_manifest: Option<ExtensionManifest>,
    ) -> Result<(), ExtensionError> {
        let bytes = download(&extension_download_url)?;
        verify_checksum(&bytes, extension_archive_checksum).map_err(|hash_of_bytes| {
            ExtensionError::ChecksumMismatch(
                hash_of_bytes,
                extension_download_url.into(),
                extension_archive_checksum.to_string(),
            )
        })?;

        crate::fs::composite::ensure_dir_exists(&self.dir)
            .map_err(ExtensionError::EnsureExtensionDirExistsFailed)?;

        let temp_dir = tempdir_in(&self.dir).map_err(|e| {
            ExtensionError::CreateTemporaryDirectoryFailed(self.dir.to_path_buf(), e)
        })?;
        extract_archive(temp_dir.path(), bytes)
            .map_err(ArchiveError::ArchiveFileInvalidPath)
            .map_err(|e| ExtensionError::Io(e.into()))?;
        if let Some(extension_manifest) = extension_manifest {
            crate::json::save_json_file(
                &temp_dir
                    .path()
                    .join(extension_archive_filename)
                    .join("extension.json"),
                &extension_manifest,
            )
            .map_err(|e| ExtensionError::SaveExtensionManifestFailed(e))?;
        }
        self.finalize_installation(
            extension_name,
            effective_extension_name,
            extension_archive_filename,
            temp_dir,
        )
    }

    fn finalize_installation(
        &self,
        extension_name: &str,
        effective_extension_name: &str,
        extension_unarchived_dir_name: &str,
        temp_dir: TempDir,
    ) -> Result<(), ExtensionError> {
        let effective_extension_dir = &self.get_extension_directory(effective_extension_name);
        crate::fs::rename(
            &temp_dir.path().join(extension_unarchived_dir_name),
            effective_extension_dir,
        )
        .map_err(|e| ExtensionError::Io(e.into()))?;
        if extension_name != effective_extension_name {
            // rename the binary
            crate::fs::rename(
                &effective_extension_dir.join(extension_name),
                &effective_extension_dir.join(effective_extension_name),
            )
            .map_err(|e| ExtensionError::Io(e.into()))?;
        }
        #[cfg(not(target_os = "windows"))]
        {
            let bin = effective_extension_dir.join(effective_extension_name);
            crate::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o777))
                .map_err(|e| ExtensionError::Io(e.into()))?;
        }
        Ok(())
    }
}

fn process_external_registry_manifest(
    manifest_url: &str,
    extension_name: &str,
    dfx_version: &Version,
) -> Result<(Url, String, ChecksumSource, Option<ExtensionManifest>), ExtensionError> {
    let (arch, platform) = get_arch_and_platform()?;
    let manifest_url = Url::parse(manifest_url)
        .map_err(|e| ExtensionError::InvalidExternalManifestUrl(manifest_url.to_string(), e))?;
    let manifest = ExternalExtensionManifest::fetch(&manifest_url)?;
    let extension_manifest = manifest.find_extension(extension_name, dfx_version)?;
    let extension_version = extension_manifest.version.clone().unwrap(); // safe to unwrap, because the value is set in find_extension
    let extension_version = Version::parse(&extension_version).map_err(|e| {
        ExtensionError::MalformedVersionsEntryForExtensionInCompatibilityMatrix(
            extension_name.to_string(),
            e,
        )
    })?;
    let archive_filename = get_extension_archive_name(extension_name, &extension_version)?;
    let binary_descriptor =
        extension_manifest.get_binary_descriptor(format!("{}-{}", platform, arch))?;

    let download_url = binary_descriptor.url.clone();
    let checksum_source = ChecksumSource::ExternalRegistryManifest(binary_descriptor.clone());

    Ok((
        download_url,
        archive_filename,
        checksum_source,
        Some(extension_manifest),
    ))
}

fn process_dfinity_registry_manifest(
    extension_name: &str,
    dfx_version: &Version,
) -> Result<(Url, String, ChecksumSource, Option<ExtensionManifest>), ExtensionError> {
    let extension_version = ExtensionCompatibilityMatrix::fetch()?
        .find_latest_compatible_extension_version(extension_name, dfx_version)?;
    let github_release_tag = get_git_release_tag(extension_name, &extension_version);
    let archive_filename = get_extension_archive_name(extension_name, &extension_version)?;
    let download_url = get_extension_download_url(&github_release_tag, &archive_filename)?;

    let checksum_source = ChecksumSource::DfinityGithubReleases {
        git_tag: github_release_tag,
        archive_filename: archive_filename.clone(),
    };

    Ok((download_url, archive_filename, checksum_source, None))
}

/// Removing the prerelease tag and build metadata, because they should
/// not be allowed in extension manifests, and semver crate won't match
/// a semver with a prerelease tag or build metadata against a semver without.
fn strip_semver(mut dfx_version: Version) -> Version {
    dfx_version.pre = Prerelease::EMPTY;
    dfx_version.build = BuildMetadata::EMPTY;
    dfx_version
}

enum ChecksumSource {
    ExternalRegistryManifest(ExtensionBinariesDescriptor),
    DfinityGithubReleases {
        git_tag: String,
        archive_filename: String,
    },
}

fn get_checksum(source: ChecksumSource) -> Result<String, ExtensionError> {
    match source {
        ChecksumSource::ExternalRegistryManifest(manifest) => Ok(manifest.sha256),
        ChecksumSource::DfinityGithubReleases {
            archive_filename,
            git_tag,
        } => {
            let url = format!(
                "{DFINITY_DFX_EXTENSIONS_RELEASES_URL}/{git_tag}/{archive_filename}.tar.gz.sha256",
            );
            let response = reqwest::blocking::get(&url)
                .map_err(|e| ExtensionError::CompatibilityMatrixFetchError(url.to_string(), e))?;
            let bytes = response
                .bytes()
                .map_err(|e| ExtensionError::CompatibilityMatrixFetchError(url.to_string(), e))?;
            let checksum = String::from_utf8_lossy(&bytes)
                .split_ascii_whitespace() // `sha -a 256` output is `<checksum>    <filename>`
                .next()
                .unwrap_or_default()
                .to_string();
            Ok(checksum)
        }
    }
}

fn get_extension_download_url(
    github_release_tag: &str,
    extension_archive_name: &str,
) -> Result<Url, ExtensionError> {
    let download_url = format!("{DFINITY_DFX_EXTENSIONS_RELEASES_URL}/{github_release_tag}/{extension_archive_name}.tar.gz",);
    Url::parse(&download_url)
        .map_err(|e| ExtensionError::MalformedExtensionDownloadUrl(download_url, e))
}

fn get_git_release_tag(extension_name: &str, extension_verion: &Version) -> String {
    format!("{extension_name}-v{extension_verion}",)
}

fn get_extension_archive_name(
    extension_name: &str,
    extension_version: &Version,
) -> Result<String, ExtensionError> {
    let (arch, platform) = get_arch_and_platform()?;
    Ok(format!(
        "{extension_name}-v{extension_version}-{arch}-{platform}"
    ))
}

fn get_arch_and_platform() -> Result<(&'static str, &'static str), ExtensionError> {
    let platform = match std::env::consts::OS {
        "linux" => "unknown-linux-gnu",
        "macos" => "apple-darwin",
        // "windows" => "pc-windows-msvc",
        unsupported_platform => {
            return Err(ExtensionError::PlatformNotSupported(
                unsupported_platform.to_string(),
            ))
        }
    };
    let arch = std::env::consts::ARCH;
    Ok((arch, platform))
}

fn download(download_url: &Url) -> Result<Bytes, ExtensionError> {
    let response = reqwest::blocking::get(download_url.clone())
        .map_err(|e| ExtensionError::ExtensionDownloadFailed(download_url.clone(), e))?;
    let bytes = response
        .bytes()
        .map_err(|e| ExtensionError::ExtensionDownloadFailed(download_url.clone(), e))?;
    Ok(bytes)
}

fn verify_checksum(bytes: &Bytes, checksum: &str) -> Result<(), String> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash = hasher.finalize();
    let hash = hex::encode(hash);
    if hash != checksum {
        return Err(hash);
    }
    Ok(())
}

fn extract_archive(path: &Path, bytes: Bytes) -> Result<(), std::io::Error> {
    let mut archive = Archive::new(GzDecoder::new(Cursor::new(bytes)));
    archive.unpack(path)
}
