use crate::asset::config::{
    AssetConfig, AssetSourceDirectoryConfiguration, ASSETS_CONFIG_FILENAME_JSON,
};
use crate::batch_upload::operations::BATCH_UPLOAD_API_VERSION;
use crate::batch_upload::plumbing::ChunkUploader;
use crate::batch_upload::plumbing::Mode::{ByProposal, NormalDeploy};
use crate::batch_upload::{
    self,
    operations::AssetDeletionReason,
    plumbing::{make_project_assets, AssetDescriptor},
};
use crate::canister_api::methods::batch::{compute_evidence, propose_commit_batch};
use crate::canister_api::methods::{
    api_version::api_version,
    asset_properties::get_assets_properties,
    batch::{commit_batch, create_batch},
    list::list_assets,
};
use crate::canister_api::types::batch_upload::v0;
use crate::canister_api::types::batch_upload::v1::BatchOperationKind;
use crate::canister_api::types::batch_upload::{
    common::ComputeEvidenceArguments, v1::CommitBatchArguments,
};
use crate::error::CompatibilityError::DowngradeV1TOV0Failed;
use crate::error::GatherAssetDescriptorsError;
use crate::error::GatherAssetDescriptorsError::{
    DuplicateAssetKey, InvalidDirectoryEntry, InvalidSourceDirectory, LoadConfigFailed,
};
use crate::error::PrepareSyncForProposalError;
use crate::error::SyncError;
use crate::error::SyncError::CommitBatchFailed;
use crate::error::UploadContentError;
use crate::error::UploadContentError::{CreateBatchFailed, ListAssetsFailed};
use candid::Nat;
use ic_agent::AgentError;
use ic_utils::Canister;
use itertools::Itertools;
use serde_bytes::ByteBuf;
use slog::{debug, info, trace, warn, Logger};
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

const KNOWN_DIRECTORIES: [&str; 1] = [".well-known"];

/// Sets the contents of the asset canister to the contents of a directory, including deleting old assets.
pub async fn upload_content_and_assemble_sync_operations(
    canister: &Canister<'_>,
    canister_api_version: u16,
    dirs: &[&Path],
    no_delete: bool,
    mode: batch_upload::plumbing::Mode,
    logger: &Logger,
    insecure_dev_mode: bool,
) -> Result<CommitBatchArguments, UploadContentError> {
    let asset_descriptors = gather_asset_descriptors(dirs, logger)?;

    let canister_assets = list_assets(canister).await.map_err(ListAssetsFailed)?;
    info!(
        logger,
        "Fetching properties for all assets in the canister."
    );
    let now = std::time::Instant::now();
    let canister_asset_properties = get_assets_properties(canister, &canister_assets).await?;

    info!(
        logger,
        "Done fetching properties for all assets in the canister. Took {:?}",
        now.elapsed()
    );

    info!(logger, "Starting batch.");

    let batch_id = create_batch(canister).await.map_err(CreateBatchFailed)?;

    info!(
        logger,
        "Staging contents of new and changed assets in batch {}:", batch_id
    );

    let chunk_uploader =
        ChunkUploader::new(canister.clone(), canister_api_version, batch_id.clone());

    let project_assets = make_project_assets(
        Some(&chunk_uploader),
        asset_descriptors,
        &canister_assets,
        mode,
        logger,
    )
    .await
    .map_err(UploadContentError::CreateProjectAssetError)?;

    let commit_batch_args = batch_upload::operations::assemble_commit_batch_arguments(
        &chunk_uploader,
        project_assets,
        canister_assets,
        match no_delete {
            true => AssetDeletionReason::Incompatible,
            false => AssetDeletionReason::Obsolete,
        },
        canister_asset_properties,
        batch_id,
        insecure_dev_mode,
    )
    .await
    .map_err(UploadContentError::AssembleCommitBatchArgumentFailed)?;

    // -v
    debug!(
        logger,
        "Count of each Batch Operation Kind: {:?}",
        commit_batch_args.group_by_kind_then_count()
    );
    debug!(
        logger,
        "Chunks: {}  Bytes: {}",
        chunk_uploader.chunks(),
        chunk_uploader.bytes()
    );

    // -vv
    trace!(logger, "Value of CommitBatch: {:?}", commit_batch_args);

    Ok(commit_batch_args)
}

