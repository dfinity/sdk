use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProjectError {
    #[error(transparent)]
    StructuredFileError(#[from] dfx_core::error::structured_file::StructuredFileError),

    #[error(transparent)]
    IoError(#[from] dfx_core::error::io::IoError),

    // Q: ok to handle Infallible?
    #[error("Can't convert string '{0}' to path: {1}")]
    ConvertingStringToPathFailed(String, std::convert::Infallible),

    // Q: can also pass in a string, e.g. `format!("{}/{}", host, path)`
    #[error("Invalid URL: {0}")]
    InvalidUrl(url::ParseError),

    #[error("The key 'canisters' is missing dfx.json.")]
    DfxJsonMissingCanisters,

    #[error("The value behind the key 'canisters' in dfx.json is not an object.")]
    DfxJsonCanistersNotObject,
    // Q: combine these two errors into one?
    // upper is a bit more specific because it menions dfx.json
    // but lower is more general, however i dont know if it's dfx.json specific
    #[error("The value behind the key '{0}' is not an JSON object.")]
    NotJsonObject(String),

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

    #[error("Failed to GET resource located at '{0}': {1}")]
    GettingResourceReturnedHTTPError(url::Url, reqwest::Error),

    #[error("Failed to get body from '{0}': {1}")]
    FailedToGetBodyFromResponse(url::Url, reqwest::Error),

    #[error("Malformed network mapping '{0}': {1} network name is empty")]
    MalformedNetworkMapping(String, String),
}
