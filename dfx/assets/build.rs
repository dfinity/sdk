use flate2::write::GzEncoder;
use flate2::Compression;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn add_assets(f: &mut File, path: &str) -> () {
    let out_dir = env::var("OUT_DIR").unwrap();
    let tgz_path = Path::new(&out_dir).join(format!("{}.tgz", path));
    let tar_gz = File::create(&tgz_path).unwrap();
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all("", path).unwrap();

    f.write_all(
        format!(
            "
        pub fn assets() -> Result<Archive<GzDecoder<Cursor<Vec<u8>>>>> {{
            let mut v = Vec::new();
            v.extend_from_slice(include_bytes!(\"{path}.tgz\"));

            let tar = GzDecoder::new(std::io::Cursor::new(v));
            let archive = Archive::new(tar);
            Ok(archive)
        }}
    ",
            path = path
        )
        .as_bytes(),
    )
    .unwrap();
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

    println!("OPENSSL_STATIC: {:#?}", env::var("OPENSSL_STATIC"));
    println!("CARGO_HOME: {:#?}", env::var("CARGO_HOME"));
    println!("DFX_ASSETS: {:#?}", env::var("DFX_ASSETS"));
    let path = env::var("DFX_ASSETS").unwrap();
    add_assets(&mut f, &path);
}
