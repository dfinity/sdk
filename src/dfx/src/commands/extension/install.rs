use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::extension::manifest::fetch_dfinity_extension_manifest;

use std::io::Cursor;

use clap::Parser;
use semver::{Prerelease, VersionReq};


#[derive(Parser)]
pub struct InstallOpts {
    /// Specifies the name of the extension to install.
    extension_name: String,
}

pub fn exec(env: &dyn Environment, opts: InstallOpts) -> DfxResult<()> {
    let manifest = fetch_dfinity_extension_manifest();

    let mut v = env.get_version().clone();
    // Remove the prerelease tag, if any. This is because prerelease tags
    // should not be allowed in extension manifests, and semver crate
    // won't match a semver with a prerelease tag against a semver without.
    v.pre = Prerelease::EMPTY;

    for (dfx_version, manifests) in manifest.unwrap().0.iter() {
        let sv_test = VersionReq::parse(dfx_version);
        if sv_test.unwrap().matches(&v) {
            if let Some(extension_location) = manifests.get(&opts.extension_name) {
                println!(
                    "installing: {} from {}",
                    &opts.extension_name, &extension_location.download_url
                );
                let extension_dir = env
                    .get_cache()
                    .get_extensions_directory()
                    .unwrap()
                    .join(&opts.extension_name);
                let extension_archive = extension_dir.join(&opts.extension_name);
                let response = reqwest::blocking::get(&extension_location.download_url).unwrap();
                std::fs::create_dir_all(extension_dir).unwrap();
                let mut file = std::fs::File::create(extension_archive)?;
                let mut content = Cursor::new(response.bytes().unwrap());
                std::io::copy(&mut content, &mut file)?;
            } else {
                println!("Extension not found")
            }
        }
    }

    Ok(())
}
