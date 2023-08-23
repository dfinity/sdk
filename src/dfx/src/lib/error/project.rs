use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProjectError {
    #[error(transparent)]
    StructuredFileError(#[from] dfx_core::error::structured_file::StructuredFileError),

    #[error(transparent)]
    IoError(#[from] dfx_core::error::fs::FsError),

    #[error("Can't convert string '{0}' to path: {1}")]
    ConvertingStringToPathFailed(String, std::convert::Infallible),

    #[error("Tried joining '{0}' and '{1}', but they form an invalid URL: {2}")]
    InvalidUrl(url::Url, String, url::ParseError),

    #[error("The key 'canisters' is missing in dfx.json.")]
    DfxJsonMissingCanisters,

    #[error("The '{0}' value in dfx.json is not an object.")]
    ValueInDfxJsonIsNotJsonObject(String),

    #[error("Unable to parse as url or file: {0}")]
    UnableToParseAsUrlOrFile(url::ParseError),

    #[error("Could not create HTTP client: {0}")]
    CouldNotCreateHttpClient(reqwest::Error),

    #[error("Failed to load project definition from '{0}': {1}")]
    FailedToLoadProjectDefinition(url::Url, serde_json::Error),

    #[error("Failed to load canister ids from '{0}': {1}")]
    FailedToLoadCanisterIds(url::Url, serde_json::Error),

    #[error("Failed to get contents of URL '{0}'.")]
    NotFound404(url::Url),

    #[error("Failed to GET resource located at '{0}': {1}")]
    FailedToGetResource(url::Url, reqwest::Error),

    #[error("Failed to GET resource located at '{0}', server returned an error: {1}")]
    GettingResourceReturnedHTTPError(url::Url, reqwest::Error),

    #[error("Failed to get body from '{0}': {1}")]
    FailedToGetBodyFromResponse(url::Url, reqwest::Error),

    #[error("Malformed network mapping '{0}': {1} network name is empty")]
    MalformedNetworkMapping(String, String),
}
