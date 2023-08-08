use flate2::write::GzEncoder;
use flate2::Compression;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{env, fs};
use walkdir::WalkDir;

mod prepare_assets;

const INPUTS: &[&str] = &[
    "nix/sources.json",
    "src/dfx/assets/prepare_assets.rs",
    "src/dfx/assets/build.rs",
    "src/distributed/assetstorage.did",
    "src/distributed/assetstorage.wasm.gz",
    "src/distributed/ui.did",
    "src/distributed/ui.wasm",
    "src/distributed/wallet.did",
    "src/distributed/wallet.wasm",
];

fn calculate_hash_of_inputs(project_root_path: &Path) -> String {
    let mut sha256 = Sha256::new();

    for input in INPUTS {
        let pathname = project_root_path.join(input);
        let mut f = File::open(pathname).expect("unable to open input file");
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)
            .expect("unable to read input file");
        sha256.update(&buffer);
    }

    hex::encode(sha256.finalize())
}

fn get_project_root_path() -> PathBuf {
    let project_root_dir = format!("{}/../..", env!("CARGO_MANIFEST_DIR"));
    PathBuf::from(project_root_dir)
        .canonicalize()
        .expect("Unable to determine project root")
}

#[derive(Deserialize, Clone)]
struct Source {
    url: String,
    sha256: String,
}

impl Source {
    fn sha256(&self) -> Vec<u8> {
        hex::decode(&self.sha256).expect("Invalid SHA-256")
    }
}

#[derive(Deserialize)]
struct Sources {
    #[serde(rename = "x86_64-linux")]
    x86_64_linux: HashMap<String, Source>,
    #[serde(rename = "x86_64-darwin")]
    x86_64_darwin: HashMap<String, Source>,
    #[serde(rename = "replica-rev")]
    replica_rev: String,
}

fn find_assets(sources: Sources) -> PathBuf {
    println!("cargo:rerun-if-env-changed=DFX_ASSETS");
    if let Ok(a) = env::var("DFX_ASSETS") {
        PathBuf::from(a)
    } else {
        let project_root_path = get_project_root_path();
        for input in INPUTS {
            println!(
                "cargo:rerun-if-changed={}",
                project_root_path.join(input).display()
            );
        }
        let hash_of_inputs = calculate_hash_of_inputs(&project_root_path);

        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let dfx_assets_path = out_dir.join("dfx-assets");
        let last_hash_of_inputs_path = out_dir.join("dfx-assets-inputs-hash");

        if dfx_assets_path.exists() && last_hash_of_inputs_path.exists() {
            let last_hash_of_inputs = read_to_string(&last_hash_of_inputs_path)
                .expect("unable to read last hash of inputs");
            if last_hash_of_inputs == hash_of_inputs {
                return dfx_assets_path;
            }
        }

        let source_set = match (
            &*env::var("CARGO_CFG_TARGET_ARCH").unwrap(),
            &*env::var("CARGO_CFG_TARGET_OS").unwrap(),
        ) {
            ("x86_64" | "aarch64", "macos") => sources.x86_64_darwin, // rosetta
            ("x86_64", "linux" | "windows") => sources.x86_64_linux,
            (arch, os) => panic!("Unsupported OS type {arch}-{os}"),
        };
        prepare_assets::prepare(&dfx_assets_path, source_set);

        fs::write(last_hash_of_inputs_path, hash_of_inputs)
            .expect("unable to write last hash of inputs");
        dfx_assets_path
    }
}

fn add_asset_archive(fn_name: &str, f: &mut File, assets_path: &Path) {
    let filename_tgz = format!("{}.tgz", fn_name);

    let prebuilt_file = assets_path.join(&filename_tgz);
    println!("cargo:rerun-if-changed={}", prebuilt_file.display());

    let out_dir = env::var("OUT_DIR").unwrap();
    let tgz_path = Path::new(&out_dir).join(&filename_tgz);

    if tgz_path.exists() {
        // This avoids PermissionDenied errors.
        std::fs::remove_file(&tgz_path).unwrap();
    }
    std::fs::copy(prebuilt_file, tgz_path).unwrap();

    write_archive_accessor(fn_name, f);
}

fn add_assets_from_directory(fn_name: &str, f: &mut File, path: &str) {
    for file in WalkDir::new(path)
        .into_iter()
        .filter_map(|x| x.ok().filter(|entry| entry.file_type().is_file()))
    {
        println!("cargo:rerun-if-changed={}", file.path().display())
    }
    let out_dir = env::var("OUT_DIR").unwrap();
    let tgz_path = Path::new(&out_dir).join(format!("{}.tgz", fn_name));

    let tar_gz = File::create(tgz_path).unwrap();
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all("", path).unwrap();

    write_archive_accessor(fn_name, f);
}

