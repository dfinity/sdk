use flate2::write::GzEncoder;
use flate2::Compression;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

fn find_assets() -> PathBuf {
    if let Ok(a) = env::var("DFX_ASSETS_NNONO") {
        PathBuf::from(a)
    } else {
        let project_root_dir = format!("{}/../..", env!("CARGO_MANIFEST_DIR"));
        let project_root_path: PathBuf = PathBuf::from(project_root_dir)
            .canonicalize()
            .expect("Unable to determine project root");
        let dfx_assets_path = project_root_path.join(".dfx-assets");

        if !dfx_assets_path.exists() {
            let prepare_script_path = project_root_path.join("scripts/prepare-dfx-assets.sh");
            let result = Command::new(&prepare_script_path)
                .output()
                .expect("unable to run prepare script");
            if !result.status.success() {
                eprintln!(
                    "cargo:error=unable to run {}:",
                    prepare_script_path.to_string_lossy()
                );
                eprintln!("cargo:error={}", String::from_utf8_lossy(&result.stderr));
                std::process::exit(1)
            }
        }

        dfx_assets_path
    }
    // } else {
    //     let assets_nix = PathBuf::from(format!("{}/../../assets.nix", env!("CARGO_MANIFEST_DIR")))
    //         .canonicalize()
    //         .expect("assets.nix doesn't exist!");
    //     eprintln!("cargo:rerun-if-changed={}", assets_nix.display());
    //     let assets = Command::new("nix-build")
    //         .arg("--no-out-link")
    //         .arg(assets_nix)
    //         .output()
    //         .expect("unable to run local nix-build");
    //     if !assets.status.success() {
    //         eprintln!("cargo:warning=unable to run nix-build:");
    //         eprintln!("cargo:warning={}", String::from_utf8_lossy(&assets.stderr));
    //         std::process::exit(1)
    //     }
    //     let path = String::from_utf8_lossy(&assets.stdout)
    //         .trim_end()
    //         .to_string();
    //     env::set_var("DFX_ASSETS", &path);
    //     PathBuf::from(path)
    // }
}

fn add_asset_archive(fn_name: &str, f: &mut File) {
    let filename_tgz = format!("{}.tgz", fn_name);

    let assets_path = find_assets();
    let prebuilt_file = assets_path.join(&filename_tgz);

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
    let describe = Command::new("git").arg("describe").arg("--dirty").output();

    if let Ok(output) = describe {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
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

    add_asset_archive("binary_cache", &mut f);
    add_asset_archive("assetstorage_canister", &mut f);
    add_asset_archive("wallet_canister", &mut f);
    add_asset_archive("ui_canister", &mut f);
    add_assets_from_directory("language_bindings", &mut f, "assets/language_bindings");
    add_assets_from_directory("new_project_files", &mut f, "assets/new_project_files");
    add_assets_from_directory(
        "new_project_node_files",
        &mut f,
        "assets/new_project_node_files",
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
