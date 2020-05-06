use lazy_static::lazy_static;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Result;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::{fs, fs::create_dir_all};

lazy_static! {
    static ref VERSION: Version =
        Version::parse("0.0.1").expect("Failed to parse version requirements");
}

// #[derive(Serialize, Deserialize)]
// #[serde(rename_all = "snake_case")]
#[derive(Debug)]
pub struct FileHierarchy {
    location: PathBuf,
    version: Version,
    #[allow(dead_code)]
    inner: HashMap<ProfileIdentifier, PrincipalProfile>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProfileIdentifier(String);

impl ProfileIdentifier {
    pub fn new(id: impl AsRef<str>) -> Self {
        Self(id.as_ref().to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PrincipalProfile {
    version: Version,
    #[serde(rename = "access")]
    // The key to the access file must be unique.
    access_files: HashMap<String, AccessFile>,
}

impl PrincipalProfile {
    fn empty() -> Self {
        Self {
            version: VERSION.clone(),
            access_files: HashMap::new(),
        }
    }

    fn add_access_file(mut self, id: String, access_file: AccessFile) -> Self {
        self.access_files.insert(id, access_file);
        self
    }
}

#[derive(Debug, Clone)]
pub struct UserProfile {
    principal_profile: PrincipalProfile,
    access_content: String,
    profile_identifier: ProfileIdentifier,
}

impl UserProfile {
    pub fn new_with_key(user: impl AsRef<str>, key: String, key_id: impl AsRef<str>) -> Self {
        let access_file = AccessFile {
            access_type: AccessType::PrivateKey,
            path: PathBuf::from(key_id.as_ref().to_owned() + ".pem"),
        };

        let principal_profile =
            PrincipalProfile::empty().add_access_file(key_id.as_ref().to_owned(), access_file);
        let profile_identifier = ProfileIdentifier::new(user);
        Self {
            principal_profile,
            access_content: key,
            profile_identifier,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct PrincipalProfileId(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AccessFile {
    pub access_type: AccessType,
    // This must be a relative path.
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessType {
    PrivateKey,
    EncPrivateKey,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
enum Action {
    Default,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Metadata {
    version: Version,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct PrincipalProfileProject {
    version: Version,
    #[serde(rename = "default")]
    default_action: Option<HashMap<Action, PrincipalProfileId>>,
}

// XXX
pub fn set_default(profile: PrincipalProfile, project_path_dfx: &Path) -> Result<()> {
    let default_principal = serde_json::to_string(&profile)?;
    fs::write(project_path_dfx, default_principal)
}

impl FileHierarchy {
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            location: root_path,
            version: VERSION.clone(),
            inner: HashMap::new(),
        }
    }

    /// Add a new profile. Consumes the file hierarchy, as it is now
    /// invalid. In the future this might return a new file hierarchy,
    /// but as this is a volatile value, requires IO, and we focus on
    /// each invocation on a single principal we simply consume the
    /// file hierarchy.
    pub fn add_profile(self, user_profile: UserProfile) -> Result<()> {
        let root_path = &self.location;
        let profile = user_profile.principal_profile;
        let access_content = user_profile.access_content;
        let identifier = user_profile.profile_identifier;
        // We want the root identity path to be absolute. Principal
        // profiles are a global resource: we should avoid promoting
        // fragmentation of principals.
        assert!(root_path.is_absolute());

        let metadata = fs::read_to_string(root_path.join("metadata.json"))?;
        let metadata: Metadata = serde_json::from_str(&metadata)?;
        let version = metadata.version;
        // We simplify a bit semver, and we only check the major
        // version for breaking changes.
        if version.major > VERSION.major {
            return Err(Error::new(
                ErrorKind::Other,
                "Incompatible user principal profile file version detected.",
            ));
        }

        println!("PHASE I");
        let principal_profile = serde_json::to_string(&profile)?;
        let principal_dir_path = root_path.join(identifier.0.clone());
        create_dir_all(&principal_dir_path)?;
        println!("PHASE II");
        let principal_file_path = principal_dir_path.join(identifier.0 + ".json");
        // XXXX  we should not override! FIX
        fs::write(principal_file_path, principal_profile)?;
        // Pick each access file. Currently we assume and handle a single file
        println!("PHASE III");
        let files_iter = profile.access_files.values();
        println!("PHASE IV");
        assert_eq!(files_iter.len(), 1);
        for access_file in files_iter {
            // let access_file_path = principal_dir_path.join(name);
            let access_file_path = principal_dir_path.join(access_file.path.clone());
            println!("path : {:?}", access_file_path);
            fs::write(access_file_path, &access_content)?;
        }
        println!("PHASE FIN");

        Ok(())
    }

    /// Load user desired profiles. Note this is not checking if the
    /// version is valid. We assume we will fail during parsing. We read
    /// robust-fully and write pessimistically.
    ////Version::new(0, 1, 0),
    pub fn partial_load_file_hierarchy(
        profiles: &[ProfileIdentifier],
        // version: Version,
        root: &Path,
    ) -> Result<FileHierarchy> {
        // let fh = OpenOptions::new()
        //             .read(true)
        //             .write(false)
        //             .create(false)
        //             .open(root+p);

        let mut map = HashMap::new();
        println!("PHASE I");

        for profile_id in profiles.iter() {
            // let profile_id = ProfileIdentifier::new(p);
            let profile_name = profile_id.0.clone();
            let path = root.join(profile_name.clone()).join(profile_name + ".json");
            let contents = fs::read_to_string(path)?;
            let file: PrincipalProfile = serde_json::from_str(&contents)?;
            map.insert(profile_id.clone(), file);

            // Add version checks here. XX
        }
        println!("PHASE II X");

        let metadata = fs::read_to_string(root.join("metadata.json"))?;
        println!("PHASE III");

        let metadata: Metadata = serde_json::from_str(&metadata)?;
        let version = metadata.version;
        Ok(FileHierarchy {
            location: root.to_path_buf(),
            version,
            inner: map,
        })
    }

    pub fn add_metadata(&self) -> Result<()> {
        let identity_root = self.location.clone();
        let metadata = Metadata {
            version: self.version.clone(),
        };

        let metadata_encoded = serde_json::to_string(&metadata)?;
        fs::write(identity_root.join("metadata.json"), metadata_encoded)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use std::fs::File;
    use std::io::{self, Read, Write};
    use tempfile::tempdir;

    #[test]
    fn test_parsing() {
        let mut map = HashMap::new();
        map.insert(Action::Default, PrincipalProfileId("Bob".to_owned()));
        let project_default = PrincipalProfileProject {
            version: Version::new(1, 2, 3),
            default_action: Some(map),
        };
        let parsed_profile =
            serde_json::to_string_pretty(&project_default).expect("Failed to parse profile");
        println!("{:?}", &parsed_profile);
        assert_eq!(
            "{\n  \"version\": \"1.2.3\",\n  \"default\": {\n    \"default\": \"Bob\"\n  }\n}"
                .to_owned(),
            parsed_profile
        );
    }
    #[test]
    fn test_parsing_principal() {
        let dir = tempdir().unwrap();

        // Create profiles and dummy key files
        let key1 = "this is a key".to_owned();
        let key2 = "this is a key also I think".to_owned();
        let alice_profile = UserProfile::new_with_key("Alice", key1.clone(), "main_key");
        let bob_profile = UserProfile::new_with_key("Bob", key2.clone(), "main_key");

        // Create file hierarchy.
        let root = dir.path().to_path_buf();
        let fh = FileHierarchy::new(root.clone());
        fh.add_metadata().unwrap();
        fh.add_profile(alice_profile.clone())
            .expect("Failed to add profile");

        let fh =
            FileHierarchy::partial_load_file_hierarchy(&[ProfileIdentifier::new("Alice")], &root)
                .unwrap();
        assert_eq!(fh.location, root.clone());
        assert_eq!(fh.version, VERSION.clone());
        assert_eq!(
            fh.inner.get(&ProfileIdentifier::new("Alice")),
            Some(&alice_profile.principal_profile)
        );
        println!("{:?}", fh);

        dir.close().unwrap();
    }
}
