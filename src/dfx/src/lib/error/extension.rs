#![allow(dead_code)]

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtensionError {
    // errors related to extension directory management
    #[error("Cannot find cache directory: '{0}'")]
    FindCacheDirectoryFailed(dfx_core::error::cache::CacheError),

    #[error("Cannot get extensions directory: {0}")]
    EnsureExtensionDirExistsFailed(dfx_core::error::fs::FsError),

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

    #[error(transparent)]
    Io(#[from] dfx_core::error::fs::FsError),

    #[error("Platform '{0}' is not supported.")]
    PlatformNotSupported(String),

    // errors related to uninstalling extensions
    #[error("Cannot uninstall extension: {0}")]
    InsufficientPermissionsToDeleteExtensionDirectory(dfx_core::error::fs::FsError),

    // errors related to listing extensions
    #[error("Cannot list extensions: {0}")]
    ExtensionsDirectoryIsNotReadable(dfx_core::error::fs::FsError),

    #[error("Malformed extension manifest: {0}")]
    ExtensionManifestIsNotValid(dfx_core::error::structured_file::StructuredFileError),

    #[error("Missing 'extension.json' file for extension '{0}' (exact location: {1}).")]
    ExtensionManifestMissing(String, std::path::PathBuf),

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