/// Sets the contents of the asset canister to the contents of a directory, including deleting old assets.
pub async fn sync(
    canister: &Canister<'_>,
    dirs: &[&Path],
    no_delete: bool,
    logger: &Logger,
    insecure_dev_mode: bool,
) -> Result<(), SyncError> {
    let canister_api_version = api_version(canister).await;
    let commit_batch_args = upload_content_and_assemble_sync_operations(
        canister,
        canister_api_version,
        dirs,
        no_delete,
        NormalDeploy,
        logger,
        insecure_dev_mode,
    )
    .await?;
    debug!(logger, "Canister API version: {canister_api_version}. ic-asset API version: {BATCH_UPLOAD_API_VERSION}");
    info!(logger, "Committing batch.");
    match canister_api_version {
        0 => {
            let commit_batch_args_v0 = v0::CommitBatchArguments::try_from(commit_batch_args).map_err(DowngradeV1TOV0Failed)?;
            warn!(logger, "The asset canister is running an old version of the API. It will not be able to set assets properties.");
            commit_batch(canister, commit_batch_args_v0).await
        }
        BATCH_UPLOAD_API_VERSION.. => commit_in_stages(canister, commit_batch_args, logger).await,
    }.map_err(CommitBatchFailed)
}

async fn commit_in_stages(
    canister: &Canister<'_>,
    commit_batch_args: CommitBatchArguments,
    logger: &Logger,
) -> Result<(), AgentError> {
    // Note that SetAssetProperties operations are only generated for assets that
    // already exist, since CreateAsset operations set all properties.
    let (set_properties_operations, other_operations): (Vec<_>, Vec<_>) = commit_batch_args
        .operations
        .into_iter()
        .partition(|op| matches!(op, BatchOperationKind::SetAssetProperties(_)));

    // This part seems reasonable in general as a separate batch
    for operations in set_properties_operations.chunks(500) {
        info!(logger, "Setting properties of {} assets.", operations.len());
        commit_batch(
            canister,
            CommitBatchArguments {
                batch_id: Nat::from(0_u8),
                operations: operations.into(),
            },
        )
        .await?
    }

    // Seen to work at 800 ({"SetAssetContent": 932, "Delete": 47, "CreateAsset": 58})
    // so 500 shouldn't exceed per-message instruction limit
    for operations in other_operations.chunks(500) {
        info!(
            logger,
            "Committing batch with {} operations.",
            operations.len()
        );
        commit_batch(
            canister,
            CommitBatchArguments {
                batch_id: Nat::from(0_u8),
                operations: operations.into(),
            },
        )
        .await?
    }

    // this just deletes the batch
    commit_batch(
        canister,
        CommitBatchArguments {
            batch_id: commit_batch_args.batch_id,
            operations: vec![],
        },
    )
    .await
}

/// Stage changes and propose the batch for commit.
pub async fn prepare_sync_for_proposal(
    canister: &Canister<'_>,
    dirs: &[&Path],
    logger: &Logger,
    insecure_dev_mode: bool,
) -> Result<(Nat, ByteBuf), PrepareSyncForProposalError> {
    let canister_api_version = api_version(canister).await;
    let arg = upload_content_and_assemble_sync_operations(
        canister,
        canister_api_version,
        dirs,
        false,
        ByProposal,
        logger,
        insecure_dev_mode,
    )
    .await?;
    let arg = sort_batch_operations(arg);
    let batch_id = arg.batch_id.clone();

    info!(logger, "Preparing batch {}.", batch_id);
    propose_commit_batch(canister, arg)
        .await
        .map_err(PrepareSyncForProposalError::ProposeCommitBatch)?;

    let compute_evidence_arg = ComputeEvidenceArguments {
        batch_id: batch_id.clone(),
        max_iterations: Some(97), // 75% of max(130) = 97.5
    };
    info!(logger, "Computing evidence.");
    let evidence = loop {
        if let Some(evidence) = compute_evidence(canister, &compute_evidence_arg)
            .await
            .map_err(PrepareSyncForProposalError::ComputeEvidence)?
        {
            break evidence;
        }
    };

    info!(logger, "Proposed commit of batch {} with evidence {}.  Either commit it by proposal, or delete it.", batch_id, hex::encode(&evidence));

    Ok((batch_id, evidence))
}

fn sort_batch_operations(mut args: CommitBatchArguments) -> CommitBatchArguments {
    args.operations.sort();
    args
}

fn include_entry(entry: &walkdir::DirEntry, config: &AssetConfig) -> bool {
    if let Some(ignored) = config.ignore {
        !ignored
    } else if let Some(entry_name) = entry.file_name().to_str() {
        let is_known = if entry.path().is_dir() {
            KNOWN_DIRECTORIES.contains(&entry_name)
        } else {
            false
        };
        is_known || !entry_name.starts_with('.')
    } else {
        true
    }
}

