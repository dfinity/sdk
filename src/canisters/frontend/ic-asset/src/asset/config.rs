use crate::error::AssetLoadConfigError;
use crate::error::AssetLoadConfigError::{LoadRuleFailed, MalformedAssetConfigFile};
use crate::error::GetAssetConfigError;
use crate::error::GetAssetConfigError::AssetConfigNotFound;
use crate::security_policy::SecurityPolicy;
use derivative::Derivative;
use globset::GlobMatcher;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use super::content_encoder::ContentEncoder;

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
    #[derivative(Default(value = "Some(true)"))]
    pub(crate) allow_raw_access: Option<bool>,
    pub(crate) encodings: Option<Vec<ContentEncoder>>,
    pub(crate) security_policy: Option<SecurityPolicy>,
    pub(crate) disable_security_policy_warning: Option<bool>,
}

impl AssetConfig {
    pub fn combined_headers(&self) -> Option<HeadersConfig> {
        match (self.headers.as_ref(), self.security_policy) {
            (None, None) => None,
            (None, Some(policy)) => Some(policy.to_headers()),
            (Some(custom_headers), None) => Some(custom_headers.clone()),
            (Some(custom_headers), Some(policy)) => {
                let mut headers = custom_headers.clone();
                let custom_header_names: HashSet<String> =
                    HashSet::from_iter(custom_headers.keys().map(|a| a.to_lowercase()));
                for (policy_header_name, policy_header_value) in policy.to_headers() {
                    if !custom_header_names.contains(&policy_header_name.to_lowercase()) {
                        headers.insert(policy_header_name, policy_header_value);
                    }
                }
                Some(headers)
            }
        }
    }

    pub fn warn_about_standard_security_policy(&self) -> bool {
        let warning_disabled = self.disable_security_policy_warning == Some(true);
        let standard_policy = self.security_policy == Some(SecurityPolicy::Standard);
        standard_policy && !warning_disabled
    }

    pub fn warn_about_no_security_policy(&self) -> bool {
        let warning_disabled = self.disable_security_policy_warning == Some(true);
        let no_policy = self.security_policy.is_none();
        no_policy && !warning_disabled
    }

    /// If the security policy is `"hardened"` it is expected that some custom headers are present.
    /// This cannot be silenced with `disable_security_policy_warning`.
    pub fn warn_about_missing_hardening_headers(&self) -> bool {
        let is_hardened = self.security_policy == Some(SecurityPolicy::Hardened);
        let has_headers = self
            .headers
            .as_ref()
            .map(|headers| !headers.is_empty())
            .unwrap_or_default();
        is_hardened && !has_headers
    }
}

