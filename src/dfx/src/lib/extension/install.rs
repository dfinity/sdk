use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;

use crate::lib::extension::manifest::ExtensionLocation;
use crate::lib::extension::manifest::ExtensionsManifest;
use anyhow::Error;
use semver::Version;

use std::path::Path;

pub fn install_extension(
    extensions_dir: &Path,
    dfx_version: Version,
    extension_name: &str,
) -> Result<(), Error> {
    let manifest = ExtensionsManifest::fetch().unwrap();

    if let Some(ExtensionLocation { download_url }) = manifest.find(extension_name, dfx_version) {
        println!("installing: {} from {}", extension_name, &download_url);
        let extension_dir = extensions_dir.join(extension_name);
        let extension_archive = extension_dir.join(extension_name);
        let response = reqwest::blocking::get(&download_url).unwrap();
        std::fs::create_dir_all(extension_dir).unwrap();
        let mut file = std::fs::File::create(extension_archive)?;
        let perm = std::fs::Permissions::from_mode(0o777);
        file.set_permissions(perm).unwrap();
        let mut content = Cursor::new(response.bytes().unwrap());
        std::io::copy(&mut content, &mut file)?;
    } else {
        println!("either not found for dfx version or not found the extension at all");
    }

    Ok(())
}
