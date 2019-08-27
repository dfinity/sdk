
use flate2::Compression;
use flate2::write::GzEncoder;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let tgz_path = Path::new(&out_dir).join("assets.tar.gz");
    let loader_path = Path::new(&out_dir).join("load_assets.rs");
    let mut f = File::create(&loader_path).unwrap();
    let tar_gz = File::create(&tgz_path).unwrap();

    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all("", "./assets/files").unwrap();

    f.write_all(b"
        use flate2::read::GzDecoder;
        use tar::Archive;

        pub fn get_assets() -> std::io::Result<tar::Archive<flate2::read::GzDecoder<std::io::Cursor<std::vec::Vec<u8>>>>> {
            let mut v = Vec::new();
            v.extend_from_slice(include_bytes!(\"assets.tar.gz\"));

            let tar = GzDecoder::new(std::io::Cursor::new(v));
            let archive = Archive::new(tar);
            Ok(archive)
        }
    ").unwrap();
}
