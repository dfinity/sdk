use crate::error::extension::LoadExtensionManifestError;
use crate::error::fs::{
    CanonicalizePathError, CreateDirAllError, EnsureDirExistsError, NoParentPathError,
};
use crate::error::get_user_home::GetUserHomeError;
use handlebars::RenderError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed to ensure config directory exists")]
    EnsureConfigDirectoryExistsFailed(#[source] EnsureDirExistsError),

    #[error("Failed to determine config directory path")]
    DetermineConfigDirectoryFailed(#[source] GetUserHomeError),

    #[error("Failed to determine shared network data directory")]
    DetermineSharedNetworkDirectoryFailed(#[source] GetUserHomeError),
}

#[derive(Error, Debug)]
pub enum GetOutputEnvFileError {
    #[error("failed to canonicalize output_env_file")]
    CanonicalizePath(#[from] CanonicalizePathError),

    #[error("The output_env_file must be within the project root, but is {}", .0.display())]
    OutputEnvFileMustBeInProjectRoot(PathBuf),

    #[error("The output_env_file must be a relative path, but is {}", .0.display())]
    OutputEnvFileMustBeRelative(PathBuf),

    #[error(transparent)]
    NoParentPath(#[from] NoParentPathError),
}

#[derive(Error, Debug)]
pub enum GetTempPathError {
    #[error(transparent)]
    CreateDirAll(#[from] CreateDirAllError),
}

#[derive(Error, Debug)]
#[error("failed to render field '{field}' with value '{value}'")]
pub struct RenderErrorWithContext {
    pub field: String,
    pub value: String,
    pub source: RenderError,
}

#[derive(Error, Debug)]
#[error("failed to apply extension canister type '{extension}' to canister '{canister}'")]
pub struct ApplyExtensionCanisterTypeErrorWithContext {
    pub canister: Box<String>,
    pub extension: Box<String>,
    pub source: ApplyExtensionCanisterTypeError,
}

#[derive(Error, Debug)]
pub enum ApplyExtensionCanisterTypesError {
    #[error("the canisters field in dfx.json must be an object")]
    CanistersFieldIsNotAnObject(),

    #[error("canister '{0}' in dfx.json must be an object")]
    CanisterIsNotAnObject(String),

    #[error(transparent)]
    ApplyExtensionCanisterType(#[from] ApplyExtensionCanisterTypeError),
}

#[derive(Error, Debug)]
pub enum ApplyExtensionCanisterTypeError {
    #[error("failed to apply defaults from extension '{extension}' to canister '{canister}'")]
    ApplyDefaults {
        canister: Box<String>,
        extension: Box<String>,
        source: ApplyExtensionCanisterTypeDefaultsError,
    },

    #[error("canister '{canister}' has unknown type '{extension}' and there is no installed extension by that name which could define it")]
    NoExtensionForUnknownCanisterType { canister: String, extension: String },

    #[error(transparent)]
    LoadExtensionManifest(LoadExtensionManifestError),

    #[error("canister '{canister}' has type '{extension}', but that extension does not define a canister type")]
    ExtensionDoesNotDefineCanisterType { canister: String, extension: String },
}

#[derive(Error, Debug)]
pub enum ApplyExtensionCanisterTypeDefaultsError {
    #[error(transparent)]
    AppendMetadata(#[from] AppendMetadataError),

    #[error(transparent)]
    MergeTechStackError(#[from] MergeTechStackError),

    #[error(transparent)]
    Render(Box<RenderErrorWithContext>),
}

#[derive(Error, Debug)]
pub enum AppendMetadataError {
    #[error("expected canister metadata to be an array")]
    ExpectedCanisterMetadataArray,

    #[error("expected extension canister type metadata to be an array")]
    ExpectedExtensionCanisterTypeMetadataArray,
}

#[derive(Error, Debug)]
pub enum MergeTechStackError {
    #[error("expected canister tech_stack to be an object")]
    ExpectedCanisterTechStackObject,

    #[error("expected extension canister type tech_stack to be an object")]
    ExpectedExtensionCanisterTypeTechStackObject,
}
