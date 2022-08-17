use flate2::write::GzEncoder;
use flate2::Compression;
use openssl::sha::Sha256;
use std::fs::{read_to_string, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};
use walkdir::WalkDir;

const INPUTS: &[&str] = &[
    "nix/sources.json",
    "scripts/dfx-asset-sources.sh",
    "scripts/prepare-dfx-assets.sh",
    "src/distributed/assetstorage.did",
    "src/distributed/assetstorage.wasm",
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

    hex::encode(sha256.finish())
}

fn find_assets() -> PathBuf {
    println!("cargo:rerun-if-env-changed=DFX_ASSETS");
    if let Ok(a) = env::var("DFX_ASSETS") {
        PathBuf::from(a)
    } else {
        let project_root_dir = format!("{}/../..", env!("CARGO_MANIFEST_DIR"));
        let project_root_path: PathBuf = PathBuf::from(project_root_dir)
            .canonicalize()
            .expect("Unable to determine project root");

        let prepare_script_path = project_root_path.join("scripts/prepare-dfx-assets.sh");
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

        let result = Command::new(&prepare_script_path)
            .arg(&dfx_assets_path)
            .output()
            .expect("unable to run prepare script");
        if !result.status.success() {
            println!(
                "cargo:error=unable to run {}:",
                prepare_script_path.to_string_lossy()
            );
            println!("cargo:error={}", String::from_utf8_lossy(&result.stderr));
            std::process::exit(1)
        }

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

    let tar_gz = File::create(&tgz_path).unwrap();
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all("", path).unwrap();

    write_archive_accessor(fn_name, f);
}

fn write_archive_accessor(fn_name: &str, f: &mut File) {
    f.write_all(
        format!(
            "
        pub fn {fn_name}() -> Result<Archive<GzDecoder<Cursor<Vec<u8>>>>> {{
            let mut v = Vec::new();
            v.extend_from_slice(include_bytes!(\"{fn_name}.tgz\"));

            let tar = GzDecoder::new(std::io::Cursor::new(v));
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

fn get_git_hash() -> Option<String> {
    match Command::new("git").arg("describe").arg("--dirty").output() {
        Ok(output) if output.status.success() => {
            Some(String::from_utf8_lossy(&output.stdout).to_string())
        }
        _ => None,
    }
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let loader_path = Path::new(&out_dir).join("load_assets.rs");
    let mut f = File::create(&loader_path).unwrap();

    f.write_all(
        b"
        use flate2::read::GzDecoder;
        use std::io::{Cursor, Result};
        use std::vec::Vec;
        use tar::Archive;

    ",
    )
    .unwrap();

    let dfx_assets = find_assets();
    add_asset_archive("binary_cache", &mut f, &dfx_assets);
    add_asset_archive("assetstorage_canister", &mut f, &dfx_assets);
    add_asset_archive("wallet_canister", &mut f, &dfx_assets);
    add_asset_archive("ui_canister", &mut f, &dfx_assets);
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

    // Pass a version in the environment, or the git describe version at time of build,
    // or let the cargo.toml version.
    if let Ok(v) = std::env::var("DFX_VERSION") {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={}", v);
    } else if let Some(git) = get_git_hash() {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={}", git);
    } else {
        // Nothing to do here, as there is no GIT. We keep the CARGO_PKG_VERSION.
    }
}
