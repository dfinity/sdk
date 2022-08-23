use crate::lib::error::{DfxError, DfxResult};
use crate::lib::manifest::Manifest;
use crate::{error_invalid_argument, error_invalid_data};

use anyhow::Context;
use flate2::read::GzDecoder;
use fn_error_context::context;
use indicatif::{ProgressBar, ProgressDrawTarget};
use semver::Version;
use std::fs;
use std::io::Write;
use std::path::Path;
use tar::Archive;

pub static DEFAULT_RELEASE_ROOT: &str = "https://sdk.dfinity.org";
pub static CACHE_ROOT: &str = ".cache/dfinity/versions/";
pub static DOWNLOADS_DIR: &str = ".cache/dfinity/downloads/";

#[context("Failed to get distribution manifest.")]
pub fn get_manifest() -> DfxResult<Manifest> {
    let url_string = format!("{}/manifest.json", DEFAULT_RELEASE_ROOT);
    let url = reqwest::Url::parse(&url_string)
        .map_err(|e| error_invalid_argument!("invalid manifest URL: {}", e))?;

    let b = ProgressBar::new_spinner();
    b.set_draw_target(ProgressDrawTarget::stderr());

    b.set_message(format!("Fetching manifest {}", url));
    b.enable_steady_tick(80);

    let response = reqwest::blocking::get(url).map_err(DfxError::new)?;
    let status_code = response.status();
    b.finish_and_clear();

    if !status_code.is_success() {
        return Err(error_invalid_data!(
            "unable to fetch manifest: {}",
            status_code.canonical_reason().unwrap_or("unknown error"),
        ));
    }

    response
        .json()
        .map_err(|e| error_invalid_data!("invalid manifest: {}", e))
}

// Download a SDK version to cache
#[context("Failed to download and install version '{}'.", version)]
pub fn install_version(version: &Version) -> DfxResult<()> {
    let arch_os = match std::env::consts::OS {
        "linux" => "x86_64-linux",
        "macos" => "x86_64-darwin",
        _ => panic!("Not supported architecture"),
    };
    let url = reqwest::Url::parse(&format!(
        "{0}/downloads/dfx/{1}/{2}/dfx-{1}.tar.gz",
        DEFAULT_RELEASE_ROOT, version, arch_os
    ))
    .map_err(|e| error_invalid_argument!("invalid url: {}", e))?;

    let home = std::env::var("HOME").context("Failed to resolve env var HOME.")?;
    let home = Path::new(&home);

    let download_dir = home.join(DOWNLOADS_DIR);
    if !download_dir.exists() {
        fs::create_dir_all(&download_dir)
            .with_context(|| format!("Failed to create dir {}.", download_dir.to_string_lossy()))?;
    }
    let download_file = download_dir.join(&format!("dfx-{}.tar.gz", version));
    if download_file.exists() {
        println!("Found downloaded file {}", download_file.to_string_lossy());
    } else {
        let mut dest = fs::File::create(&download_file).with_context(|| {
            format!("Failed to create file {}.", download_file.to_string_lossy())
        })?;
        let b = ProgressBar::new_spinner();
        b.set_draw_target(ProgressDrawTarget::stderr());
        b.set_message(format!("Downloading {}", url));
        b.enable_steady_tick(80);
        let response = reqwest::blocking::get(url).map_err(DfxError::new)?;
        let content = response.bytes().context("Failed to get response body.")?;
        dest.write_all(&*content).with_context(|| {
            format!(
                "Failed to write response content to {}.",
                download_file.to_string_lossy()
            )
        })?;
        b.finish_with_message("Download complete");
    }

    let mut cache_dir = home.join(CACHE_ROOT);
    cache_dir.push(version.to_string());
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)
            .with_context(|| format!("Failed to create {}.", cache_dir.to_string_lossy()))?;
    }

    let b = ProgressBar::new_spinner();
    b.set_draw_target(ProgressDrawTarget::stderr());
    b.set_message(format!(
        "Unpacking file {}",
        download_file.to_string_lossy()
    ));
    b.enable_steady_tick(80);
    let tar_gz = fs::File::open(&download_file)
        .with_context(|| format!("Failed to open {}.", download_file.to_string_lossy()))?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(&cache_dir).with_context(|| {
        format!(
            "Failed to unpack archive at {}.",
            download_file.to_string_lossy()
        )
    })?;
    b.finish_with_message("Unpack complete");

    // Install components
    let dfx = cache_dir.join("dfx");
    std::process::Command::new(dfx)
        .args(&["cache", "install"])
        .status()
        .map_err(DfxError::from)?;

    Ok(())
}
