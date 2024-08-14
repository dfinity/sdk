use std::{env, fs::File, io::Write, path::Path};

const CANISTER_IDS_URL: &str = "https://raw.githubusercontent.com/dfinity/ic/1402bf35308ec9bd87356c26f7c430f49b49423a/rs/nns/canister_ids.json";
fn define_well_known_canisters() {
    let well_known_canisters = reqwest::blocking::get(CANISTER_IDS_URL)
        .unwrap()
        .error_for_status()
        .unwrap()
        .text()
        .unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();
    let loader_path = Path::new(&out_dir).join("well_known_canisters.rs");
    let mut f = File::create(loader_path).unwrap();
    f.write_all(
        format!(
            "
const WELL_KNOWN_CANISTERS: &str = r#\"
{}
\"#;

pub fn map_wellknown_canisters() -> CanisterIds {{
    serde_json::from_str(WELL_KNOWN_CANISTERS).unwrap_or(CanisterIds::new())
}}
",
            well_known_canisters.replace("mainnet", "ic")
        )
        .as_bytes(),
    )
    .unwrap()
}

fn main() {
    define_well_known_canisters();
}
