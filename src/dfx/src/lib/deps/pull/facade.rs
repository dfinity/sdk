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
}

lazy_static::lazy_static! {
    static ref ICP_LEDGER: Principal=Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
    static ref CKBTC_LEDGER: Principal=Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai").unwrap();
    static ref CKETH_LEDGER: Principal=Principal::from_text("ss2fx-dyaaa-aaaar-qacoq-cai").unwrap();
    static ref FACADE: HashMap<Principal, Facade> = {
        let mut m = HashMap::new();
        m.insert(
            *ICP_LEDGER,
            Facade {
                wasm_url: format!("https://download.dfinity.systems/ic/{IC_REV}/canisters/ledger-canister.wasm.gz"),
                candid_url: format!("https://raw.githubusercontent.com/dfinity/ic/{IC_REV}/rs/ledger_suite/icp/ledger.did"),
                dependencies:vec![],
                init_guide: r#"
1. Create a 'minter' identity: dfx identity new minter
2. Run the following multi-line command:

dfx deps init ryjl3-tyaaa-aaaaa-aaaba-cai --argument "(variant { 
    Init = record {
        minting_account = \"$(dfx --identity minter ledger account-id)\";
        initial_values = vec {};
        send_whitelist = vec {};
        transfer_fee = opt record { e8s = 10_000 : nat64; };
        token_symbol = opt \"LICP\";
        token_name = opt \"Local ICP\"; 
    }
})"
"#.to_string(),
            }
        );
        m.insert(
            *CKBTC_LEDGER,
            Facade {
                wasm_url: format!("https://download.dfinity.systems/ic/{IC_REV}/canisters/ic-icrc1-ledger.wasm.gz"),
                candid_url: format!("https://raw.githubusercontent.com/dfinity/ic/{IC_REV}/rs/ledger_suite/icrc1/ledger/ledger.did"),
                dependencies:vec![],
                init_guide: r#"
1. Create a 'minter' identity: dfx identity new minter
2. Run the following multi-line command:

dfx deps init mxzaz-hqaaa-aaaar-qaada-cai --argument "(variant {
    Init = record {
        minting_account = record { owner = principal \"$(dfx --identity minter identity get-principal)\"; };
        transfer_fee = 10;
        token_symbol = \"ckBTC\";
        token_name = \"ckBTC\";
        metadata = vec {};
        initial_balances = vec {};
        max_memo_length = opt 80;
        archive_options = record {
            num_blocks_to_archive = 1000;
            trigger_threshold = 2000;
            max_message_size_bytes = null;
            cycles_for_archive_creation = opt 100_000_000_000_000;
            node_max_memory_size_bytes = opt 3_221_225_472;
            controller_id = principal \"2vxsx-fae\"
        }
    }
})"
"#.to_string(),
            }
        );
        m.insert(
            *CKETH_LEDGER,
            Facade {
                wasm_url: format!("https://download.dfinity.systems/ic/{IC_REV}/canisters/ic-icrc1-ledger-u256.wasm.gz"),
                candid_url: format!("https://raw.githubusercontent.com/dfinity/ic/{IC_REV}/rs/ledger_suite/icrc1/ledger/ledger.did"),
                dependencies:vec![],
                init_guide: r#"
1. Create a 'minter' identity: dfx identity new minter
2. Run the following multi-line command:

dfx deps init ss2fx-dyaaa-aaaar-qacoq-cai --argument "(variant {
    Init = record {
        minting_account = record { owner = principal \"$(dfx --identity minter identity get-principal)\"; };
        decimals = opt 18;
        max_memo_length = opt 80;
        transfer_fee = 2_000_000_000_000;
        token_symbol = \"ckETH\";
        token_name = \"ckETH\";
        feature_flags = opt record { icrc2 = true };
        metadata = vec {};
        initial_balances = vec {};
        archive_options = record {
            num_blocks_to_archive = 1000;
            trigger_threshold = 2000;
            max_message_size_bytes = null;
            cycles_for_archive_creation = opt 100_000_000_000_000;
            node_max_memory_size_bytes = opt 3_221_225_472;
            controller_id = principal \"2vxsx-fae\"
        }
    }
})"
"#.to_string(),
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
