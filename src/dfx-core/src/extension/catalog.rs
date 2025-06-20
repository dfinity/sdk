use crate::error::extension::FetchCatalogError;
use crate::extension::url::ExtensionJsonUrl;
use crate::http::get::get_with_retries;
use crate::json::structure::UrlWithJsonSchema;
use backoff::exponential::ExponentialBackoff;
use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

const DEFAULT_CATALOG_URL: &str =
    "https://raw.githubusercontent.com/dfinity/dfx-extensions/main/catalog.json";

#[derive(Deserialize, Debug, JsonSchema)]
pub struct ExtensionCatalog(pub HashMap<String, UrlWithJsonSchema>);

impl ExtensionCatalog {
    pub async fn fetch(url: Option<&Url>) -> Result<Self, FetchCatalogError> {
        let url: Option<Url> = url.cloned();
        let url = url.unwrap_or_else(|| Url::parse(DEFAULT_CATALOG_URL).unwrap());
        let retry_policy = ExponentialBackoff {
            max_elapsed_time: Some(Duration::from_secs(60)),
            ..Default::default()
        };
        let resp = get_with_retries(url, retry_policy)
            .await
            .map_err(FetchCatalogError::Get)?;

        resp.json().await.map_err(FetchCatalogError::ParseJson)
    }

    pub fn lookup(&self, name: &str) -> Option<ExtensionJsonUrl> {
        self.0
            .get(name)
            .map(|url| ExtensionJsonUrl::new(url.0.clone()))
    }
}
