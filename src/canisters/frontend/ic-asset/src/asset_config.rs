use anyhow::{bail, Context};
use derivative::Derivative;
use globset::GlobMatcher;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

pub(crate) const ASSETS_CONFIG_FILENAME_JSON: &str = ".ic-assets.json";
pub(crate) const ASSETS_CONFIG_FILENAME_JSON5: &str = ".ic-assets.json5";

/// A final piece of metadata assigned to the asset
#[derive(Derivative, PartialEq, Eq, Serialize, Clone)]
#[derivative(Default)]
pub struct AssetConfig {
    pub(crate) cache: Option<CacheConfig>,
    pub(crate) headers: Option<HeadersConfig>,
    pub(crate) ignore: Option<bool>,
    pub(crate) enable_aliasing: Option<bool>,
    #[derivative(Default(value = "Some(false)"))]
    pub(crate) allow_raw_access: Option<bool>,
}

pub(crate) type HeadersConfig = HashMap<String, String>;

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct CacheConfig {
    pub(crate) max_age: Option<u64>,
}

fn default_raw_access() -> Option<bool> {
    Some(false)
}

/// A single configuration object, from `.ic-assets.json` config file
#[derive(Derivative, Clone, Serialize)]
#[derivative(Debug, PartialEq)]
pub struct AssetConfigRule {
    #[derivative(
        Debug(format_with = "rule_utils::glob_fmt"),
        PartialEq(compare_with = "rule_utils::glob_cmp")
    )]
    #[serde(serialize_with = "rule_utils::glob_serialize")]
    r#match: GlobMatcher,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache: Option<CacheConfig>,
    #[serde(
        serialize_with = "rule_utils::headers_serialize",
        skip_serializing_if = "Maybe::is_absent"
    )]
    headers: Maybe<HeadersConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ignore: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_aliasing: Option<bool>,
    #[serde(skip_serializing)]
    used: bool,
    /// Redirects the traffic from .raw.ic0.app domain to .ic0.app
    #[serde(skip_serializing_if = "Option::is_none")]
    allow_raw_access: Option<bool>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
enum Maybe<T> {
    Null,
    Absent,
    Value(T),
}

impl<T> Default for Maybe<T> {
    fn default() -> Self {
        Self::Absent
    }
}

impl AssetConfigRule {
    fn applies(&self, canonical_path: &Path) -> bool {
        // TODO: better dot files/dirs handling, awaiting upstream changes:
        // https://github.com/BurntSushi/ripgrep/issues/2229
        self.r#match.is_match(canonical_path)
    }
}

type ConfigNode = Arc<Mutex<AssetConfigTreeNode>>;
type ConfigMap = HashMap<PathBuf, ConfigNode>;

/// The main public interface for aggregating `.ic-assets.json` files
/// nested in directories. Each sub/directory will be represented
/// as `AssetConfigTreeNode`.
#[derive(Debug)]
pub struct AssetSourceDirectoryConfiguration {
    config_map: ConfigMap,
}

/// A directory or subdirectory with assets.
#[derive(Debug, Default)]
struct AssetConfigTreeNode {
    pub parent: Option<ConfigNode>,
    pub rules: Vec<AssetConfigRule>,
    pub origin: PathBuf,
}

impl AssetSourceDirectoryConfiguration {
    /// Constructs config tree for assets directory.
    pub fn load(root_dir: &Path) -> anyhow::Result<Self> {
        if !root_dir.has_root() {
            bail!("root_dir paramenter is expected to be canonical path")
        }
        let mut config_map = HashMap::new();
        AssetConfigTreeNode::load(None, root_dir, &mut config_map)?;

        Ok(Self { config_map })
    }

    /// Fetches the configuration for the asset.
    pub fn get_asset_config(&mut self, canonical_path: &Path) -> anyhow::Result<AssetConfig> {
        let parent_dir = canonical_path.parent().with_context(|| {
            format!(
                "unable to get the parent directory for asset path: {:?}",
                canonical_path
            )
        })?;
        Ok(self
            .config_map
            .get(parent_dir)
            .with_context(|| {
                format!(
                    "unable to find asset config for following path: {:?}",
                    parent_dir
                )
            })?
            .lock()
            .unwrap()
            .get_config(canonical_path))
    }

