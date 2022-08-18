use crate::asset_canister::batch::{commit_batch, create_batch};
use crate::asset_canister::list::list_assets;
use crate::asset_canister::protocol::{AssetDetails, BatchOperationKind};
use crate::asset_config::{AssetConfig, AssetSourceDirectoryConfiguration, ASSETS_CONFIG_FILENAME};
use crate::params::CanisterCallParams;

use crate::operations::{
    create_new_assets, delete_obsolete_assets, set_encodings, unset_obsolete_encodings,
};
use crate::plumbing::{make_project_assets, AssetDescriptor, ProjectAsset};
use anyhow::{bail, Context};
use ic_utils::Canister;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use walkdir::WalkDir;

/// Sets the contents of the asset canister to the contents of a directory, including deleting old assets.
pub async fn sync(
    canister: &Canister<'_>,
    dirs: &[&Path],
    timeout: Duration,
) -> anyhow::Result<()> {
    let asset_descriptors = gather_asset_descriptors(dirs)?;

    let canister_call_params = CanisterCallParams { canister, timeout };

    let container_assets = list_assets(&canister_call_params).await?;

    println!("Starting batch.");

    let batch_id = create_batch(&canister_call_params).await?;

    println!("Staging contents of new and changed assets:");

    let project_assets = make_project_assets(
        &canister_call_params,
        &batch_id,
        asset_descriptors,
        &container_assets,
    )
    .await?;

    let operations = assemble_synchronization_operations(project_assets, container_assets);

    println!("Committing batch.");
    commit_batch(&canister_call_params, &batch_id, operations).await?;

    Ok(())
}

fn include_entry(entry: &walkdir::DirEntry, config: &AssetConfig) -> bool {
    let starts_with_a_dot = entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false);

    match (starts_with_a_dot, config.ignore) {
        (dot, None) => !dot,
        (_dot, Some(ignored)) => !ignored,
    }
}

