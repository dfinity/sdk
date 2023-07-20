#![allow(dead_code)]

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtensionError {
    #[error(transparent)]
    Io(#[from] crate::error::unified_io::UnifiedIoError),

    // errors related to extension directory management
    #[error("Cannot find cache directory: '{0}'")]
    FindCacheDirectoryFailed(crate::error::cache::CacheError),

    #[error("Cannot get extensions directory: {0}")]
    EnsureExtensionDirExistsFailed(crate::error::fs::FsError),

    #[error("Extension directory '{0}' does not exist.")]
    ExtensionDirDoesNotExist(std::path::PathBuf),

    #[error("Extension '{0}' not installed.")]
    ExtensionNotInstalled(String),

    // errors related to installing extensions
    #[error("Extension '{0}' is already installed.")]
    ExtensionAlreadyInstalled(String),

    #[error("Extension '{0}' cannot be installed because it conflicts with an existing command. Consider using '--install-as' flag to install this extension under different name.")]
    CommandAlreadyExists(String),

    #[error("Cannot fetch compatibility.json from '{0}': {1}")]
    CompatibilityMatrixFetchError(String, reqwest::Error),

    #[error("Cannot parse compatibility.json: {0}")]
    MalformedCompatibilityMatrix(reqwest::Error),

    #[error("Cannot parse compatibility.json due to malformed semver '{0}': {1}")]
    MalformedVersionsEntryForExtensionInCompatibilityMatrix(String, semver::Error),

    #[error("Cannot find compatible extension for dfx version '{1}': compatibility.json (downloaded from '{0}') has empty list of extension versions.")]
    ListOfVersionsForExtensionIsEmpty(String, semver::Version),

    #[error("Cannot parse extension manifest URL '{0}': {1}")]
    MalformedExtensionDownloadUrl(String, url::ParseError),

    #[error("DFX version '{0}' is not supported.")]
    DfxVersionNotFoundInCompatibilityJson(semver::Version),

    #[error("Extension '{0}' (version '{1}') not found for DFX version {2}.")]
    ExtensionVersionNotFoundInRepository(String, semver::Version, String),

    #[error("Downloading extension from '{0}' failed: {1}")]
    ExtensionDownloadFailed(url::Url, reqwest::Error),

    #[error("Cannot decompress extension archive (downloaded from: '{0}'): {1}")]
    DecompressFailed(url::Url, std::io::Error),

    #[error("Cannot create temporary directory at '{0}': {1}")]
    CreateTemporaryDirectoryFailed(std::path::PathBuf, std::io::Error),

    #[error("Failed to download extension, because the checksum of the downloaded archive (sha256:{0}) (downloaded from: '{1}') doesn't match the one provided by the manifest (sha256:{2})")]
    ChecksumMismatch(String, String, String),

    #[error("Platform '{0}' is not supported.")]
    PlatformNotSupported(String),

    // errors related to installing extensions from 3rd party registries
    #[error("Extension manifest URL '{0}' is not valid: {1}")]
    InvalidExternalManifestUrl(String, url::ParseError),

    #[error("Entry 'binaries' not found in extension manifest entry for extension '{0}' version '{1}'. Please contact the extension author.")]
    BinaryEntryNotFoundInExtensionManifest(String, semver::Version),

    #[error("Entry 'extensions.{0}' not found in extension manifest entry.")]
    ExtensionNameNotFoundInManifest(String),

    #[error("Extension '{0}' was found in the manifest in both 'comppatibility' and 'extensions' entries, however, the latest compatible version of the extension ({1}) could not be found in 'extensions' entry. Please contact the extension author.")]
    MalformedManifestExtensionVersionNotFound(String, semver::Version),

    #[error("Cannot save extension manifest: {0}")]
    SaveExtensionManifestFailed(crate::error::structured_file::StructuredFileError),

    // errors related to uninstalling extensions
    #[error("Cannot uninstall extension: {0}")]
    InsufficientPermissionsToDeleteExtensionDirectory(crate::error::fs::FsError),

    // errors related to listing extensions
    #[error("Cannot list extensions: {0}")]
    ExtensionsDirectoryIsNotReadable(crate::error::fs::FsError),

    #[error("Cannot load extension manifest: {0}")]
    LoadExtensionManifestFailed(crate::error::structured_file::StructuredFileError),

    // errors related to executing extensions
    #[error("Invalid extension name '{0:?}'.")]
    InvalidExtensionName(std::ffi::OsString),

    #[error("Extension's subcommand argument '{0}' is missing description.")]
    ExtensionSubcommandArgMissingDescription(String),

    #[error("Cannot find extension binary at '{0}'.")]
    ExtensionBinaryDoesNotExist(std::path::PathBuf),

    #[error("Extension binary at {0} is not an executable file.")]
    ExtensionBinaryIsNotAFile(std::path::PathBuf),

    #[error("Failed to run extension '{0}': {1}")]
    FailedToLaunchExtension(String, std::io::Error),

    #[error("Extension '{0}' never finished: {1}")]
    ExtensionNeverFinishedExecuting(String, std::io::Error),

    #[error("Extension terminated by signal.")]
    ExtensionExecutionTerminatedViaSignal,

    #[error("Extension exited with non-zero status code '{0}'.")]
    ExtensionExitedWithNonZeroStatus(i32),
}