    /// Returns a collection of unused configuration objects from all `.ic-assets.json` files
    pub fn get_unused_configs(&self) -> HashMap<PathBuf, Vec<AssetConfigRule>> {
        let mut hm = HashMap::new();
        // aggregate
        for node in self.config_map.values() {
            let config_node = &node.lock().unwrap();

            let origin = config_node.origin.clone();

            for rule in config_node.rules.clone() {
                if !rule.used {
                    hm.entry(origin.clone())
                        .and_modify(|v: &mut Vec<AssetConfigRule>| v.push(rule.clone()))
                        .or_insert_with(|| vec![rule.clone()]);
                }
            }
        }
        // dedup and remove full path from "match" field
        for (path, rules) in hm.iter_mut() {
            rules.sort_by_key(|v| v.r#match.glob().to_string());
            rules.dedup();
            for mut rule in rules {
                let prefix_path = format!("{}/", path.display());
                let modified_glob = rule.r#match.glob().to_string();
                let original_glob = &modified_glob.strip_prefix(&prefix_path);
                if let Some(og) = original_glob {
                    let original_glob = globset::Glob::new(og).unwrap().compile_matcher();
                    rule.r#match = original_glob;
                }
            }
        }
        hm
    }
}

impl AssetConfigTreeNode {
    /// Constructs config tree for assets directory in a recursive fashion.
    fn load(parent: Option<ConfigNode>, dir: &Path, configs: &mut ConfigMap) -> anyhow::Result<()> {
        let config_path: Option<PathBuf>;
        match (
            dir.join(ASSETS_CONFIG_FILENAME_JSON).exists(),
            dir.join(ASSETS_CONFIG_FILENAME_JSON5).exists(),
        ) {
            (true, true) => {
                return Err(anyhow::anyhow!(
                    "both {} and {} files exist in the same directory (dir = {:?})",
                    ASSETS_CONFIG_FILENAME_JSON,
                    ASSETS_CONFIG_FILENAME_JSON5,
                    dir
                ))
            }
            (true, false) => config_path = Some(dir.join(ASSETS_CONFIG_FILENAME_JSON)),
            (false, true) => config_path = Some(dir.join(ASSETS_CONFIG_FILENAME_JSON5)),
            (false, false) => config_path = None,
        }
        let mut rules = vec![];
        if let Some(config_path) = config_path {
            let content = fs::read_to_string(&config_path).with_context(|| {
                format!("unable to read config file: {}", config_path.display())
            })?;
            let interim_rules: Vec<rule_utils::InterimAssetConfigRule> = json5::from_str(&content)
                .with_context(|| {
                    format!(
                        "malformed JSON asset config file: {}",
                        config_path.display()
                    )
                })?;
            for interim_rule in interim_rules {
                rules.push(AssetConfigRule::from_interim(interim_rule, dir)?);
            }
        }

        let parent_ref = match parent {
            Some(p) if rules.is_empty() => p,
            _ => Arc::new(Mutex::new(Self {
                parent,
                rules,
                origin: dir.to_path_buf(),
            })),
        };

        configs.insert(dir.to_path_buf(), parent_ref.clone());
        for f in std::fs::read_dir(dir)
            .with_context(|| format!("Unable to read directory {}", &dir.display()))?
            .filter_map(|x| x.ok())
            .filter(|x| x.file_type().map_or_else(|_e| false, |ft| ft.is_dir()))
        {
            Self::load(Some(parent_ref.clone()), &f.path(), configs)?;
        }
        Ok(())
    }

