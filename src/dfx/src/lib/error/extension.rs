use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtensionError {
    // errors related to extension directory management
    #[error("Cannot find cache directory at '{0}'.")]
    FindCacheDirectoryFailed(std::path::PathBuf, anyhow::Error),

    #[error("Cannot create extensions directory at '{0}'.")]
    CreateExtensionDirectoryFailed(std::path::PathBuf),

    #[error("Extension '{0}' not installed.")]
    ExtensionNotInstalled(String),

    #[error("Extensions directory is not a directory.")]
    ExtensionsDirectoryIsNotADirectory,

    // errors related to installing extensions
    #[error("Extension '{0}' is already installed.")]
    ExtensionAlreadyInstalled(String),

    #[error("Cannot fetch compatibility.json from '{0}'.")]
    CompatibilityMatrixFetchError(String),

    #[error("Cannot parse compatibility.json due to error '{0}'.")]
    MalformedCompatibilityMatrix(reqwest::Error),

    #[error("Cannot parse compatibility.json due to error '{0}'.")]
    MalformedVersionsEntryForExtensionInCompatibilityMatrix(String),

    #[error("Cannot parse compatibility.json due to error '{0}'.")]
    ListOfVersionsForExtensionIsEmpty(String),

    #[error("Cannot parse extension manifest '{0}'.")]
    MalformedExtensionDownloadUrl(url::ParseError),

    #[error("DFX version '{0}' is not supported.")]
    DfxVersionNotFoundInCompatibilityJson(semver::Version, String),

    #[error("Extension '{0}' (version '{1}') not found for DFX version {2}.")]
    ExtensionVersionNotFoundInRepository(String, String, String),

    #[error("Extension '{0}' download failed.")]
    ExtensionDownloadFailed(url::Url),

    #[error("Cannot decompress extension archive (downloaded from: '{0}'), due to error: '{1}'.")]
    DecompressFailed(url::Url, std::io::Error),

    #[error("Cannot create temporary directory at '{0}'.")]
    CreateTemporaryDirectoryFailed(std::path::PathBuf),

    #[cfg(not(target_os = "windows"))]
    #[error("Insufficient permissions to open extension's binary permissions '{0}'.")]
    InsufficientPermissionsToOpenExtensionBinaryInWriteMode(String),

    #[cfg(not(target_os = "windows"))]
    #[error("Cannot change file permissions at '{0}', due to error: {1}.")]
    ChangeFilePermissionsFailed(std::path::PathBuf, std::io::Error),

    #[error("Cannot rename directory at '{0}'.")]
    RenameDirectoryFailed(std::io::Error),

    // errors related to uninstalling extensions
    #[error("Cannot uninstall extension '{0}'.")]
    InsufficientPermissionsToDeleteExtensionDirectory(std::io::Error),

    // errors related to listing extensions
    #[error("Cannot list extensions.")]
    ExtensionsDirectoryIsNotReadable(std::io::Error),

    #[error("Malformed extension manifest ({0}): '{1}'.")]
    ExtensionManifestIsNotValidJson(std::path::PathBuf, serde_json::Error),

    #[error("Malformed extension manifest ({0})..")]
    ExtensionManifestDoesNotExist(std::path::PathBuf, std::io::Error),

    // errors related to executing extensions
    #[error("Invalid extension name '{0}'.")]
    InvalidExtensionName(String),

    #[error("Cannot find extension binary '{0}'.")]
    ExtensionBinaryDoesNotExist(String),

    #[error("Extension is not an executable file '{0}'.")]
    ExtensionBinaryIsNotAFile(String),

    #[error("Failed to run extension '{0}'.")]
    FailedToLaunchExtension(String),

    #[error("Extension never finished '{0}'.")]
    ExtensionNeverFinishedExecuting(String),

    #[cfg(not(target_os = "windows"))]
    #[error("Extension terminated by signal.")]
    ExtensionExecutionTerminatedViaSignal,

    #[error("Extension exited with non-zero status code '{0}'.")]
    ExtensionExitedWithNonZeroStatus(i32),
}
