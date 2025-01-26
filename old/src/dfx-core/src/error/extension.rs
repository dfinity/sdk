#![allow(dead_code)]
use crate::error::structured_file::StructuredFileError;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Failed to load extension manifest")]
pub struct LoadExtensionManifestError(#[from] StructuredFileError);

#[derive(Error, Debug)]
pub enum ConvertExtensionIntoClapCommandError {
    #[error(transparent)]
    LoadExtensionManifest(#[from] LoadExtensionManifestError),

    #[error(transparent)]
    ListInstalledExtensionsError(#[from] ListInstalledExtensionsError),

    #[error(transparent)]
    ConvertExtensionSubcommandIntoClapCommandError(
        #[from] ConvertExtensionSubcommandIntoClapCommandError,
    ),
}

#[derive(Error, Debug)]
pub enum ConvertExtensionSubcommandIntoClapCommandError {
    #[error(transparent)]
    ConvertExtensionSubcommandIntoClapArgError(#[from] ConvertExtensionSubcommandIntoClapArgError),
}

#[derive(Error, Debug)]
pub enum ListInstalledExtensionsError {
    #[error(transparent)]
    ExtensionsDirectoryIsNotReadable(#[from] crate::error::fs::FsError),
}

#[derive(Error, Debug)]
pub enum ConvertExtensionSubcommandIntoClapArgError {
    #[error("Extension's subcommand argument '{0}' is missing description.")]
    ExtensionSubcommandArgMissingDescription(String),
}

#[derive(Error, Debug)]
pub enum RunExtensionError {
    #[error("Invalid extension name '{0:?}'.")]
    InvalidExtensionName(std::ffi::OsString),

    #[error("Cannot find cache directory")]
    FindCacheDirectoryFailed(#[source] crate::error::cache::CacheError),

    #[error("Failed to run extension '{0}'")]
    FailedToLaunchExtension(String, #[source] std::io::Error),

    #[error("Extension '{0}' never finished")]
    ExtensionNeverFinishedExecuting(String, #[source] std::io::Error),

    #[error("Extension terminated by signal.")]
    ExtensionExecutionTerminatedViaSignal,

    #[error("Extension exited with non-zero status code '{0}'.")]
    ExtensionExitedWithNonZeroStatus(i32),

    #[error(transparent)]
    GetExtensionBinaryError(#[from] GetExtensionBinaryError),
}

#[derive(Error, Debug)]
pub enum GetExtensionBinaryError {
    #[error("Extension '{0}' not installed.")]
    ExtensionNotInstalled(String),

    #[error("Cannot find extension binary at '{0}'.")]
    ExtensionBinaryDoesNotExist(std::path::PathBuf),

    #[error("Extension binary at {0} is not an executable file.")]
    ExtensionBinaryIsNotAFile(std::path::PathBuf),
}

#[derive(Error, Debug)]
pub enum NewExtensionManagerError {
    #[error("Cannot find cache directory")]
    FindCacheDirectoryFailed(#[source] crate::error::cache::CacheError),
}

#[derive(Error, Debug)]
pub enum DownloadAndInstallExtensionToTempdirError {
    #[error("Downloading extension from '{0}' failed")]
    ExtensionDownloadFailed(url::Url, #[source] reqwest::Error),

    #[error("Cannot get extensions directory")]
    EnsureExtensionDirExistsFailed(#[source] crate::error::fs::FsError),

    #[error("Cannot create temporary directory at '{0}'")]
    CreateTemporaryDirectoryFailed(std::path::PathBuf, #[source] std::io::Error),

    #[error("Cannot decompress extension archive (downloaded from: '{0}')")]
    DecompressFailed(url::Url, #[source] std::io::Error),
}

#[derive(Error, Debug)]
pub enum InstallExtensionError {
    #[error("Extension '{0}' is already installed.")]
    ExtensionAlreadyInstalled(String),

    #[error(transparent)]
    GetExtensionArchiveName(#[from] GetExtensionArchiveNameError),

    #[error(transparent)]
    FindLatestExtensionCompatibleVersion(#[from] FindLatestExtensionCompatibleVersionError),

    #[error(transparent)]
    GetExtensionDownloadUrl(#[from] GetExtensionDownloadUrlError),

    #[error(transparent)]
    DownloadAndInstallExtensionToTempdir(#[from] DownloadAndInstallExtensionToTempdirError),

    #[error(transparent)]
    FinalizeInstallation(#[from] FinalizeInstallationError),
}

#[derive(Error, Debug)]
pub enum GetExtensionArchiveNameError {
    #[error("Platform '{0}' is not supported.")]
    PlatformNotSupported(String),
}

#[derive(Error, Debug)]
pub enum FindLatestExtensionCompatibleVersionError {
    #[error("DFX version '{0}' is not supported.")]
    DfxVersionNotFoundInCompatibilityJson(semver::Version),

    #[error("Extension '{0}' (version '{1}') not found for DFX version {2}.")]
    ExtensionVersionNotFoundInRepository(String, semver::Version, String),

    #[error("Cannot parse compatibility.json due to malformed semver '{0}'")]
    MalformedVersionsEntryForExtensionInCompatibilityMatrix(String, #[source] semver::Error),

    #[error("Cannot find compatible extension for dfx version '{1}': compatibility.json (downloaded from '{0}') has empty list of extension versions.")]
    ListOfVersionsForExtensionIsEmpty(String, semver::Version),

    #[error(transparent)]
    FetchExtensionCompatibilityMatrix(#[from] FetchExtensionCompatibilityMatrixError),
}

#[derive(Error, Debug)]
#[error("Failed to parse extension manifest URL '{url}'")]
pub struct GetExtensionDownloadUrlError {
    pub url: String,
    pub source: url::ParseError,
}

#[derive(Error, Debug)]
#[error(transparent)]
pub struct FinalizeInstallationError(#[from] crate::error::fs::FsError);

#[derive(Error, Debug)]
pub enum FetchExtensionCompatibilityMatrixError {
    #[error("Cannot fetch compatibility.json from '{0}'")]
    CompatibilityMatrixFetchError(String, #[source] reqwest::Error),

    #[error("Cannot parse compatibility.json")]
    MalformedCompatibilityMatrix(#[source] reqwest::Error),
}

#[derive(Error, Debug)]
#[error(transparent)]
pub struct UninstallExtensionError(#[from] crate::error::fs::FsError);
