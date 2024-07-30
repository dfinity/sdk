use dfx_core::error::fs::{
    CanonicalizePathError, CreateDirAllError, ReadFileError, WriteFileError,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProjectError {
    #[error(transparent)]
    CanonicalizePath(#[from] CanonicalizePathError),

    #[error(transparent)]
    CreateDirAll(#[from] CreateDirAllError),

    #[error(transparent)]
    StructuredFileError(#[from] dfx_core::error::structured_file::StructuredFileError),

    #[error(transparent)]
    IoError(#[from] dfx_core::error::fs::FsError),

    #[error(transparent)]
    ReadFile(#[from] ReadFileError),

    #[error(transparent)]
    WriteFile(#[from] WriteFileError),

    #[error("Can't convert string '{0}' to path")]
    ConvertingStringToPathFailed(String, #[source] std::convert::Infallible),

    #[error("Tried joining '{0}' and '{1}', but they form an invalid URL")]
    InvalidUrl(url::Url, String, #[source] url::ParseError),

    #[error("The key 'canisters' is missing in dfx.json.")]
    DfxJsonMissingCanisters,

    #[error("The '{0}' value in dfx.json is not an object.")]
    ValueInDfxJsonIsNotJsonObject(String),

    #[error("Unable to parse as url or file")]
    UnableToParseAsUrlOrFile(#[source] url::ParseError),

    #[error("Could not create HTTP client")]
    CouldNotCreateHttpClient(#[source] reqwest::Error),

    #[error("Failed to load project definition from '{0}'")]
    FailedToLoadProjectDefinition(url::Url, #[source] serde_json::Error),

    #[error("Failed to load canister ids from '{0}'")]
    FailedToLoadCanisterIds(url::Url, #[source] serde_json::Error),

    #[error("Failed to get contents of URL '{0}'.")]
    NotFound404(url::Url),

    #[error("Failed to GET resource located at '{0}'")]
    FailedToGetResource(url::Url, #[source] reqwest::Error),

    #[error("Failed to GET resource located at '{0}', server returned an error")]
    GettingResourceReturnedHTTPError(url::Url, #[source] reqwest::Error),

    #[error("Failed to get body from '{0}'")]
    FailedToGetBodyFromResponse(url::Url, #[source] reqwest::Error),

    #[error("Malformed network mapping '{0}': {1} network name is empty")]
    MalformedNetworkMapping(String, String),
}