fn write_archive_accessor(fn_name: &str, f: &mut File) {
    f.write_all(
        format!(
            "
        pub fn {fn_name}() -> Result<Archive<GzDecoder<&'static [u8]>>> {{
            let tar = GzDecoder::new(&include_bytes!(\"{fn_name}.tgz\")[..]);
            let archive = Archive::new(tar);
            Ok(archive)
        }}
    ",
            fn_name = fn_name,
        )
        .as_bytes(),
    )
    .unwrap();
}

/// Gets a git tag with the least number of revs between HEAD of current branch and the tag,
/// and combines is with SHA of the HEAD commit. Example of expected output: `0.12.0-beta.1-b9ace030`
fn get_git_hash() -> Result<String, std::io::Error> {
    let mut latest_tag = String::from("0");
    let mut latest_distance = u128::MAX;
    let tags = Command::new("git")
        .arg("tag")
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?
        .stdout;
    for tag in String::from_utf8_lossy(&tags).split_whitespace() {
        let output = Command::new("git")
            .arg("rev-list")
            .arg("--count")
            .arg(format!("{}..HEAD", tag))
            .arg(tag)
            .stdout(Stdio::piped())
            .spawn()?
            .wait_with_output()?
            .stdout;
        if let Some(count) = String::from_utf8_lossy(&output)
            .split_whitespace()
            .next()
            .and_then(|v| v.parse::<u128>().ok())
        {
            if count < latest_distance {
                latest_tag = String::from(tag);
                latest_distance = count;
            }
        }
    }
    let head_commit_sha = Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .output()?
        .stdout;
    let head_commit_sha = String::from_utf8_lossy(&head_commit_sha);
    let is_dirty = !Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()?
        .stdout
        .is_empty();

    Ok(format!(
        "{latest_tag}+rev{count}.{head_status}{head_commit_sha}",
        count = latest_distance,
        head_status = if is_dirty { "dirty-" } else { "" }
    ))
}

fn add_assets(sources: Sources) {
    let out_dir = env::var("OUT_DIR").unwrap();
    let loader_path = Path::new(&out_dir).join("load_assets.rs");
    let mut f = File::create(loader_path).unwrap();

    f.write_all(
        b"
        use flate2::read::GzDecoder;
        use std::io::Result;
        use std::vec::Vec;
        use tar::Archive;

    ",
    )
    .unwrap();

    let dfx_assets = find_assets(sources);
    add_asset_archive("binary_cache", &mut f, &dfx_assets);
    add_asset_archive("assetstorage_canister", &mut f, &dfx_assets);
    add_asset_archive("wallet_canister", &mut f, &dfx_assets);
    add_asset_archive("ui_canister", &mut f, &dfx_assets);
    add_asset_archive("btc_canister", &mut f, &dfx_assets);
    add_assets_from_directory("language_bindings", &mut f, "assets/language_bindings");
    add_assets_from_directory(
        "new_project_motoko_files",
        &mut f,
        "assets/new_project_motoko_files",
    );
    add_assets_from_directory(
        "new_project_node_files",
        &mut f,
        "assets/new_project_node_files",
    );
    add_assets_from_directory(
        "new_project_rust_files",
        &mut f,
        "assets/new_project_rust_files",
    );
    add_assets_from_directory(
        "new_project_no_frontend_files",
        &mut f,
        "assets/new_project_no_frontend_files",
    );
    add_assets_from_directory(
        "new_project_base_files",
        &mut f,
        "assets/new_project_base_files",
    );
}

/// Use a verion based on environment variable,
/// or the latest git tag plus sha of current git HEAD at time of build,
/// or let the cargo.toml version.
fn define_dfx_version() {
    if let Ok(v) = std::env::var("DFX_VERSION") {
        // If the version is passed in the environment, use that.
        // Used by the release process in .github/workflows/publish.yml
        println!("cargo:rustc-env=CARGO_PKG_VERSION={}", v);
    } else if let Ok(git) = get_git_hash() {
        // If the version isn't passed in the environment, use the git describe version.
        // Used when building from source.
        println!("cargo:rustc-env=CARGO_PKG_VERSION={}", git);
    } else {
        // Nothing to do here, as there is no GIT. We keep the CARGO_PKG_VERSION.
    }
}

fn define_replica_rev(replica_rev: &str) {
    println!("cargo:rustc-env=DFX_ASSET_REPLICA_REV={}", replica_rev);
}

fn main() {
    let sources: Sources = toml::from_str(
        &fs::read_to_string("assets/dfx-asset-sources.toml")
            .expect("unable to read dfx-asset-sources.toml"),
    )
    .expect("unable to parse dfx-asset-sources.toml");
    define_replica_rev(&sources.replica_rev);
    add_assets(sources);
    define_dfx_version();
}