pub(crate) fn gather_asset_descriptors(
    dirs: &[&Path],
    logger: &Logger,
) -> Result<Vec<AssetDescriptor>, GatherAssetDescriptorsError> {
    let mut asset_descriptors: HashMap<String, AssetDescriptor> = HashMap::new();
    for dir in dirs {
        let dir = dfx_core::fs::canonicalize(dir).map_err(InvalidSourceDirectory)?;
        let mut configuration =
            AssetSourceDirectoryConfiguration::load(&dir).map_err(LoadConfigFailed)?;
        let mut asset_descriptors_interim = vec![];
        let entries = WalkDir::new(&dir)
            .into_iter()
            .filter_entry(|entry| {
                if let Ok(canonical_path) = &dfx_core::fs::canonicalize(entry.path()) {
                    let config = configuration
                        .get_asset_config(canonical_path)
                        .unwrap_or_default();
                    include_entry(entry, &config)
                } else {
                    false
                }
            })
            .filter_map(|r| r.ok())
            .filter(|entry| {
                entry.file_type().is_file() && entry.file_name() != ASSETS_CONFIG_FILENAME_JSON
            })
            .collect::<Vec<_>>();

        for e in entries {
            let source = dfx_core::fs::canonicalize(e.path()).map_err(InvalidDirectoryEntry)?;
            let relative = source.strip_prefix(&dir).expect("cannot strip prefix");
            let key = String::from("/") + relative.to_string_lossy().as_ref();
            let config = configuration.get_asset_config(&source)?;

            asset_descriptors_interim.push(AssetDescriptor {
                source,
                key,
                config,
            })
        }

        for asset_descriptor in asset_descriptors_interim {
            if let Some(already_seen) = asset_descriptors.get(&asset_descriptor.key) {
                return Err(DuplicateAssetKey(
                    asset_descriptor.key.clone(),
                    Box::new(asset_descriptor.source.clone()),
                    Box::new(already_seen.source.clone()),
                ));
            }
            asset_descriptors.insert(asset_descriptor.key.clone(), asset_descriptor);
        }

        for (config_path, rules) in configuration.get_unused_configs() {
            warn!(
                logger,
                "{count} unmatched configuration{s} in {path}/.ic-assets.json config file:",
                count = rules.len(),
                s = if rules.len() > 1 { "s" } else { "" },
                path = config_path.display()
            );
            for rule in rules {
                warn!(logger, "{}", serde_json::to_string_pretty(&rule).unwrap());
            }
        }

        let no_policy_assets = asset_descriptors
            .values()
            .filter(|asset| asset.config.warn_about_no_security_policy())
            .collect_vec();
        if !no_policy_assets.is_empty() {
            warn!(
                logger,
                "This project does not define a security policy for some assets."
            );
            warn!(
                logger,
                "You should define a security policy in .ic-assets.json5. For example:"
            );
            warn!(logger, "[");
            warn!(logger, "  {{");
            warn!(logger, r#"    "match": "**/*","#);
            warn!(logger, r#"    "security_policy": "standard""#);
            warn!(logger, "  }}");
            warn!(logger, "]");

            if no_policy_assets.len() == asset_descriptors.len() {
                warn!(logger, "Assets without any security policy: all");
            } else {
                warn!(logger, "Assets without any security policy:");
                for asset in &no_policy_assets {
                    warn!(logger, "  - {}", asset.key);
                }
            }
        }
        let standard_policy_assets = asset_descriptors
            .values()
            .filter(|asset| asset.config.warn_about_standard_security_policy())
            .collect_vec();
        if !standard_policy_assets.is_empty() {
            warn!(logger, "This project uses the default security policy for some assets. While it is set up to work with many applications, it is recommended to further harden the policy to increase security against attacks like XSS.");
            warn!(logger, "To get started, have a look at 'dfx info canister-security-policy'. It shows the default security policy along with suggestions on how to improve it.");
            if standard_policy_assets.len() == asset_descriptors.len() {
                warn!(logger, "Unhardened assets: all");
            } else {
                warn!(logger, "Unhardened assets:");
                for asset in &standard_policy_assets {
                    warn!(logger, "  - {}", asset.key);
                }
            }
        }
        if !standard_policy_assets.is_empty() || !no_policy_assets.is_empty() {
            warn!(logger, "To disable the policy warning, define \"disable_security_policy_warning\": true in .ic-assets.json5.");
        }
        let missing_hardening_assets = asset_descriptors
            .values()
            .filter(|asset| asset.config.warn_about_missing_hardening_headers())
            .collect_vec();
        if !missing_hardening_assets.is_empty() {
            let mut error = String::new();
            if missing_hardening_assets.len() == asset_descriptors.len() {
                error.push_str("Unhardened assets: all");
            } else {
                error.push_str("Unhardened assets:");
                for asset in &missing_hardening_assets {
                    error.push_str(&format!("\n  - {}", asset.key));
                }
            }
            return Err(GatherAssetDescriptorsError::HardenedSecurityPolicyIsNotHardened(error));
        }
    }
    Ok(asset_descriptors.into_values().collect())
}

#[cfg(test)]
mod test_gathering_asset_descriptors_with_tempdir {

    use crate::asset::config::{CacheConfig, HeadersConfig};

    use super::AssetDescriptor;
    use std::{
        collections::HashMap,
        fs,
        path::{Path, PathBuf},
    };
    use tempfile::{Builder, TempDir};

    fn gather_asset_descriptors(dirs: &[&Path]) -> Vec<AssetDescriptor> {
        let logger = slog::Logger::root(slog::Discard, slog::o!());
        super::gather_asset_descriptors(dirs, &logger).unwrap()
    }

    impl AssetDescriptor {
        fn default_from_path(assets_dir: &Path, relative_path: &str) -> Self {
            let relative_path = relative_path.split('/').collect::<Vec<_>>();
            let relative_path = relative_path
                .iter()
                .fold(PathBuf::new(), |acc, x| acc.join(x));
            AssetDescriptor {
                source: assets_dir.join(&relative_path),
                key: format!("/{}", relative_path.to_str().unwrap()),
                config: Default::default(),
            }
        }
        fn with_headers(mut self, headers: HashMap<&str, &str>) -> Self {
            let headers = headers
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<HeadersConfig>();
            let mut h = self.config.headers.unwrap_or_default();
            h.extend(headers);
            self.config.headers = Some(h);
            self
        }
        fn with_cache(mut self, cache: CacheConfig) -> Self {
            self.config.cache = Some(cache);
            self
        }
    }

    impl PartialEq for AssetDescriptor {
        fn eq(&self, other: &Self) -> bool {
            [
                self.source == other.source,
                self.key == other.key,
                self.config.cache == other.config.cache,
                self.config.headers == other.config.headers,
                self.config.ignore.unwrap_or(false) == other.config.ignore.unwrap_or(false),
            ]
            .into_iter()
            .all(|v| v)
        }
    }
    /// assets_tempdir directory structure:
    /// /assetsRAND5
    /// ├── .ic-assets.json
    /// ├── .hfile
    /// ├── file
    /// ├─- .hidden-dir
    /// │  ├── .ic-assets.json
    /// │  ├── .hfile
    /// │  ├── file
    /// │  └── .hidden-dir-nested
    /// │     ├── .ic-assets.json
    /// │     ├── .hfile
    /// │     └── file
    /// └── .hidden-dir-flat
    ///    ├── .ic-assets.json
    ///    ├── .hfile
    ///    └── file
    fn create_temporary_assets_directory(modified_files: HashMap<PathBuf, String>) -> TempDir {
        let assets_tempdir = Builder::new()
            .prefix("assets")
            .rand_bytes(5)
            .tempdir()
            .unwrap();

        let mut default_files = HashMap::from([
            (Path::new(".ic-assets.json").to_path_buf(), "[]".to_string()),
            (Path::new(".hfile").to_path_buf(), "".to_string()),
            (Path::new("file").to_path_buf(), "".to_string()),
            (
                Path::new(".hidden-dir/.ic-assets.json").to_path_buf(),
                "[]".to_string(),
            ),
            (
                Path::new(".hidden-dir/.hfile").to_path_buf(),
                "".to_string(),
            ),
            (Path::new(".hidden-dir/file").to_path_buf(), "".to_string()),
            (
                Path::new(".hidden-dir/.hidden-dir-nested/.ic-assets.json").to_path_buf(),
                "[]".to_string(),
            ),
            (
                Path::new(".hidden-dir/.hidden-dir-nested/.hfile").to_path_buf(),
                "".to_string(),
            ),
            (
                Path::new(".hidden-dir/.hidden-dir-nested/file").to_path_buf(),
                "".to_string(),
            ),
            (
                Path::new(".hidden-dir-flat/.ic-assets.json").to_path_buf(),
                "[]".to_string(),
            ),
            (
                Path::new(".hidden-dir-flat/.hfile").to_path_buf(),
                "".to_string(),
            ),
            (
                Path::new(".hidden-dir-flat/file").to_path_buf(),
                "".to_string(),
            ),
        ]);
        default_files.extend(modified_files);

        for (k, v) in default_files {
            let path = assets_tempdir.path().join(k);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, v).unwrap();
        }

        assets_tempdir
    }

    #[test]
    /// test gathering all files (including dotfiles in nested dotdirs)
    fn gather_all_files() {
        let files = HashMap::from([(
            Path::new(".ic-assets.json").to_path_buf(),
            r#"[
                {"match": ".*", "ignore": false}
            ]"#
            .to_string(),
        )]);

        let assets_temp_dir = create_temporary_assets_directory(files);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut asset_descriptors = dbg!(gather_asset_descriptors(&[&assets_dir]));

        let mut expected_asset_descriptors = vec![
            AssetDescriptor::default_from_path(&assets_dir, ".hfile"),
            AssetDescriptor::default_from_path(&assets_dir, "file"),
            AssetDescriptor::default_from_path(
                &assets_dir,
                ".hidden-dir/.hidden-dir-nested/.hfile",
            ),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir/.hidden-dir-nested/file"),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir-flat/.hfile"),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir-flat/file"),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir/.hfile"),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir/file"),
        ];

        expected_asset_descriptors.sort_by_key(|v| v.source.clone());
        asset_descriptors.sort_by_key(|v| v.source.clone());
        assert_eq!(asset_descriptors, expected_asset_descriptors);
    }

    #[test]
    /// test gathering all non-dot files, from non-dot dirs
    fn gather_all_nondot_files_from_nondot_dirs() {
        let files = HashMap::from([(
            Path::new(".ic-assets.json").to_path_buf(),
            r#"[
                    {"match": ".*", "ignore": true}
                ]"#
            .to_string(),
        )]);
        let assets_temp_dir = create_temporary_assets_directory(files);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let asset_descriptors = gather_asset_descriptors(&[&assets_dir]);
        let expected_asset_descriptors =
            vec![AssetDescriptor::default_from_path(&assets_dir, "file")];
        assert_eq!(asset_descriptors, expected_asset_descriptors);

        // same but without the `ignore` flag (defaults to `true`)
        let files = HashMap::from([(
            Path::new(".ic-assets.json").to_path_buf(),
            r#"[
                    {"match": ".*"}
                ]"#
            .to_string(),
        )]);
        let assets_temp_dir = create_temporary_assets_directory(files);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let asset_descriptors = gather_asset_descriptors(&[&assets_dir]);
        let expected_asset_descriptors =
            vec![AssetDescriptor::default_from_path(&assets_dir, "file")];
        assert_eq!(asset_descriptors, expected_asset_descriptors);

        // different glob pattern
        let files = HashMap::from([(
            Path::new(".ic-assets.json").to_path_buf(),
            r#"[
                    {"match": "*"}
                ]"#
            .to_string(),
        )]);
        let assets_temp_dir = create_temporary_assets_directory(files);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let asset_descriptors = gather_asset_descriptors(&[&assets_dir]);
        let expected_asset_descriptors =
            vec![AssetDescriptor::default_from_path(&assets_dir, "file")];
        assert_eq!(asset_descriptors, expected_asset_descriptors);

        // different glob pattern
        let files = HashMap::from([(
            Path::new(".ic-assets.json").to_path_buf(),
            r#"[
                    {"match": "**/*"}
                ]"#
            .to_string(),
        )]);
        let assets_temp_dir = create_temporary_assets_directory(files);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let asset_descriptors = gather_asset_descriptors(&[&assets_dir]);
        let expected_asset_descriptors =
            vec![AssetDescriptor::default_from_path(&assets_dir, "file")];
        assert_eq!(asset_descriptors, expected_asset_descriptors);
    }

    #[cfg(target_family = "unix")]
    #[test]
    /// Cannot include files inside hidden directory using only config file
    /// inside hidden directory. Hidden directory has to be first included in
    /// config file sitting in parent dir.
    /// The behaviour will have to stay until this lands:
    /// https://github.com/BurntSushi/ripgrep/issues/2229
    fn failed_to_include_hidden_dir() {
        let files = HashMap::from([(
            Path::new(".hidden-dir/.ic-assets.json").to_path_buf(),
            r#"[
                    {"match": ".", "ignore": false},
                    {"match": "?", "ignore": false},
                    {"match": "*", "ignore": false},
                    {"match": "**", "ignore": false},
                    {"match": ".?", "ignore": false},
                    {"match": ".*", "ignore": false},
                    {"match": ".**", "ignore": false},
                    {"match": "./*", "ignore": false},
                    {"match": "./**", "ignore": false},
                    {"match": "./**/*", "ignore": false},
                    {"match": "./**/**", "ignore": false},
                    {"match": "../*", "ignore": false},
                    {"match": "../.*", "ignore": false},
                    {"match": "../.**", "ignore": false},
                    {"match": "../.**/*", "ignore": false},
                    {"match": ".hfile", "ignore": false},
                    {"match": "file", "ignore": false},
                    {"match": "file"}
                ]"#
            .to_string(),
        )]);

        let assets_temp_dir = create_temporary_assets_directory(files);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut asset_descriptors = dbg!(gather_asset_descriptors(&[&assets_dir]));

        let mut expected_asset_descriptors =
            vec![AssetDescriptor::default_from_path(&assets_dir, "file")];

        expected_asset_descriptors.sort_by_key(|v| v.key.clone());
        asset_descriptors.sort_by_key(|v| v.key.clone());

        assert_eq!(asset_descriptors, expected_asset_descriptors)
    }

    #[test]
    fn configuring_dotfiles_step_by_step() {
        let files = HashMap::from([
            (
                Path::new(".ic-assets.json").to_path_buf(),
                r#"[{"match": ".hidden-dir", "ignore": false}]"#.to_string(),
            ),
            (
                Path::new(".hidden-dir/.ic-assets.json").to_path_buf(),
                r#"[
                    {"match": ".hidden-dir-nested", "ignore": false},
                    {"match": ".*", "ignore": false, "headers": {"A": "z"}},
                    {"match": ".hfile", "headers": {"B": "y"}}
                ]"#
                .to_string(),
            ),
            (
                Path::new(".hidden-dir/.hidden-dir-nested/.ic-assets.json").to_path_buf(),
                r#"[
                    {"match": "*", "ignore": false, "headers": {"C": "x"}},
                    {"match": ".hfile", "headers": {"D": "w"}}
                ]"#
                .to_string(),
            ),
        ]);

        let assets_temp_dir = create_temporary_assets_directory(files);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut asset_descriptors = dbg!(gather_asset_descriptors(&[&assets_dir]));

        let mut expected_asset_descriptors = vec![
            AssetDescriptor::default_from_path(&assets_dir, "file"),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir/.hfile")
                .with_headers(HashMap::from([("B", "y"), ("A", "z")])),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir/file"),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir/.hidden-dir-nested/file")
                .with_headers(HashMap::from([("A", "z"), ("C", "x")])),
            AssetDescriptor::default_from_path(
                &assets_dir,
                ".hidden-dir/.hidden-dir-nested/.hfile",
            )
            .with_headers(HashMap::from([("D", "w"), ("A", "z"), ("C", "x")])),
        ];

        expected_asset_descriptors.sort_by_key(|v| v.source.clone());
        asset_descriptors.sort_by_key(|v| v.source.clone());
        assert_eq!(asset_descriptors, expected_asset_descriptors)
    }

    #[test]
    fn include_only_a_specific_dotfile() {
        let files = HashMap::from([
            (
                Path::new(".ic-assets.json").to_path_buf(),
                r#"[
                    {"match": ".hidden-dir", "ignore": false},
                    {"match": "file", "ignore": true}
                ]"#
                .to_string(),
            ),
            (
                Path::new(".hidden-dir/.ic-assets.json").to_path_buf(),
                r#"[
                    {"match": "file", "ignore": true},
                    {"match": ".hidden-dir-nested", "ignore": false}
                ]"#
                .to_string(),
            ),
            (
                Path::new(".hidden-dir/.hidden-dir-nested/.ic-assets.json").to_path_buf(),
                r#"[
                    {"match": "file", "ignore": true},
                    {"match": ".hfile", "ignore": false, "headers": {"D": "w"}}
                ]"#
                .to_string(),
            ),
        ]);

        let assets_temp_dir = create_temporary_assets_directory(files);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut asset_descriptors = dbg!(gather_asset_descriptors(&[&assets_dir]));

        let mut expected_asset_descriptors = vec![AssetDescriptor::default_from_path(
            &assets_dir,
            ".hidden-dir/.hidden-dir-nested/.hfile",
        )
        .with_headers(HashMap::from([("D", "w")]))];

        expected_asset_descriptors.sort_by_key(|v| v.source.clone());
        asset_descriptors.sort_by_key(|v| v.source.clone());
        assert_eq!(asset_descriptors, expected_asset_descriptors);
    }

    #[test]
    fn include_all_files_except_one() {
        let files = HashMap::from([
            (
                Path::new(".ic-assets.json").to_path_buf(),
                r#"[
                    {"match": ".*", "ignore": false}
                ]"#
                .to_string(),
            ),
            (
                Path::new(".hidden-dir/.ic-assets.json").to_path_buf(),
                r#"[
                    {"match": "file", "ignore": true}
                ]"#
                .to_string(),
            ),
        ]);

        let assets_temp_dir = create_temporary_assets_directory(files);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut asset_descriptors = dbg!(gather_asset_descriptors(&[&assets_dir]));

        let mut expected_asset_descriptors = vec![
            AssetDescriptor::default_from_path(&assets_dir, "file"),
            AssetDescriptor::default_from_path(&assets_dir, ".hfile"),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir-flat/file"),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir-flat/.hfile"),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir/.hfile"),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir/.hidden-dir-nested/file"),
            AssetDescriptor::default_from_path(
                &assets_dir,
                ".hidden-dir/.hidden-dir-nested/.hfile",
            ),
        ];

        expected_asset_descriptors.sort_by_key(|v| v.source.clone());
        asset_descriptors.sort_by_key(|v| v.source.clone());
        assert_eq!(asset_descriptors, expected_asset_descriptors);
    }

    #[test]
    fn possible_to_reinclude_previously_ignored_file() {
        let files = HashMap::from([
            (
                Path::new(".ic-assets.json").to_path_buf(),
                r#"[
                    {"match": ".hidden-dir-flat", "ignore": false},
                    {"match": ".hidden-dir-flat/file", "ignore": true }

                ]"#
                .to_string(),
            ),
            (
                Path::new(".hidden-dir-flat/.ic-assets.json").to_path_buf(),
                r#"[
                    {"match": "*", "ignore": false},
                    {"match": "file", "ignore": false}
                ]"#
                .to_string(),
            ),
        ]);

        let assets_temp_dir = create_temporary_assets_directory(files);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut asset_descriptors = dbg!(gather_asset_descriptors(&[&assets_dir]));

        let mut expected_asset_descriptors = vec![
            AssetDescriptor::default_from_path(&assets_dir, "file"),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir-flat/.hfile"),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir-flat/file"),
        ];

        expected_asset_descriptors.sort_by_key(|v| v.source.clone());
        asset_descriptors.sort_by_key(|v| v.source.clone());
        assert_eq!(asset_descriptors, expected_asset_descriptors);
    }

    #[test]
    /// It is not possible to include a file if its parent directory has been excluded
    fn impossible_to_reinclude_file_from_already_ignored_directory() {
        let files = HashMap::from([
            // additional, non-dot dirs and files
            (Path::new("dir/file").to_path_buf(), "".to_string()),
            (Path::new("anotherdir/file").to_path_buf(), "".to_string()),
            (
                Path::new("anotherdir/.ic-assets.json").to_path_buf(),
                r#"[
                    {"match": "file", "ignore": false}
                ]"#
                .to_string(),
            ),
            // end of additional, non-dot dirs and files
            (
                Path::new(".ic-assets.json").to_path_buf(),
                r#"[
                    {"match": "anotherdir", "ignore": true}
                ]"#
                .to_string(),
            ),
        ]);

        let assets_temp_dir = create_temporary_assets_directory(files);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut asset_descriptors = dbg!(gather_asset_descriptors(&[&assets_dir]));

        let mut expected_asset_descriptors = vec![
            AssetDescriptor::default_from_path(&assets_dir, "file"),
            AssetDescriptor::default_from_path(&assets_dir, "dir/file"),
        ];

        expected_asset_descriptors.sort_by_key(|v| v.source.clone());
        asset_descriptors.sort_by_key(|v| v.source.clone());
        assert_eq!(asset_descriptors, expected_asset_descriptors);
    }

    #[test]
    fn known_directories_included_by_default() {
        let files = HashMap::from([
            // a typical use case of the .well-known folder
            (
                Path::new(".well-known/ic-domains").to_path_buf(),
                "foo.bar.com".to_string(),
            ),
        ]);

        let assets_temp_dir = create_temporary_assets_directory(files);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut asset_descriptors = dbg!(gather_asset_descriptors(&[&assets_dir]));

        let mut expected_asset_descriptors = vec![
            AssetDescriptor::default_from_path(&assets_dir, "file"),
            AssetDescriptor::default_from_path(&assets_dir, ".well-known/ic-domains"),
        ];

        expected_asset_descriptors.sort_by_key(|v| v.source.clone());
        asset_descriptors.sort_by_key(|v| v.source.clone());
        assert_eq!(asset_descriptors, expected_asset_descriptors);
    }

    #[test]
    fn known_directories_can_be_ignored() {
        let files = HashMap::from([
            // a typical use case of the .well-known folder
            (
                Path::new(".well-known/ic-domains").to_path_buf(),
                "foo.bar.com".to_string(),
            ),
            (
                Path::new(".ic-assets.json").to_path_buf(),
                r#"[
                    {"match": ".well-known", "ignore": true}
                ]"#
                .to_string(),
            ),
        ]);

        let assets_temp_dir = create_temporary_assets_directory(files);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut asset_descriptors = dbg!(gather_asset_descriptors(&[&assets_dir]));

        let mut expected_asset_descriptors =
            vec![AssetDescriptor::default_from_path(&assets_dir, "file")];

        expected_asset_descriptors.sort_by_key(|v| v.source.clone());
        asset_descriptors.sort_by_key(|v| v.source.clone());
        assert_eq!(asset_descriptors, expected_asset_descriptors);
    }

    #[test]
    fn bonanza() {
        let files = HashMap::from([
            // additional, non-dot dirs and files
            (Path::new("dir/file").to_path_buf(), "".to_string()),
            (
                Path::new("dir/.ic-assets.json").to_path_buf(),
                r#"[
                    {"match": "file", "headers": { "Access-Control-Allow-Origin": "null" }}
                ]"#
                .to_string(),
            ),
            (Path::new("anotherdir/file").to_path_buf(), "".to_string()),
            (
                Path::new("anotherdir/.ic-assets.json").to_path_buf(),
                r#"[
                    {"match": "file", "cache": { "max_age": 42 }, "headers": null }
                ]"#
                .to_string(),
            ),
            // end of additional, non-dot dirs and files
            (
                Path::new(".ic-assets.json").to_path_buf(),
                r#"[
                    {"match": "*", "cache": { "max_age": 11 }, "headers": { "X-Content-Type-Options": "nosniff" } },
                    {"match": "**/.hfile", "ignore": false, "headers": { "X-Content-Type-Options": "*" }},
                    {"match": ".hidden-dir-flat", "ignore": false },
                    {"match": ".hidden-dir", "ignore": false }

                ]"#
                .to_string(),
            ),
            (
                Path::new(".hidden-dir-flat/.ic-assets.json").to_path_buf(),
                r#"[
                    {"match": "*", "ignore": false, "headers": {"Cross-Origin-Resource-Policy": "same-origin"}},
                    {"match": ".hfile", "ignore": true}
                ]"#
                .to_string(),
            ),
        ]);

        let assets_temp_dir = create_temporary_assets_directory(files);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut asset_descriptors = gather_asset_descriptors(&[&assets_dir]);

        let mut expected_asset_descriptors = vec![
            AssetDescriptor::default_from_path(&assets_dir, ".hfile")
                .with_headers(HashMap::from([("X-Content-Type-Options", "*")]))
                .with_cache(CacheConfig { max_age: Some(11) }),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir/.hfile")
                .with_headers(HashMap::from([("X-Content-Type-Options", "*")]))
                .with_cache(CacheConfig { max_age: Some(11) }),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir/file")
                .with_headers(HashMap::from([("X-Content-Type-Options", "nosniff")]))
                .with_cache(CacheConfig { max_age: Some(11) }),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir-flat/file")
                .with_headers(HashMap::from([("X-Content-Type-Options", "nosniff")]))
                .with_headers(HashMap::from([(
                    "Cross-Origin-Resource-Policy",
                    "same-origin",
                )]))
                .with_cache(CacheConfig { max_age: Some(11) }),
            AssetDescriptor::default_from_path(&assets_dir, "anotherdir/file")
                .with_cache(CacheConfig { max_age: Some(42) }),
            AssetDescriptor::default_from_path(&assets_dir, "dir/file")
                .with_headers(HashMap::from([("X-Content-Type-Options", "nosniff")]))
                .with_headers(HashMap::from([("Access-Control-Allow-Origin", "null")]))
                .with_cache(CacheConfig { max_age: Some(11) }),
            AssetDescriptor::default_from_path(&assets_dir, "file")
                .with_cache(CacheConfig { max_age: Some(11) })
                .with_headers(HashMap::from([("X-Content-Type-Options", "nosniff")])),
        ];

        expected_asset_descriptors.sort_by_key(|v| v.source.clone());
        asset_descriptors.sort_by_key(|v| v.source.clone());
        assert_eq!(dbg!(asset_descriptors), expected_asset_descriptors);
    }
}
