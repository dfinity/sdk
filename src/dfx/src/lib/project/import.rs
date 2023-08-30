use crate::lib::error::ProjectError;
use dfx_core::config::model::canister_id_store;
use dfx_core::config::model::canister_id_store::CanisterIds;
use dfx_core::config::model::dfinity::Config;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::{Map, Value};
use slog::{info, Logger};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use url::Url;

#[derive(Clone, Debug, Deserialize)]
struct DfxJsonCanister {
    pub candid: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct DfxJsonProject {
    pub canisters: BTreeMap<String, DfxJsonCanister>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportNetworkMapping {
    pub network_name_in_this_project: String,
    pub network_name_in_project_being_imported: String,
}

/// import canister definitions from another project.
/// their_dfx_json_location can either be a URL or a local file path.
pub async fn import_canister_definitions(
    logger: &Logger,
    config: &mut Config,
    their_dfx_json_location: &str,
    prefix: Option<&str>,
    import_only_canister_name: Option<String>,
    network_mappings: &[ImportNetworkMapping],
) -> Result<(), ProjectError> {
    let mut loader = Loader::new();

    let their_dfx_json_url = location_to_url(their_dfx_json_location)?;
    let their_canister_ids_json_url =
        their_dfx_json_url.join("canister_ids.json").map_err(|e| {
            ProjectError::InvalidUrl(
                their_dfx_json_url.clone(),
                "canister_ids.json".to_string(),
                e,
            )
        })?;

    let what = if let Some(ref name) = import_only_canister_name {
        format!("canister '{}'", name)
    } else {
        "all canisters".to_string()
    };
    info!(logger, "Importing {} from {}", what, their_dfx_json_url);

    let their_project = loader.load_project_definition(&their_dfx_json_url).await?;
    let their_canister_ids = loader
        .load_canister_ids(&their_canister_ids_json_url)
        .await?;

    let our_project_root = config.get_project_root().to_path_buf();
    let candid_output_dir = our_project_root.join("candid");
    dfx_core::fs::create_dir_all(&candid_output_dir)?;

    let config_canisters_object = get_canisters_json_object(config)?;

    for (their_canister_name, their_canister) in their_project.canisters {
        if matches!(import_only_canister_name, Some(ref n) if *n != their_canister_name) {
            continue;
        }
        if let Some(ref their_relative_candid) = their_canister.candid {
            let our_canister_name = format!("{}{}", prefix.unwrap_or(""), their_canister_name);
            info!(
                logger,
                "Importing canister '{}' as '{}'", their_canister_name, our_canister_name
            );

            let our_canister_definition =
                ensure_child_object(config_canisters_object, &our_canister_name)?;

            import_candid_definition(
                logger,
                &mut loader,
                &their_dfx_json_url,
                &our_project_root,
                their_relative_candid,
                &our_canister_name,
                our_canister_definition,
            )
            .await?;

            set_remote_canister_ids(
                logger,
                &their_canister_name,
                network_mappings,
                &their_canister_ids,
                our_canister_definition,
            )?;

            set_additional_fields(our_canister_definition);
        }
    }

    config.save()?;

    Ok(())
}

async fn import_candid_definition(
    logger: &Logger,
    loader: &mut Loader,
    their_dfx_json_url: &Url,
    our_project_root: &Path,
    their_relative_candid: &str,
    our_canister_name: &str,
    our_canister: &mut Map<String, Value>,
) -> Result<(), ProjectError> {
    let our_relative_candid_path = format!("candid/{}.did", our_canister_name);
    let their_candid_url = their_dfx_json_url
        .join(their_relative_candid)
        .map_err(|e| {
            ProjectError::InvalidUrl(
                their_dfx_json_url.clone(),
                their_relative_candid.to_string(),
                e,
            )
        })?;
    let our_candid_path_incl_project_root = our_project_root.join(&our_relative_candid_path);
    info!(
        logger,
        "Importing {} from {}",
        our_candid_path_incl_project_root.display(),
        their_candid_url,
    );
    let candid_definition = loader.get_required_url_contents(&their_candid_url).await?;
    dfx_core::fs::write(&our_candid_path_incl_project_root, candid_definition)?;

    our_canister.insert(
        "candid".to_string(),
        Value::String(our_relative_candid_path),
    );
    Ok(())
}

pub fn get_canisters_json_object(
    config: &mut Config,
) -> Result<&mut Map<String, Value>, ProjectError> {
    let config_canisters_object = config
        .get_mut_json()
        .pointer_mut("/canisters")
        .ok_or(ProjectError::DfxJsonMissingCanisters)?
        .as_object_mut()
        .ok_or_else(|| ProjectError::ValueInDfxJsonIsNotJsonObject("/canisters".to_string()))?;
    Ok(config_canisters_object)
}

pub fn set_remote_canister_ids(
    logger: &Logger,
    their_canister_name: &str,
    network_mappings: &[ImportNetworkMapping],
    their_canister_ids: &CanisterIds,
    canister: &mut Map<String, Value>,
) -> Result<(), ProjectError> {
    for network_mapping in network_mappings {
        let remote_canister_id = their_canister_ids
            .get(their_canister_name)
            .and_then(|c| c.get(&network_mapping.network_name_in_project_being_imported));
        if let Some(remote_canister_id) = remote_canister_id {
            let remote = ensure_child_object(canister, "remote")?;
            let id = ensure_child_object(remote, "id")?;
            id.insert(
                network_mapping.network_name_in_this_project.clone(),
                Value::String(remote_canister_id.clone()),
            );
            info!(
                logger,
                "{} canister id on network '{}' is {}",
                their_canister_name,
                network_mapping.network_name_in_this_project,
                remote_canister_id,
            );
        } else {
            info!(
                logger,
                "{} has no canister id for network '{}'",
                their_canister_name,
                network_mapping.network_name_in_this_project
            );
        }
    }
    Ok(())
}

fn set_additional_fields(our_canister: &mut Map<String, Value>) {
    our_canister.insert("type".to_string(), Value::String("custom".to_string()));
    our_canister.insert("build".to_string(), Value::String("".to_string()));
    our_canister.insert("wasm".to_string(), Value::String("".to_string()));
}

fn ensure_child_object<'a>(
    parent: &'a mut Map<String, Value>,
    name: &str,
) -> Result<&'a mut Map<String, Value>, ProjectError> {
    if !parent.contains_key(name) {
        parent.insert(name.to_string(), Value::Object(Map::new()));
    }
    parent
        .get_mut(name)
        .unwrap() // we just added it
        .as_object_mut()
        .ok_or_else(|| ProjectError::ValueInDfxJsonIsNotJsonObject(name.to_string()))
}

