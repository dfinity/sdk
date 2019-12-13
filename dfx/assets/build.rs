use flate2::write::GzEncoder;
use flate2::Compression;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn add_assets(fn_name: &str, f: &mut File, path: &str) -> () {
    let out_dir = env::var("OUT_DIR").unwrap();
    let tgz_path = Path::new(&out_dir).join(format!("{}.tgz", fn_name));
    let tar_gz = File::create(&tgz_path).unwrap();
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all("", path).unwrap();

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
    use std::process::Command;

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

    let path = env::var("DFX_ASSETS").expect("Cannot find DFX_ASSETS");
    add_assets("binary_cache", &mut f, &path);
    add_assets("language_bindings", &mut f, "assets/language_bindings");
    add_assets("build_assets", &mut f, "assets/build_assets");
    add_assets("new_project_files", &mut f, "assets/new_project_files");
    add_assets(
        "new_project_node_files",
        &mut f,
        "assets/new_project_node_files",
    );

    // Pass the git describe version at time of build.
    if let Some(git) = get_git_hash() {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={}", git);
    } else {
        // Nothing to do here, as there is no GIT. We keep the CARGO_PKG_VERSION.
    }
}