pub(crate) type HeadersConfig = BTreeMap<String, String>;

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct CacheConfig {
    pub(crate) max_age: Option<u64>,
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
    /// Redirects the traffic from .raw.icp0.io domain to .icp0.io
    #[serde(skip_serializing_if = "Option::is_none")]
    allow_raw_access: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    encodings: Option<Vec<ContentEncoder>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    security_policy: Option<SecurityPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disable_security_policy_warning: Option<bool>,
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
    pub fn load(root_dir: &Path) -> Result<Self, AssetLoadConfigError> {
        if !root_dir.has_root() {
            return Err(AssetLoadConfigError::InvalidRootDir(root_dir.to_path_buf()));
        }
        let mut config_map = HashMap::new();
        AssetConfigTreeNode::load(None, root_dir, &mut config_map)?;

        Ok(Self { config_map })
    }

    /// Fetches the configuration for the asset.
    pub fn get_asset_config(
        &mut self,
        canonical_path: &Path,
    ) -> Result<AssetConfig, GetAssetConfigError> {
        let parent_dir = dfx_core::fs::parent(canonical_path)?;
        Ok(self
            .config_map
            .get(&parent_dir)
            .ok_or_else(|| AssetConfigNotFound(parent_dir.to_path_buf()))?
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
            for rule in rules {
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
    fn load(
        parent: Option<ConfigNode>,
        dir: &Path,
        configs: &mut ConfigMap,
    ) -> Result<(), AssetLoadConfigError> {
        let config_path = match (
            dir.join(ASSETS_CONFIG_FILENAME_JSON).exists(),
            dir.join(ASSETS_CONFIG_FILENAME_JSON5).exists(),
        ) {
            (true, true) => {
                return Err(AssetLoadConfigError::MultipleConfigurationFiles(
                    dir.to_path_buf(),
                ));
            }
            (true, false) => Some(dir.join(ASSETS_CONFIG_FILENAME_JSON)),
            (false, true) => Some(dir.join(ASSETS_CONFIG_FILENAME_JSON5)),
            (false, false) => None,
        };
        let mut rules = vec![];
        if let Some(config_path) = config_path {
            let content = dfx_core::fs::read_to_string(&config_path)?;

            let interim_rules: Vec<rule_utils::InterimAssetConfigRule> = json5::from_str(&content)
                .map_err(|e| MalformedAssetConfigFile(config_path.to_path_buf(), e))?;
            for interim_rule in interim_rules {
                let rule = AssetConfigRule::from_interim(interim_rule, dir)
                    .map_err(|e| LoadRuleFailed(config_path.to_path_buf(), e))?;
                rules.push(rule);
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
        for f in dfx_core::fs::read_dir(dir)?
            .filter_map(|x| x.ok())
            .filter(|x| x.file_type().map_or_else(|_e| false, |ft| ft.is_dir()))
        {
            Self::load(Some(parent_ref.clone()), &f.path(), configs)?;
        }
        Ok(())
    }

    /// Fetches asset config in a recursive fashion.
    /// Marks config rules as *used*, whenever the rule' glob patter matched queried file.
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

        if other.encodings.is_some() {
            self.encodings = other.encodings.clone();
        }

        if other.security_policy.is_some() {
            self.security_policy = other.security_policy;
        }

        if other.disable_security_policy_warning.is_some() {
            self.disable_security_policy_warning = other.disable_security_policy_warning;
        }
        self
    }
}

/// This module contains various utilities needed for serialization/deserialization
/// and pretty-printing of the `AssetConfigRule` data structure.
mod rule_utils {
    use super::{AssetConfig, AssetConfigRule, CacheConfig, HeadersConfig, Maybe, SecurityPolicy};
    use crate::asset::content_encoder::ContentEncoder;
    use crate::error::LoadRuleError;
    use globset::{Glob, GlobMatcher};
    use itertools::Itertools;
    use serde::{Deserialize, Serializer};
    use serde_json::Value;
    use std::collections::BTreeMap;
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
                    .collect::<BTreeMap<String, String>>(),
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
        allow_raw_access: Option<bool>,
        encodings: Option<Vec<ContentEncoder>>,
        security_policy: Option<SecurityPolicy>,
        disable_security_policy_warning: Option<bool>,
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
                encodings,
                security_policy,
                disable_security_policy_warning,
            }: InterimAssetConfigRule,
            config_file_parent_dir: &Path,
        ) -> Result<Self, LoadRuleError> {
            let glob = config_file_parent_dir.join(&r#match);
            let glob = glob.to_str().ok_or_else(|| {
                LoadRuleError::FormGlobPatternFailed(
                    config_file_parent_dir.to_path_buf(),
                    r#match.clone(),
                )
            })?;
            let matcher = Glob::new(glob)
                .map_err(|e| LoadRuleError::InvalidGlobPattern(r#match, e))?
                .compile_matcher();

            Ok(Self {
                r#match: matcher,
                cache,
                headers,
                ignore,
                used: false,
                enable_aliasing,
                allow_raw_access,
                encodings,
                security_policy,
                disable_security_policy_warning,
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
                if let Some(encodings) = self.encodings.as_ref() {
                    s.push_str(&format!(", {} encodings", encodings.len()));
                }
                if let Some(policy) = self.security_policy {
                    s.push_str(&format!(" and security policy '{policy}'"));
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
            if let Some(allow_raw_access) = self.allow_raw_access {
                s.push_str(&format!(
                    "  - enable raw access: {}\n",
                    if allow_raw_access {
                        "enabled"
                    } else {
                        "disabled"
                    }
                ));
            }
            if let Some(policy) = self.security_policy {
                s.push_str(&format!("  - Security policy: {policy}"));
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
            if let Some(encodings) = self.encodings.as_ref() {
                s.push_str(&format!(
                    "  - encodings: {}",
                    encodings.iter().map(|enc| enc.to_string()).join(",")
                ));
            }
            if let Some(disable_warning) = self.disable_security_policy_warning {
                s.push_str(&format!(
                    "  - disable standard security policy warning: {disable_warning}"
                ));
            }
            write!(f, "{}", s)
        }
    }
}

#[cfg(test)]
mod with_tempdir {

    use super::*;
    #[cfg(target_family = "unix")]
    use std::error::Error;
    use std::io::Write;
    #[cfg(target_family = "unix")]
    use std::os::unix::prelude::PermissionsExt;
    use std::{collections::BTreeMap, fs::File};
    use tempfile::{Builder, TempDir};

    fn create_temporary_assets_directory(
        config_files: Option<HashMap<String, String>>,
        assets_count: usize,
    ) -> TempDir {
        let assets_dir = Builder::new()
            .prefix("assets")
            .rand_bytes(5)
            .tempdir()
            .unwrap();

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

        assets_dir
    }

    #[test]
    fn match_only_nested_files() {
        let cfg = HashMap::from([(
            "nested".to_string(),
            r#"[{"match": "*", "cache": {"max_age": 333}}]"#.to_string(),
        )]);
        let assets_temp_dir = create_temporary_assets_directory(Some(cfg), 7);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();

        let mut assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir).unwrap();
        for f in ["nested/the-thing.txt", "nested/deep/the-next-thing.toml"] {
            assert_eq!(
                assets_config
                    .get_asset_config(assets_dir.join(f).as_path())
                    .unwrap(),
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
                assets_config
                    .get_asset_config(assets_dir.join(f).as_path())
                    .unwrap(),
                AssetConfig::default()
            );
        }
    }

    #[test]
    fn overriding_cache_rules() {
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
        let assets_temp_dir = create_temporary_assets_directory(cfg, 7);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();

        let mut assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir).unwrap();
        for f in ["nested/the-thing.txt", "nested/deep/the-next-thing.toml"] {
            assert_eq!(
                assets_config
                    .get_asset_config(assets_dir.join(f).as_path())
                    .unwrap(),
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
                assets_config
                    .get_asset_config(assets_dir.join(f).as_path())
                    .unwrap(),
                AssetConfig {
                    cache: Some(CacheConfig { max_age: Some(333) }),
                    ..Default::default()
                }
            );
        }
    }

    #[test]
    fn overriding_headers() {
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
        let assets_temp_dir = create_temporary_assets_directory(cfg, 1);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir).unwrap();
        let parsed_asset_config = assets_config
            .get_asset_config(assets_dir.join("index.html").as_path())
            .unwrap();
        let expected_asset_config = AssetConfig {
            cache: Some(CacheConfig { max_age: Some(88) }),
            headers: Some(BTreeMap::from([
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
    }

    #[test]
    fn overriding_encodings() {
        let cfg = Some(HashMap::from([
            (
                "".to_string(),
                r#"[{"match": "**/*.txt", "encodings": []},{"match": "**/*.unknown", "encodings": ["gzip"]}]"#.to_string(),
            ),
        ]));
        let assets_temp_dir = create_temporary_assets_directory(cfg, 7);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();

        let mut assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir).unwrap();
        // override default (.unknown defaults to empty list)
        assert_eq!(
            assets_config
                .get_asset_config(assets_dir.join("file.unknown").as_path())
                .unwrap(),
            AssetConfig {
                encodings: Some(Vec::from([ContentEncoder::Gzip])),
                ..Default::default()
            }
        );
        // override default with empty list (.txt defaults to gzip)
        assert_eq!(
            assets_config
                .get_asset_config(assets_dir.join("text.txt").as_path())
                .unwrap(),
            AssetConfig {
                encodings: Some(Vec::from([])),
                ..Default::default()
            }
        );
    }

    #[test]
    fn prioritization() {
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
        let assets_temp_dir = create_temporary_assets_directory(cfg, 7);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();

        let mut assets_config = dbg!(AssetSourceDirectoryConfiguration::load(&assets_dir)).unwrap();
        for f in [
            "index.html",
            "js/index.js",
            "js/index.map.js",
            "css/main.css",
            "css/stylish.css",
        ] {
            assert_eq!(
                assets_config
                    .get_asset_config(assets_dir.join(f).as_path())
                    .unwrap(),
                AssetConfig {
                    cache: Some(CacheConfig { max_age: Some(999) }),
                    ..Default::default()
                }
            );
        }

        assert_eq!(
            assets_config
                .get_asset_config(assets_dir.join("nested/the-thing.txt").as_path())
                .unwrap(),
            AssetConfig {
                cache: Some(CacheConfig { max_age: Some(400) }),
                ..Default::default()
            },
        );
        assert_eq!(
            assets_config
                .get_asset_config(assets_dir.join("nested/deep/the-next-thing.toml").as_path())
                .unwrap(),
            AssetConfig {
                cache: Some(CacheConfig { max_age: Some(100) }),
                ..Default::default()
            },
        );
    }

    #[test]
    fn json5_config_file_with_comments() {
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
        let assets_temp_dir = create_temporary_assets_directory(cfg, 0);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir).unwrap();
        assert_eq!(
            assets_config
                .get_asset_config(assets_dir.join("index.html").as_path())
                .unwrap(),
            AssetConfig {
                cache: Some(CacheConfig { max_age: Some(999) }),
                ..Default::default()
            },
        );
    }

    #[test]
    fn no_content_config_file() {
        let cfg = Some(HashMap::from([
            ("".to_string(), "".to_string()),
            ("css".to_string(), "".to_string()),
            ("js".to_string(), "".to_string()),
            ("nested".to_string(), "".to_string()),
            ("nested/deep".to_string(), "".to_string()),
        ]));
        let assets_temp_dir = create_temporary_assets_directory(cfg, 0);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir);
        assert_eq!(
            assets_config.err().unwrap().to_string(),
            format!(
                "Malformed JSON asset config file '{}':  {}",
                assets_dir
                    .join(ASSETS_CONFIG_FILENAME_JSON)
                    .to_str()
                    .unwrap(),
                "--> 1:1\n  |\n1 | \n  | ^---\n  |\n  = expected array, boolean, null, number, object, or string"
            )
        );
    }

    #[test]
    fn invalid_json_config_file() {
        let cfg = Some(HashMap::from([("".to_string(), "[[[{{{".to_string())]));
        let assets_temp_dir = create_temporary_assets_directory(cfg, 0);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir);
        assert_eq!(
            assets_config.err().unwrap().to_string(),
            format!(
                "Malformed JSON asset config file '{}':  {}",
                assets_dir
                    .join(ASSETS_CONFIG_FILENAME_JSON)
                    .to_str()
                    .unwrap(),
                "--> 1:5\n  |\n1 | [[[{{{\n  |     ^---\n  |\n  = expected identifier or string"
            )
        );
    }

    #[test]
    fn invalid_glob_pattern() {
        let cfg = Some(HashMap::from([(
            "".to_string(),
            r#"[
        {"match": "{{{\\\", "cache": {"max_age": 900}},
    ]"#
            .to_string(),
        )]));
        let assets_temp_dir = create_temporary_assets_directory(cfg, 0);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir);
        assert_eq!(
            assets_config.err().unwrap().to_string(),
            format!(
                "Malformed JSON asset config file '{}':  {}",
                assets_dir
                    .join(ASSETS_CONFIG_FILENAME_JSON)
                    .to_str()
                    .unwrap(),
                "--> 2:19\n  |\n2 |         {\"match\": \"{{{\\\\\\\", \"cache\": {\"max_age\": 900}},\n  |                   ^---\n  |\n  = expected boolean or null"
            )
        );
    }

    #[test]
    fn invalid_asset_path() {
        let cfg = Some(HashMap::new());
        let assets_temp_dir = create_temporary_assets_directory(cfg, 0);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir).unwrap();
        assert_eq!(
            assets_config
                .get_asset_config(assets_dir.join("doesnt.exists").as_path())
                .unwrap(),
            AssetConfig::default()
        );
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn no_read_permission() {
        let cfg = Some(HashMap::from([(
            "".to_string(),
            r#"[
        {"match": "*", "cache": {"max_age": 20}}
    ]"#
            .to_string(),
        )]));
        let assets_temp_dir = create_temporary_assets_directory(cfg, 1);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        std::fs::set_permissions(
            assets_dir.join(ASSETS_CONFIG_FILENAME_JSON).as_path(),
            std::fs::Permissions::from_mode(0o000),
        )
        .unwrap();

        let assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir);
        assert_eq!(
            assets_config.as_ref().err().unwrap().to_string(),
            format!(
                "failed to read {} as string",
                assets_dir
                    .join(ASSETS_CONFIG_FILENAME_JSON)
                    .as_path()
                    .to_str()
                    .unwrap()
            )
        );
        assert_eq!(
            assets_config.err().unwrap().source().unwrap().to_string(),
            "Permission denied (os error 13)"
        );
    }

    #[test]
    fn allow_raw_access_flag() {
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
        let assets_temp_dir = create_temporary_assets_directory(cfg, 0);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir).unwrap();
        assert_eq!(
            assets_config
                .get_asset_config(assets_dir.join("index.html").as_path())
                .unwrap(),
            AssetConfig {
                allow_raw_access: Some(true),
                ..Default::default()
            },
        );
    }

    #[test]
    fn default_value_for_allow_raw_access_flag() {
        let cfg = Some(HashMap::from([("".to_string(), "[]".to_string())]));
        let assets_temp_dir = create_temporary_assets_directory(cfg, 0);
        let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
        let mut assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir).unwrap();
        assert_eq!(
            assets_config
                .get_asset_config(assets_dir.join("index.html").as_path())
                .unwrap(),
            AssetConfig {
                allow_raw_access: Some(true),
                ..Default::default()
            },
        );
    }

    #[test]
    fn the_order_does_not_matter() {
        let cfg = Some(HashMap::from([(
            "".to_string(),
            r#"[
                {
                    "match": "**/deep/**/*",
                    "allow_raw_access": false,
                    "cache": {
                      "max_age": 22
                    },
                    "enable_aliasing": true,
                    "ignore": true
                },
                {
                    "match": "**/*",
                    "headers": {
                        "X-Frame-Options": "DENY"
                    }
                }
            ]
"#
            .to_string(),
        )]));
        let cfg2 = Some(HashMap::from([(
            "".to_string(),
            r#"[
                {
                    "match": "**/*",
                    "headers": {
                        "X-Frame-Options": "DENY"
                    }
                },
                {
                    "match": "**/deep/**/*",
                    "allow_raw_access": false,
                    "cache": {
                      "max_age": 22
                    },
                    "enable_aliasing": true,
                    "ignore": true
                }
            ]
            "#
            .to_string(),
        )]));

        let x = {
            let assets_temp_dir = create_temporary_assets_directory(cfg, 0);
            let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
            let mut assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir).unwrap();
            assets_config
                .get_asset_config(assets_dir.join("nested/deep/the-next-thing.toml").as_path())
                .unwrap()
        };
        let y = {
            let assets_temp_dir = create_temporary_assets_directory(cfg2, 0);
            let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
            let mut assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir).unwrap();
            assets_config
                .get_asset_config(assets_dir.join("nested/deep/the-next-thing.toml").as_path())
                .unwrap()
        };

        dbg!(&x, &y);
        assert_eq!(x.allow_raw_access, Some(false));
        assert_eq!(y.allow_raw_access, Some(false));
        assert_eq!(x.enable_aliasing, Some(true));
        assert_eq!(y.enable_aliasing, Some(true));
        assert_eq!(x.ignore, Some(true));
        assert_eq!(y.ignore, Some(true));
        assert_eq!(x.cache.clone().unwrap().max_age, Some(22));
        assert_eq!(y.cache.clone().unwrap().max_age, Some(22));

        // same as above but with different values
        let cfg = Some(HashMap::from([(
            "".to_string(),
            r#"[
                {
                    "match": "**/deep/**/*",
                    "allow_raw_access": true,
                    "enable_aliasing": false,
                    "ignore": false,
                    "headers": {
                        "X-Frame-Options": "ALLOW"
                    }
                },
                {
                    "match": "**/*",
                    "cache": {
                      "max_age": 22
                    }
                }
            ]
"#
            .to_string(),
        )]));
        let cfg2 = Some(HashMap::from([(
            "".to_string(),
            r#"[
                {
                    "match": "**/*",
                    "cache": {
                      "max_age": 22
                    }
                },
                {
                    "match": "**/deep/**/*",
                    "allow_raw_access": true,
                    "enable_aliasing": false,
                    "ignore": false,
                    "headers": {
                        "X-Frame-Options": "ALLOW"
                    }
                }
            ]
            "#
            .to_string(),
        )]));

        let x = {
            let assets_temp_dir = create_temporary_assets_directory(cfg, 0);
            let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
            let mut assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir).unwrap();
            assets_config
                .get_asset_config(assets_dir.join("nested/deep/the-next-thing.toml").as_path())
                .unwrap()
        };
        let y = {
            let assets_temp_dir = create_temporary_assets_directory(cfg2, 0);
            let assets_dir = assets_temp_dir.path().canonicalize().unwrap();
            let mut assets_config = AssetSourceDirectoryConfiguration::load(&assets_dir).unwrap();
            assets_config
                .get_asset_config(assets_dir.join("nested/deep/the-next-thing.toml").as_path())
                .unwrap()
        };

        dbg!(&x, &y);
        assert_eq!(x.allow_raw_access, Some(true));
        assert_eq!(y.allow_raw_access, Some(true));
        assert_eq!(x.enable_aliasing, Some(false));
        assert_eq!(y.enable_aliasing, Some(false));
        assert_eq!(x.ignore, Some(false));
        assert_eq!(y.ignore, Some(false));
        assert_eq!(x.cache.clone().unwrap().max_age, Some(22));
        assert_eq!(y.cache.clone().unwrap().max_age, Some(22));
    }
}