fn location_to_url(dfx_json_location: &str) -> Result<Url, ProjectError> {
    Url::parse(dfx_json_location).or_else(|url_error| {
        let path = PathBuf::from_str(dfx_json_location).map_err(|e| {
            ProjectError::ConvertingStringToPathFailed(dfx_json_location.to_string(), e)
        })?;
        let canonical = dfx_core::fs::canonicalize(&path)?;

        Url::from_file_path(canonical)
            .map_err(|_file_error_is_unit| ProjectError::UnableToParseAsUrlOrFile(url_error))
    })
}

struct Loader {
    client: Option<Client>,
}

impl Loader {
    fn new() -> Self {
        Loader { client: None }
    }

    fn client(&mut self) -> Result<&Client, ProjectError> {
        if self.client.is_none() {
            let client = reqwest::Client::builder()
                .use_rustls_tls()
                .build()
                .map_err(ProjectError::CouldNotCreateHttpClient)?;
            self.client = Some(client);
        }
        Ok(self.client.as_ref().unwrap())
    }

    async fn load_project_definition(&mut self, url: &Url) -> Result<DfxJsonProject, ProjectError> {
        let body = self.get_required_url_contents(url).await?;
        let project = serde_json::from_slice(&body)
            .map_err(|e| ProjectError::FailedToLoadProjectDefinition(url.clone(), e))?;
        Ok(project)
    }

    async fn load_canister_ids(
        &mut self,
        url: &Url,
    ) -> Result<canister_id_store::CanisterIds, ProjectError> {
        match self.get_optional_url_contents(url).await? {
            None => Ok(canister_id_store::CanisterIds::new()),
            Some(body) => serde_json::from_slice(&body)
                .map_err(|e| ProjectError::FailedToLoadCanisterIds(url.clone(), e)),
        }
    }

    async fn get_required_url_contents(&mut self, url: &Url) -> Result<Vec<u8>, ProjectError> {
        self.get_optional_url_contents(url)
            .await?
            .ok_or_else(|| ProjectError::NotFound404(url.clone()))
    }

    async fn get_optional_url_contents(
        &mut self,
        url: &Url,
    ) -> Result<Option<Vec<u8>>, ProjectError> {
        if url.scheme() == "file" {
            Self::read_optional_file_contents(&PathBuf::from(url.path()))
        } else {
            self.get_optional_url_body(url).await
        }
    }

    fn read_optional_file_contents(path: &Path) -> Result<Option<Vec<u8>>, ProjectError> {
        if path.exists() {
            let contents = dfx_core::fs::read(path)?;
            Ok(Some(contents))
        } else {
            Ok(None)
        }
    }

    async fn get_optional_url_body(&mut self, url: &Url) -> Result<Option<Vec<u8>>, ProjectError> {
        let client = self.client()?;
        let response = client
            .get(url.clone())
            .send()
            .await
            .map_err(|e| ProjectError::FailedToGetResource(url.clone(), e))?;
        if response.status() == StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            let body = response
                .error_for_status()
                .map_err(|e| ProjectError::GettingResourceReturnedHTTPError(url.clone(), e))?
                .bytes()
                .await
                .map_err(|e| ProjectError::FailedToGetBodyFromResponse(url.clone(), e))?;
            Ok(Some(body.into()))
        }
    }
}
