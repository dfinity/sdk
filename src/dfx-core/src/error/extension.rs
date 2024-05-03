#![allow(dead_code)]
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtensionError {
    // errors related to extension directory management
    #[error("Cannot find cache directory")]
    FindCacheDirectoryFailed(#[source] crate::error::cache::CacheError),

    #[error("Cannot get extensions directory")]
    EnsureExtensionDirExistsFailed(#[source] crate::error::fs::FsError),

    #[error("Extension directory '{0}' does not exist.")]
    ExtensionDirDoesNotExist(std::path::PathBuf),

    #[error("Extension '{0}' not installed.")]
    ExtensionNotInstalled(String),

    // errors related to installing extensions
    #[error("Extension '{0}' is already installed.")]
    ExtensionAlreadyInstalled(String),

    #[error("Extension '{0}' cannot be installed because it conflicts with an existing command. Consider using '--install-as' flag to install this extension under different name.")]
    CommandAlreadyExists(String),

    #[error("Cannot fetch compatibility.json from '{0}'")]
    CompatibilityMatrixFetchError(String, #[source] reqwest::Error),

    #[error("Cannot parse compatibility.json")]
    MalformedCompatibilityMatrix(#[source] reqwest::Error),

    #[error("Cannot parse compatibility.json due to malformed semver '{0}'")]
    MalformedVersionsEntryForExtensionInCompatibilityMatrix(String, #[source] semver::Error),

    #[error("Cannot find compatible extension for dfx version '{1}': compatibility.json (downloaded from '{0}') has empty list of extension versions.")]
    ListOfVersionsForExtensionIsEmpty(String, semver::Version),

    #[error("Cannot parse extension manifest URL '{0}'")]
    MalformedExtensionDownloadUrl(String, #[source] url::ParseError),

    #[error("DFX version '{0}' is not supported.")]
    DfxVersionNotFoundInCompatibilityJson(semver::Version),

    #[error("Extension '{0}' (version '{1}') not found for DFX version {2}.")]
    ExtensionVersionNotFoundInRepository(String, semver::Version, String),

    #[error("Downloading extension from '{0}' failed")]
    ExtensionDownloadFailed(url::Url, #[source] reqwest::Error),

    #[error("Cannot decompress extension archive (downloaded from: '{0}')")]
    DecompressFailed(url::Url, #[source] std::io::Error),

    #[error("Cannot create temporary directory at '{0}'")]
    CreateTemporaryDirectoryFailed(std::path::PathBuf, #[source] std::io::Error),

    #[error(transparent)]
    Io(#[from] crate::error::fs::FsError),

    #[error("Platform '{0}' is not supported.")]
    PlatformNotSupported(String),

    // errors related to uninstalling extensions
    #[error("Cannot uninstall extension")]
    InsufficientPermissionsToDeleteExtensionDirectory(#[source] crate::error::fs::FsError),

    // errors related to listing extensions
    #[error("Cannot list extensions")]
    ExtensionsDirectoryIsNotReadable(#[source] crate::error::fs::FsError),

    #[error("Cannot load extension manifest")]
    LoadExtensionManifestFailed(#[source] crate::error::structured_file::StructuredFileError),

    // errors related to executing extensions
    #[error("Invalid extension name '{0:?}'.")]
    InvalidExtensionName(std::ffi::OsString),

    #[error("Extension's subcommand argument '{0}' is missing description.")]
    ExtensionSubcommandArgMissingDescription(String),

    #[error("Cannot find extension binary at '{0}'.")]
    ExtensionBinaryDoesNotExist(std::path::PathBuf),

    #[error("Extension binary at {0} is not an executable file.")]
    ExtensionBinaryIsNotAFile(std::path::PathBuf),

    #[error("Failed to run extension '{0}'")]
    FailedToLaunchExtension(String, #[source] std::io::Error),

    #[error("Extension '{0}' never finished")]
    ExtensionNeverFinishedExecuting(String, #[source] std::io::Error),

    #[error("Extension terminated by signal.")]
    ExtensionExecutionTerminatedViaSignal,

    #[error("Extension exited with non-zero status code '{0}'.")]
    ExtensionExitedWithNonZeroStatus(i32),
}
