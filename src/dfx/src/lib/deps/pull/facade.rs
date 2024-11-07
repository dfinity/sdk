use candid::{pretty::candid::pp_args, Principal};
use candid_parser::utils::{instantiate_candid, CandidSource};
use dfx_core::config::cache::get_cache_root;
use dfx_core::fs::{composite::ensure_parent_dir_exists, read, read_to_string, write};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use super::{super::PulledCanister, write_to_tempfile_then_rename};
use crate::lib::deps::{
    get_pulled_canister_dir, get_pulled_service_candid_path, get_pulled_wasm_path,
};
use crate::lib::error::DfxResult;
use crate::util::{download_file, download_file_to_path};

static IC_REV: &str = "1eeb4d74deb00bd52739cbd6f37ce1dc72e0c76e";

#[derive(Debug)]
struct Facade {
    wasm_url: String,
    candid_url: String,
    dependencies: Vec<Principal>,
    init_guide: String,
    init_arg: String,
}

lazy_static::lazy_static! {
    static ref ICP_LEDGER: Principal=Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
    static ref FACADE: HashMap<Principal, Facade> = {
        let mut m = HashMap::new();
        m.insert(
            *ICP_LEDGER,
            Facade {
                wasm_url: format!("https://download.dfinity.systems/ic/{IC_REV}/canisters/ledger-canister.wasm.gz"),
                candid_url: format!("https://raw.githubusercontent.com/dfinity/ic/{IC_REV}/rs/ledger_suite/icp/ledger.did"),
                dependencies:vec![],
                init_guide: "The account of the anonymous identity will be the minting_account.".to_string(),
                init_arg:
r#"(variant { 
    Init = record {
        minting_account = "1c7a48ba6a562aa9eaa2481a9049cdf0433b9738c992d698c31d8abf89cadc79";
        initial_values = vec {};
        send_whitelist = vec {};
        transfer_fee = opt record { e8s = 10_000 : nat64; };
        token_symbol = opt "LICP";
        token_name = opt "Local ICP"; 
    }
})"#.to_string()
            }
        );
        m
    };
}

pub(super) fn facade_dependencies(canister_id: &Principal) -> Option<Vec<Principal>> {
    FACADE
        .get(canister_id)
        .map(|facade| facade.dependencies.clone())
}

pub(super) async fn facade_download(canister_id: &Principal) -> DfxResult<Option<PulledCanister>> {
    if let Some(facade) = FACADE.get(canister_id) {
        let mut pulled_canister = PulledCanister {
            dependencies: facade.dependencies.clone(),
            init_guide: facade.init_guide.clone(),
            init_arg: Some(facade.init_arg.clone()),
            gzip: facade.wasm_url.ends_with(".gz"),
            ..Default::default()
        };
        let ic_rev_path = get_cache_root()?
            .join("pulled")
            .join(".facade")
            .join(canister_id.to_text());
        let wasm_path = get_pulled_wasm_path(canister_id, pulled_canister.gzip)?;
        let service_candid_path = get_pulled_service_candid_path(canister_id)?;
        let mut cache_hit = false;
        if ic_rev_path.exists() && wasm_path.exists() && service_candid_path.exists() {
            let ic_rev = read_to_string(&ic_rev_path)?;
            if ic_rev == IC_REV {
                cache_hit = true;
            }
        }
        if !cache_hit {
            // delete files from previous pull
            let pulled_canister_dir = get_pulled_canister_dir(canister_id)?;
            if pulled_canister_dir.exists() {
                dfx_core::fs::remove_dir_all(&pulled_canister_dir)?;
            }
            dfx_core::fs::create_dir_all(&pulled_canister_dir)?;
            // download wasm and candid
            let wasm_url = reqwest::Url::parse(&facade.wasm_url)?;
            download_file_to_path(&wasm_url, &wasm_path).await?;
            let candid_url = reqwest::Url::parse(&facade.candid_url)?;
            let candid_bytes = download_file(&candid_url).await?;
            let candid_service = String::from_utf8(candid_bytes)?;
            write_to_tempfile_then_rename(candid_service.as_bytes(), &service_candid_path)?;
            // write ic_rev for cache logic
            ensure_parent_dir_exists(&ic_rev_path)?;
            write(&ic_rev_path, IC_REV)?;
        }

        // wasm_hash
        let wasm_content = read(&wasm_path)?;
        let wasm_hash = Sha256::digest(wasm_content).to_vec();
        pulled_canister.wasm_hash = hex::encode(&wasm_hash);
        pulled_canister.wasm_hash_download = hex::encode(&wasm_hash);

        // candid_args
        let candid_service = read_to_string(&service_candid_path)?;
        let candid_source = CandidSource::Text(&candid_service);
        let (args, _service) = instantiate_candid(candid_source)?;
        let candid_args = pp_args(&args).pretty(80).to_string();
        pulled_canister.candid_args = candid_args;

        Ok(Some(pulled_canister))
    } else {
        Ok(None)
    }
}
