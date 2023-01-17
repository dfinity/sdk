use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;

use crate::lib::error::DfxResult;
use crate::lib::extension::manifest::ExtensionLocation;
use crate::lib::extension::manifest::ExtensionsManifest;
use anyhow::Error;
use reqwest::Url;
use semver::Prerelease;
use tempfile::TempDir;
use tempfile::tempdir_in;

use super::ExtensionsManager;

impl ExtensionsManager {
    pub fn install_extension(&self, extension_name: &str) -> DfxResult<()> {
        // ) -> Result<std::fs::File, Error> {
        let mut dfx_version = self.dfx_version.clone();
        // Remove the prerelease tag, if any. This is because prerelease tags
        // should not be allowed in extension manifests, and semver crate
        // won't match a semver with a prerelease tag against a semver without.
        dfx_version.pre = Prerelease::EMPTY;

        let manifest = ExtensionsManifest::fetch().unwrap();

        if let Some(ExtensionLocation { download_url, .. }) = manifest.find(extension_name, dfx_version)
        {
            let url = Url::parse(&download_url)?;
            let tempdir = tempdir_in(&self.dir)?;
            let extension_dir = self.dir.join(extension_name);
            download_extension_to_tempdir(&tempdir, url, extension_name)?;
            std::fs::rename(tempdir, extension_dir)?;
            Ok(())
        } else {
            Err(Error::msg("either not found for dfx version or not found the extension at all"))
        }
    }
}

fn download_extension_to_tempdir(tempdir: &TempDir, download_url: Url, extension_name: &str) -> Result<(), Error> {
        let response = reqwest::blocking::get(download_url).unwrap();
        let tempdir_bin = tempdir.path().join(extension_name);
        let mut file = std::fs::File::create(tempdir_bin)?;
        let perm = std::fs::Permissions::from_mode(0o777);
        file.set_permissions(perm).unwrap();
        let mut content = Cursor::new(response.bytes().unwrap());
        std::io::copy(&mut content, &mut file)?;
        Ok(())
}

