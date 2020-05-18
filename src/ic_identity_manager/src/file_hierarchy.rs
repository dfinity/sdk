use lazy_static::lazy_static;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Result;
use std::io::Write;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::{fs, fs::create_dir_all, fs::OpenOptions};

lazy_static! {
    static ref VERSION: Version =
        Version::parse("0.0.1").expect("Failed to parse version requirements");
}

#[derive(Debug)]
pub(crate) struct FileHierarchy {
    pub location: PathBuf,
    version: Version,
    // We can keep it simple as this is still an internal type to
    pub inner: HashMap<ProfileIdentifier, PrincipalProfile>,
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

    pub fn get_default_file(&self, profile_id: ProfileIdentifier) -> Option<AccessFile> {
        self.access_files
            .get("main_key")
            .cloned()
            // Return the relative path to the root of the file hierarchy.
            .map(|mut r_path| {
                r_path.path = PathBuf::from(profile_id.0).join(r_path.path);
                r_path
            })
    }
}

#[derive(Debug, Clone)]
pub struct UserProfile {
    principal_profile: PrincipalProfile,
    access_content: Vec<u8>,
    profile_identifier: ProfileIdentifier,
}

impl UserProfile {
    pub fn new_with_key(
        user: impl AsRef<str>,
        key: impl AsRef<[u8]>,
        key_id: impl AsRef<str>,
    ) -> Self {
        let access_file = AccessFile {
            access_type: AccessType::PrivateKey,
            path: PathBuf::from(key_id.as_ref().to_owned() + ".pem"),
        };

        let principal_profile =
            PrincipalProfile::empty().add_access_file(key_id.as_ref().to_owned(), access_file);
        let profile_identifier = ProfileIdentifier::new(user);
        Self {
            principal_profile,
            access_content: key.as_ref().to_vec(),
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

impl FileHierarchy {
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            location: root_path,
            version: VERSION.clone(),
            inner: HashMap::new(),
        }
    }

    /// Setup adds necessary metadata fStringiles in the provided path if
    /// necessary. We first check if the existing metadata is valid. If
    /// not, we **fail** as we can not distinguish outdated
    pub fn setup(&self) -> Result<()> {
        // Check if there is a metadata file.
        match fs::read_to_string(&self.location.join("metadata.json")) {
            Ok(metadata) => {
                let m: serde_json::Result<Metadata> = serde_json::from_str(&metadata);
                m.map(|_| ()).map_err(|e| e.into())
            }
            // If there is none, create one.
            Err(_) => self.add_metadata(),
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

        let principal_profile = serde_json::to_string_pretty(&profile)?;
        let principal_dir_path = root_path.join(identifier.0.clone());
        create_dir_all(&principal_dir_path)?;

        let principal_file_path = principal_dir_path.join(identifier.0 + ".json");

        let mut file_handle = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(principal_file_path)?;
        file_handle.write_all(principal_profile.as_ref())?;

        // Pick each access file. Currently we assume and handle a
        // single file.
        let files_iter = profile.access_files.values();

        assert_eq!(files_iter.len(), 1);
        for access_file in files_iter {
            // let access_file_path = principal_dir_path.join(name);
            let access_file_path = principal_dir_path.join(access_file.path.clone());
            println!("path : {:?}", access_file_path);
            fs::write(access_file_path, &access_content)?;
        }

        Ok(())
    }

    /// Load user desired profiles. Note this is not checking if the
    /// version is valid. We assume we will fail during parsing. We read
    /// robust-fully and write pessimistically.
    pub fn partial_load_file_hierarchy(
        profiles: &[ProfileIdentifier],
        root: &Path,
    ) -> Result<FileHierarchy> {
        let mut map = HashMap::new();

        for profile_id in profiles.iter() {
            let profile_name = profile_id.0.clone();
            let path = root.join(profile_name.clone()).join(profile_name + ".json");
            let contents = fs::read_to_string(path)?;
            let file: PrincipalProfile = serde_json::from_str(&contents)?;
            map.insert(profile_id.clone(), file);

            // TODO(eftychis): Add version checks here.
        }

        let metadata = fs::read_to_string(root.join("metadata.json"))?;

        let metadata: Metadata = serde_json::from_str(&metadata)?;
        let version = metadata.version;
        Ok(FileHierarchy {
            location: root.to_path_buf(),
            version,
            inner: map,
        })
    }

    fn add_metadata(&self) -> Result<()> {
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

    use tempfile::tempdir;

    #[test]
    fn test_parsing_principal() {
        let dir = tempdir().unwrap();

        // Create profiles and dummy key files
        let key1 = "this is a key".to_owned();
        let key2 = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let alice_profile = UserProfile::new_with_key("Alice", key1.clone(), "main_key");
        let bob_profile = UserProfile::new_with_key("Bob", key2.clone(), "main_key");

        // Create file hierarchy.
        let root = dir.path().to_path_buf();
        let fh = FileHierarchy::new(root.clone());
        fh.setup().unwrap();
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
        // Check that profiles do not interfere.
        fh.add_profile(bob_profile.clone())
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
        assert_eq!(
            alice_profile
                .principal_profile
                .access_files
                .get("main_key")
                .unwrap()
                .path
                .clone(),
            PathBuf::from("main_key.pem")
        );

        assert_eq!(
            alice_profile
                .principal_profile
                .get_default_file(ProfileIdentifier::new("Alice"))
                .unwrap()
                .path,
            (PathBuf::from("Alice/main_key.pem"))
        );

        let key1_contents = fs::read_to_string(root.join("Alice").join("main_key.pem")).unwrap();
        assert_eq!(key1_contents, key1);
        let key2_contents = fs::read_to_string(root.join("Bob").join("main_key.pem")).unwrap();
        assert_eq!(key2_contents, String::from_utf8(key2).unwrap());

        let fh =
            FileHierarchy::partial_load_file_hierarchy(&[ProfileIdentifier::new("Bob")], &root)
                .unwrap();
        assert_eq!(fh.location, root);
        assert_eq!(fh.version, VERSION.clone());
        assert_eq!(
            fh.inner.get(&ProfileIdentifier::new("Bob")),
            Some(&bob_profile.principal_profile)
        );
        assert_eq!(
            bob_profile
                .principal_profile
                .access_files
                .get("main_key")
                .unwrap()
                .path
                .clone(),
            PathBuf::from("main_key.pem")
        );

        assert_eq!(
            bob_profile
                .principal_profile
                .get_default_file(ProfileIdentifier::new("Bob"))
                .unwrap()
                .path,
            (PathBuf::from("Bob/main_key.pem"))
        );

        dir.close().unwrap();
    }
}
