use std::{env, fs::File, io::Write, path::Path};

use itertools::Itertools;
use serde_json::Value;

fn define_well_known_canisters() {
    let well_known_canisters = std::fs::read_to_string(format!(
        "{}/src/assets/canister_ids.json",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let well_known_canisters = serde_json::from_str::<Value>(&well_known_canisters).unwrap();
    let well_known_canisters = well_known_canisters.as_object().unwrap();

    let well_known_canisters = well_known_canisters.iter().map(|(key, val)| {
        (
            key.as_str(),
            val.as_object()
                .unwrap()
                .values()
                .last()
                .unwrap()
                .as_str()
                .unwrap(),
        )
    });

    let out_dir = env::var("OUT_DIR").unwrap();
    let loader_path = Path::new(&out_dir).join("well_known_canisters.rs");
    let mut f = File::create(loader_path).unwrap();
    f.write_all(
        format!(
            "
const WELL_KNOWN_CANISTERS: &[(&str, &str)] = &[
{}
];

pub fn map_wellknown_canisters() -> HashMap<CanisterName, Principal> {{
    WELL_KNOWN_CANISTERS.iter().map(|(key, value)| (key.to_string(), (*value).try_into().unwrap())).collect()
}}
",
            well_known_canisters
                .map(|(key, val)| format!("(\"{}\", \"{}\")", key, val))
                .join(",\n")
        )
        .as_bytes(),
    )
    .unwrap()
}

fn main() {
    define_well_known_canisters();
}