    /// Fetches asset config in a recursive fashion.
    /// Marks config rules as *used*, whenever the rule' glob patter matched querried file.
    fn get_config(&mut self, canonical_path: &Path) -> AssetConfig {
        let base_config = match &self.parent {
            Some(parent) => parent.clone().lock().unwrap().get_config(canonical_path),
            None => AssetConfig::default(),
        };
        self.rules
            .iter_mut()
            .filter(|rule| rule.applies(canonical_path))
            .fold(base_config, |acc, x| {
                x.used = true;
                acc.merge(x)
            })
    }
}

impl AssetConfig {
    fn merge(mut self, other: &AssetConfigRule) -> Self {
        if let Some(c) = &other.cache {
            self.cache = Some(c.to_owned());
        };
        match (self.headers.as_mut(), &other.headers) {
            (Some(sh), Maybe::Value(oh)) => sh.extend(oh.to_owned()),
            (None, Maybe::Value(oh)) => self.headers = Some(oh.to_owned()),
            (_, Maybe::Null) => self.headers = None,
            (_, Maybe::Absent) => (),
        };

        if other.ignore.is_some() {
            self.ignore = other.ignore;
        }

        if other.enable_aliasing.is_some() {
            self.enable_aliasing = other.enable_aliasing;
        }

        if other.allow_raw_access.is_some() {
            self.allow_raw_access = other.allow_raw_access;
        }
        self
    }
}

/// This module contains various utilities needed for serialization/deserialization
/// and pretty-printing of the `AssetConfigRule` data structure.
mod rule_utils {
    use super::{AssetConfig, AssetConfigRule, CacheConfig, HeadersConfig, Maybe};
    use anyhow::Context;
    use globset::{Glob, GlobMatcher};
    use serde::{Deserialize, Serializer};
    use serde_json::Value;
    use std::collections::HashMap;
    use std::fmt;
    use std::path::Path;

    pub(super) fn glob_cmp(a: &GlobMatcher, b: &GlobMatcher) -> bool {
        a.glob() == b.glob()
    }

    pub(super) fn glob_fmt(
        field: &GlobMatcher,
        formatter: &mut fmt::Formatter,
    ) -> Result<(), fmt::Error> {
        formatter.write_str(field.glob().glob())?;
        Ok(())
    }

    pub(super) fn glob_serialize<S>(bytes: &GlobMatcher, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&bytes.glob().to_string())
    }

    pub(super) fn headers_serialize<S>(
        headers: &super::Maybe<HeadersConfig>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        match headers {
            super::Maybe::Null => serializer.serialize_map(Some(0))?.end(),
            super::Maybe::Value(hm) => {
                let mut map = serializer.serialize_map(Some(hm.len()))?;
                for (k, v) in hm {
                    map.serialize_entry(k, &v)?;
                }
                map.end()
            }
            super::Maybe::Absent => unreachable!(), // this option is already skipped via `skip_serialization_with`
        }
    }