fn gather_asset_descriptors(dirs: &[&Path]) -> anyhow::Result<Vec<AssetDescriptor>> {
    let mut asset_descriptors: HashMap<String, AssetDescriptor> = HashMap::new();
    for dir in dirs {
        let dir = dir.canonicalize().with_context(|| {
            format!(
                "unable to canonicalize the following path: {}",
                dir.display()
            )
        })?;
        let configuration = AssetSourceDirectoryConfiguration::load(&dir)?;
        let mut asset_descriptors_interim = vec![];
        for e in WalkDir::new(&dir)
            .into_iter()
            .filter_entry(|entry| {
                if let Ok(canonical_path) = &entry.path().canonicalize() {
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
                entry.file_type().is_file() && entry.file_name() != ASSETS_CONFIG_FILENAME
            })
        {
            let source = e.path().canonicalize().with_context(|| {
                format!(
                    "unable to canonicalize the path when gathering asset descriptors: {}",
                    dir.display()
                )
            })?;
            let relative = source.strip_prefix(&dir).expect("cannot strip prefix");
            let key = String::from("/") + relative.to_string_lossy().as_ref();
            let config = configuration.get_asset_config(&source).context(format!(
                "failed to get config for asset: {}",
                source.display()
            ))?;

            asset_descriptors_interim.push(AssetDescriptor {
                source,
                key,
                config,
            })
        }

        for asset_descriptor in asset_descriptors_interim {
            if let Some(already_seen) = asset_descriptors.get(&asset_descriptor.key) {
                bail!(
                    "Asset with key '{}' defined at {} and {}",
                    &asset_descriptor.key,
                    asset_descriptor.source.display(),
                    already_seen.source.display()
                )
            }
            asset_descriptors.insert(asset_descriptor.key.clone(), asset_descriptor);
        }
    }
    Ok(asset_descriptors.into_values().collect())
}

fn assemble_synchronization_operations(
    project_assets: HashMap<String, ProjectAsset>,
    container_assets: HashMap<String, AssetDetails>,
) -> Vec<BatchOperationKind> {
    let mut container_assets = container_assets;

    let mut operations = vec![];

    delete_obsolete_assets(&mut operations, &project_assets, &mut container_assets);
    create_new_assets(&mut operations, &project_assets, &container_assets);
    unset_obsolete_encodings(&mut operations, &project_assets, &container_assets);
    set_encodings(&mut operations, project_assets);

    operations
}

#[cfg(test)]
mod test_gathering_asset_descriptors_with_tempdir {

    use crate::asset_config::{CacheConfig, HeadersConfig};

    use super::{gather_asset_descriptors, AssetDescriptor};
    use std::{
        collections::HashMap,
        fs,
        path::{Path, PathBuf},
    };
    use tempfile::{Builder, TempDir};

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
    fn create_temporary_assets_directory(
        modified_files: HashMap<PathBuf, String>,
    ) -> anyhow::Result<TempDir> {
        let assets_tempdir = Builder::new().prefix("assets").rand_bytes(5).tempdir()?;

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

        Ok(assets_tempdir)
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

        let assets_temp_dir = create_temporary_assets_directory(files).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut asset_descriptors = dbg!(gather_asset_descriptors(&[&assets_dir]).unwrap());

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
        let assets_temp_dir = create_temporary_assets_directory(files).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let asset_descriptors = gather_asset_descriptors(&[&assets_dir]).unwrap();
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
        let assets_temp_dir = create_temporary_assets_directory(files).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let asset_descriptors = gather_asset_descriptors(&[&assets_dir]).unwrap();
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
        let assets_temp_dir = create_temporary_assets_directory(files).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let asset_descriptors = gather_asset_descriptors(&[&assets_dir]).unwrap();
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
        let assets_temp_dir = create_temporary_assets_directory(files).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let asset_descriptors = gather_asset_descriptors(&[&assets_dir]).unwrap();
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

        let assets_temp_dir = create_temporary_assets_directory(files).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut asset_descriptors = dbg!(gather_asset_descriptors(&[&assets_dir]).unwrap());

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

        let assets_temp_dir = create_temporary_assets_directory(files).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut asset_descriptors = dbg!(gather_asset_descriptors(&[&assets_dir]).unwrap());

        let mut expected_asset_descriptors = vec![
            AssetDescriptor::default_from_path(&assets_dir, "file"),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir/.hfile")
                .with_headers(HashMap::from([("B", "\"y\""), ("A", "\"z\"")])),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir/file"),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir/.hidden-dir-nested/file")
                .with_headers(HashMap::from([("A", "\"z\""), ("C", "\"x\"")])),
            AssetDescriptor::default_from_path(
                &assets_dir,
                ".hidden-dir/.hidden-dir-nested/.hfile",
            )
            .with_headers(HashMap::from([
                ("D", "\"w\""),
                ("A", "\"z\""),
                ("C", "\"x\""),
            ])),
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

        let assets_temp_dir = create_temporary_assets_directory(files).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut asset_descriptors = dbg!(gather_asset_descriptors(&[&assets_dir]).unwrap());

        let mut expected_asset_descriptors = vec![AssetDescriptor::default_from_path(
            &assets_dir,
            ".hidden-dir/.hidden-dir-nested/.hfile",
        )
        .with_headers(HashMap::from([("D", "\"w\"")]))];

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

        let assets_temp_dir = create_temporary_assets_directory(files).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut asset_descriptors = dbg!(gather_asset_descriptors(&[&assets_dir]).unwrap());

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

        let assets_temp_dir = create_temporary_assets_directory(files).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut asset_descriptors = dbg!(gather_asset_descriptors(&[&assets_dir]).unwrap());

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

        let assets_temp_dir = create_temporary_assets_directory(files).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut asset_descriptors = dbg!(gather_asset_descriptors(&[&assets_dir]).unwrap());

        let mut expected_asset_descriptors = vec![
            AssetDescriptor::default_from_path(&assets_dir, "file"),
            AssetDescriptor::default_from_path(&assets_dir, "dir/file"),
        ];

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

        let assets_temp_dir = create_temporary_assets_directory(files).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut asset_descriptors = gather_asset_descriptors(&[&assets_dir]).unwrap();

        let mut expected_asset_descriptors = vec![
            AssetDescriptor::default_from_path(&assets_dir, ".hfile")
                .with_headers(HashMap::from([("X-Content-Type-Options", "\"*\"")]))
                .with_cache(CacheConfig { max_age: Some(11) }),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir/.hfile")
                .with_headers(HashMap::from([("X-Content-Type-Options", "\"*\"")]))
                .with_cache(CacheConfig { max_age: Some(11) }),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir/file")
                .with_headers(HashMap::from([("X-Content-Type-Options", "\"nosniff\"")]))
                .with_cache(CacheConfig { max_age: Some(11) }),
            AssetDescriptor::default_from_path(&assets_dir, ".hidden-dir-flat/file")
                .with_headers(HashMap::from([("X-Content-Type-Options", "\"nosniff\"")]))
                .with_headers(HashMap::from([(
                    "Cross-Origin-Resource-Policy",
                    "\"same-origin\"",
                )]))
                .with_cache(CacheConfig { max_age: Some(11) }),
            AssetDescriptor::default_from_path(&assets_dir, "anotherdir/file")
                .with_cache(CacheConfig { max_age: Some(42) }),
            AssetDescriptor::default_from_path(&assets_dir, "dir/file")
                .with_headers(HashMap::from([("X-Content-Type-Options", "\"nosniff\"")]))
                .with_headers(HashMap::from([("Access-Control-Allow-Origin", "\"null\"")]))
                .with_cache(CacheConfig { max_age: Some(11) }),
            AssetDescriptor::default_from_path(&assets_dir, "file")
                .with_cache(CacheConfig { max_age: Some(11) })
                .with_headers(HashMap::from([("X-Content-Type-Options", "\"nosniff\"")])),
        ];

        expected_asset_descriptors.sort_by_key(|v| v.source.clone());
        asset_descriptors.sort_by_key(|v| v.source.clone());
        assert_eq!(dbg!(asset_descriptors), expected_asset_descriptors);
    }
}