    fn headers_deserialize<'de, D>(deserializer: D) -> Result<Maybe<HeadersConfig>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match serde_json::value::Value::deserialize(deserializer)? {
            Value::Object(v) => Ok(Maybe::Value(
                v.into_iter()
                    .map(|(k, v)| (k, v.to_string().trim_matches('"').to_string()))
                    .collect::<HashMap<String, String>>(),
            )),
            Value::Null => Ok(Maybe::Null),
            _ => Err(serde::de::Error::custom(
                "wrong data format for field `headers` (only map or null are allowed)",
            )),
        }
    }

    impl<T> Maybe<T> {
        pub(super) fn is_absent(&self) -> bool {
            matches!(*self, Self::Absent)
        }
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub(super) struct InterimAssetConfigRule {
        r#match: String,
        cache: Option<CacheConfig>,
        #[serde(default, deserialize_with = "headers_deserialize")]
        headers: Maybe<HeadersConfig>,
        ignore: Option<bool>,
        enable_aliasing: Option<bool>,
        #[serde(default = "super::default_raw_access")]
        allow_raw_access: Option<bool>,
    }

    impl AssetConfigRule {
        pub(super) fn from_interim(
            InterimAssetConfigRule {
                r#match,
                cache,
                headers,
                ignore,
                enable_aliasing,
                allow_raw_access,
            }: InterimAssetConfigRule,
            config_file_parent_dir: &Path,
        ) -> anyhow::Result<Self> {
            let glob = Glob::new(
            config_file_parent_dir
                .join(&r#match)
                .to_str()
                .with_context(|| {
                    format!(
                        "cannot combine {} and {} into a string (to be later used as a glob pattern)",
                        config_file_parent_dir.display(),
                        r#match
                    )
                })?,
        )
        .with_context(|| format!("{} is not a valid glob pattern", r#match))?.compile_matcher();

            Ok(Self {
                r#match: glob,
                cache,
                headers,
                ignore,
                used: false,
                enable_aliasing,
                allow_raw_access,
            })
        }
    }

    impl std::fmt::Display for AssetConfig {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut s = String::new();

            if self.cache.is_some() || self.headers.is_some() {
                s.push('(');
                if self.cache.as_ref().map_or(false, |v| v.max_age.is_some()) {
                    s.push_str("with cache");
                }
                if let Some(ref headers) = self.headers {
                    if !headers.is_empty() {
                        if s.len() > 1 {
                            s.push_str(" and ");
                        } else {
                            s.push_str("with ");
                        }
                        s.push_str(headers.len().to_string().as_str());
                        if headers.len() == 1 {
                            s.push_str(" header");
                        } else {
                            s.push_str(" headers");
                        }
                    }
                }
                s.push(')');
            }

            write!(f, "{}", s)
        }
    }

    impl std::fmt::Debug for AssetConfig {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut s = String::new();

            if let Some(ref cache) = self.cache {
                if let Some(ref max_age) = cache.max_age {
                    s.push_str(&format!("  - HTTP cache max-age: {}\n", max_age));
                }
            }
            if let Some(aliasing) = self.enable_aliasing {
                s.push_str(&format!(
                    "  - URL path aliasing: {}\n",
                    if aliasing { "enabled" } else { "disabled" }
                ));
            }
            if let Some(ref headers) = self.headers {
                for (key, value) in headers {
                    s.push_str(&format!(
                        "  - HTTP Response header: {key}: {value}\n",
                        key = key,
                        value = value
                    ));
                }
            }

            write!(f, "{}", s)
        }
    }
}

#[cfg(test)]
mod with_tempdir {

    use super::*;
    use std::io::Write;
    #[cfg(target_family = "unix")]
    use std::os::unix::prelude::PermissionsExt;
    use std::{collections::BTreeMap, fs::File};
    use tempfile::{Builder, TempDir};

    fn create_temporary_assets_directory(
        config_files: Option<HashMap<String, String>>,
        assets_count: usize,
    ) -> anyhow::Result<TempDir> {
        let assets_dir = Builder::new().prefix("assets").rand_bytes(5).tempdir()?;

        let _subdirs = ["css", "js", "nested/deep"]
            .map(|d| assets_dir.as_ref().join(d))
            .map(std::fs::create_dir_all);

        [
            "index.html",
            "js/index.js",
            "js/index.map.js",
            "css/main.css",
            "css/stylish.css",
            "nested/the-thing.txt",
            "nested/deep/the-next-thing.toml",
        ]
        .iter()
        .map(|path| assets_dir.path().join(path))
        .take(assets_count)
        .for_each(|path| {
            File::create(path).unwrap();
        });

        let new_empty_config = |directory: &str| (directory.to_string(), "[]".to_string());
        let mut h = HashMap::from([
            new_empty_config(""),
            new_empty_config("css"),
            new_empty_config("js"),
            new_empty_config("nested"),
            new_empty_config("nested/deep"),
        ]);
        if let Some(cf) = config_files {
            h.extend(cf);
        }
        h.into_iter().for_each(|(dir, content)| {
            let path = assets_dir
                .path()
                .join(dir)
                .join(ASSETS_CONFIG_FILENAME_JSON);
            let mut file = File::create(path).unwrap();
            write!(file, "{}", content).unwrap();
        });

        Ok(assets_dir)
    }

    #[test]
    fn match_only_nested_files() -> anyhow::Result<()> {
        let cfg = HashMap::from([(
            "nested".to_string(),
            r#"[{"match": "*", "cache": {"max_age": 333}}]"#.to_string(),
        )]);
        let assets_temp_dir = create_temporary_assets_directory(Some(cfg), 7).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize()?;

        let mut assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir)?;
        for f in ["nested/the-thing.txt", "nested/deep/the-next-thing.toml"] {
            assert_eq!(
                assets_config.get_asset_config(assets_dir.join(f).as_path())?,
                AssetConfig {
                    cache: Some(CacheConfig { max_age: Some(333) }),
                    ..Default::default()
                }
            );
        }
        for f in [
            "index.html",
            "js/index.js",
            "js/index.map.js",
            "css/main.css",
            "css/stylish.css",
        ] {
            assert_eq!(
                assets_config.get_asset_config(assets_dir.join(f).as_path())?,
                AssetConfig::default()
            );
        }

        Ok(())
    }

    #[test]
    fn overriding_cache_rules() -> anyhow::Result<()> {
        let cfg = Some(HashMap::from([
            (
                "nested".to_string(),
                r#"[{"match": "*", "cache": {"max_age": 111}}]"#.to_string(),
            ),
            (
                "".to_string(),
                r#"[{"match": "*", "cache": {"max_age": 333}}]"#.to_string(),
            ),
        ]));
        let assets_temp_dir = create_temporary_assets_directory(cfg, 7).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize()?;

        let mut assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir)?;
        for f in ["nested/the-thing.txt", "nested/deep/the-next-thing.toml"] {
            assert_eq!(
                assets_config.get_asset_config(assets_dir.join(f).as_path())?,
                AssetConfig {
                    cache: Some(CacheConfig { max_age: Some(111) }),
                    ..Default::default()
                }
            );
        }
        for f in [
            "index.html",
            "js/index.js",
            "js/index.map.js",
            "css/main.css",
            "css/stylish.css",
        ] {
            assert_eq!(
                assets_config.get_asset_config(assets_dir.join(f).as_path())?,
                AssetConfig {
                    cache: Some(CacheConfig { max_age: Some(333) }),
                    ..Default::default()
                }
            );
        }

        Ok(())
    }

    #[test]
    fn overriding_headers() -> anyhow::Result<()> {
        use serde_json::Value::*;
        let cfg = Some(HashMap::from([(
            "".to_string(),
            r#"
    [
      {
        "match": "index.html",
        "cache": {
          "max_age": 22
        },
        "headers": {
          "Content-Security-Policy": "add",
          "x-frame-options": "NONE",
          "x-content-type-options": "nosniff"
        }
      },
      {
        "match": "*",
        "headers": {
          "Content-Security-Policy": "delete"
        }
      },
      {
        "match": "*",
        "headers": {
          "Some-Other-Policy": "add"
        }
      },
      {
        "match": "*",
        "cache": {
          "max_age": 88
        },
        "headers": {
          "x-xss-protection": 1,
          "x-frame-options": "SAMEORIGIN"
        }
      }
    ]
    "#
            .to_string(),
        )]));
        let assets_temp_dir = create_temporary_assets_directory(cfg, 1).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize()?;
        let mut assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir)?;
        let parsed_asset_config =
            assets_config.get_asset_config(assets_dir.join("index.html").as_path())?;
        let expected_asset_config = AssetConfig {
            cache: Some(CacheConfig { max_age: Some(88) }),
            headers: Some(HashMap::from([
                ("x-content-type-options".to_string(), "nosniff".to_string()),
                ("x-frame-options".to_string(), "SAMEORIGIN".to_string()),
                ("Some-Other-Policy".to_string(), "add".to_string()),
                ("Content-Security-Policy".to_string(), "delete".to_string()),
                (
                    "x-xss-protection".to_string(),
                    Number(serde_json::Number::from(1)).to_string(),
                ),
            ])),
            ..Default::default()
        };

        assert_eq!(parsed_asset_config.cache, expected_asset_config.cache);
        assert_eq!(
            parsed_asset_config
                .headers
                .unwrap()
                .iter()
                // keys are sorted
                .collect::<BTreeMap<_, _>>(),
            expected_asset_config
                .headers
                .unwrap()
                .iter()
                .collect::<BTreeMap<_, _>>(),
        );

        Ok(())
    }

    #[test]
    fn prioritization() -> anyhow::Result<()> {
        // 1. the most deeply nested config file takes precedens over the one in parent dir
        // 2. order of rules withing file matters - last rule in config file takes precedens over the first one
        let cfg = Some(HashMap::from([
            (
                "".to_string(),
                r#"[
        {"match": "**/*", "cache": {"max_age": 999}},
        {"match": "nested/**/*", "cache": {"max_age": 900}},
        {"match": "nested/deep/*", "cache": {"max_age": 800}},
        {"match": "nested/**/*.toml","cache": {"max_age": 700}}
    ]"#
                .to_string(),
            ),
            (
                "nested".to_string(),
                r#"[
        {"match": "the-thing.txt", "cache": {"max_age": 600}},
        {"match": "*.txt", "cache": {"max_age": 500}},
        {"match": "*", "cache": {"max_age": 400}}
    ]"#
                .to_string(),
            ),
            (
                "nested/deep".to_string(),
                r#"[
        {"match": "**/*", "cache": {"max_age": 300}},
        {"match": "*", "cache": {"max_age": 200}},
        {"match": "*.toml", "cache": {"max_age": 100}}
    ]"#
                .to_string(),
            ),
        ]));
        let assets_temp_dir = create_temporary_assets_directory(cfg, 7).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize()?;

        let mut assets_config = dbg!(AssetSourceDirectoryConfiguration::load(&assets_dir))?;
        for f in [
            "index.html",
            "js/index.js",
            "js/index.map.js",
            "css/main.css",
            "css/stylish.css",
        ] {
            assert_eq!(
                assets_config.get_asset_config(assets_dir.join(f).as_path())?,
                AssetConfig {
                    cache: Some(CacheConfig { max_age: Some(999) }),
                    ..Default::default()
                }
            );
        }

        assert_eq!(
            assets_config.get_asset_config(assets_dir.join("nested/the-thing.txt").as_path())?,
            AssetConfig {
                cache: Some(CacheConfig { max_age: Some(400) }),
                ..Default::default()
            },
        );
        assert_eq!(
            assets_config
                .get_asset_config(assets_dir.join("nested/deep/the-next-thing.toml").as_path())?,
            AssetConfig {
                cache: Some(CacheConfig { max_age: Some(100) }),
                ..Default::default()
            },
        );

        Ok(())
    }

    #[test]
    fn json5_config_file_with_comments() -> anyhow::Result<()> {
        let cfg = Some(HashMap::from([(
            "".to_string(),
            r#"[
// comment
  {
    "match": "*",
    /*
    look at this beatiful key below, not wrapped in quotes
*/  cache: { max_age: 999 } }
]"#
            .to_string(),
        )]));
        let assets_temp_dir = create_temporary_assets_directory(cfg, 0).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize()?;
        let mut assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir)?;
        assert_eq!(
            assets_config.get_asset_config(assets_dir.join("index.html").as_path())?,
            AssetConfig {
                cache: Some(CacheConfig { max_age: Some(999) }),
                ..Default::default()
            },
        );
        Ok(())
    }

    #[test]
    fn no_content_config_file() -> anyhow::Result<()> {
        let cfg = Some(HashMap::from([
            ("".to_string(), "".to_string()),
            ("css".to_string(), "".to_string()),
            ("js".to_string(), "".to_string()),
            ("nested".to_string(), "".to_string()),
            ("nested/deep".to_string(), "".to_string()),
        ]));
        let assets_temp_dir = create_temporary_assets_directory(cfg, 0).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize()?;
        let assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir);
        assert_eq!(
            assets_config.err().unwrap().to_string(),
            format!(
                "malformed JSON asset config file: {}",
                assets_dir
                    .join(ASSETS_CONFIG_FILENAME_JSON)
                    .to_str()
                    .unwrap()
            )
        );
        Ok(())
    }

    #[test]
    fn invalid_json_config_file() -> anyhow::Result<()> {
        let cfg = Some(HashMap::from([("".to_string(), "[[[{{{".to_string())]));
        let assets_temp_dir = create_temporary_assets_directory(cfg, 0).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize()?;
        let assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir);
        assert_eq!(
            assets_config.err().unwrap().to_string(),
            format!(
                "malformed JSON asset config file: {}",
                assets_dir
                    .join(ASSETS_CONFIG_FILENAME_JSON)
                    .to_str()
                    .unwrap()
            )
        );
        Ok(())
    }

    #[test]
    fn invalid_glob_pattern() -> anyhow::Result<()> {
        let cfg = Some(HashMap::from([(
            "".to_string(),
            r#"[
        {"match": "{{{\\\", "cache": {"max_age": 900}},
    ]"#
            .to_string(),
        )]));
        let assets_temp_dir = create_temporary_assets_directory(cfg, 0).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize()?;
        let assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir);
        assert_eq!(
            assets_config.err().unwrap().to_string(),
            format!(
                "malformed JSON asset config file: {}",
                assets_dir
                    .join(ASSETS_CONFIG_FILENAME_JSON)
                    .to_str()
                    .unwrap()
            )
        );
        Ok(())
    }

    #[test]
    fn invalid_asset_path() -> anyhow::Result<()> {
        let cfg = Some(HashMap::new());
        let assets_temp_dir = create_temporary_assets_directory(cfg, 0).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize()?;
        let mut assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir)?;
        assert_eq!(
            assets_config.get_asset_config(assets_dir.join("doesnt.exists").as_path())?,
            AssetConfig::default()
        );
        Ok(())
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn no_read_permission() -> anyhow::Result<()> {
        let cfg = Some(HashMap::from([(
            "".to_string(),
            r#"[
        {"match": "*", "cache": {"max_age": 20}}
    ]"#
            .to_string(),
        )]));
        let assets_temp_dir = create_temporary_assets_directory(cfg, 1).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize()?;
        std::fs::set_permissions(
            assets_dir.join(ASSETS_CONFIG_FILENAME_JSON).as_path(),
            std::fs::Permissions::from_mode(0o000),
        )
        .unwrap();

        let assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir);
        assert_eq!(
            assets_config.err().unwrap().to_string(),
            format!(
                "unable to read config file: {}",
                assets_dir
                    .join(ASSETS_CONFIG_FILENAME_JSON)
                    .as_path()
                    .to_str()
                    .unwrap()
            )
        );

        Ok(())
    }

    #[test]
    fn allow_raw_access_flag() -> anyhow::Result<()> {
        let cfg = Some(HashMap::from([(
            "".to_string(),
            r#"[
  {
    "match": "*",
    "allow_raw_access": true
  }
]"#
            .to_string(),
        )]));
        let assets_temp_dir = create_temporary_assets_directory(cfg, 0).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize()?;
        let mut assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir)?;
        assert_eq!(
            assets_config.get_asset_config(assets_dir.join("index.html").as_path())?,
            AssetConfig {
                allow_raw_access: Some(true),
                ..Default::default()
            },
        );
        Ok(())
    }

    #[test]
    fn default_value_for_allow_raw_access_flag() -> anyhow::Result<()> {
        let cfg = Some(HashMap::from([("".to_string(), "[]".to_string())]));
        let assets_temp_dir = create_temporary_assets_directory(cfg, 0).unwrap();
        let assets_dir = assets_temp_dir.path().canonicalize()?;
        let mut assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir)?;
        assert_eq!(
            assets_config.get_asset_config(assets_dir.join("index.html").as_path())?,
            AssetConfig {
                allow_raw_access: Some(false),
                ..Default::default()
            },
        );
        Ok(())
    }
}
