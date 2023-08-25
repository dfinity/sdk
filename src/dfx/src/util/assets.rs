use crate::lib::error::DfxResult;
use anyhow::{bail, Context};
use fn_error_context::context;
use slog::info;
use std::io::Read;

include!(concat!(env!("OUT_DIR"), "/load_assets.rs"));

pub fn dfinity_logo() -> String {
    let colors = supports_color::on(atty::Stream::Stdout);
    if let Some(colors) = colors {
        //Some terminals, notably MacOS's Terminal.app, do not support Truecolor (RGB-colored characters) properly.
        //Therefore we use xterm256 coloring when the program is running in such a terminal.
        if colors.has_16m {
            return include_str!("../../assets/dfinity-color.aart").to_string();
        } else if colors.has_256 {
            return include_str!("../../assets/dfinity-color-xterm256.aart").to_string();
        }
    }
    include_str!("../../assets/dfinity-nocolor.aart").to_string()
}

#[context("Failed to load wallet wasm.")]
pub fn wallet_wasm(logger: &slog::Logger) -> DfxResult<Vec<u8>> {
    if let Ok(dfx_wallet_wasm) = std::env::var("DFX_WALLET_WASM") {
        info!(logger, "Using wasm at path: {}", dfx_wallet_wasm);
        Ok(dfx_core::fs::read(dfx_wallet_wasm.as_ref())?)
    } else {
        let mut canister_assets =
            wallet_canister().context("Failed to load wallet canister archive.")?;
        for file in canister_assets
            .entries()
            .context("Failed to read wallet canister archive entries.")?
        {
            let mut file = file.context("Failed to read wallet canister archive entry.")?;
            if file
                .header()
                .path()
                .context("Failed to read archive entry path.")?
                .ends_with("wallet.wasm")
            {
                let mut wasm = vec![];
                file.read_to_end(&mut wasm)
                    .context("Failed to read archive entry.")?;
                return Ok(wasm);
            }
        }
        bail!("Failed to find wallet canister archive entry.");
    }
}

#[context("Failed to load assets wasm.")]
pub fn assets_wasm(logger: &slog::Logger) -> DfxResult<Vec<u8>> {
    if let Ok(dfx_assets_wasm) = std::env::var("DFX_ASSETS_WASM") {
        info!(logger, "Using wasm at path: {}", dfx_assets_wasm);
        Ok(dfx_core::fs::read(dfx_assets_wasm.as_ref())?)
    } else {
        let mut canister_assets =
            assetstorage_canister().context("Failed to load asset canister archive.")?;
        for file in canister_assets
            .entries()
            .context("Failed to read asset canister archive entries.")?
        {
            let mut file = file.context("Failed to read asset canister archive entry.")?;
            if file
                .header()
                .path()
                .context("Failed to read archive entry path.")?
                .ends_with("assetstorage.wasm.gz")
            {
                let mut wasm = vec![];
                file.read_to_end(&mut wasm)
                    .context("Failed to read archive entry.")?;
                return Ok(wasm);
            }
        }
        bail!("Failed to find asset canister archive entry.");
    }
}

#[allow(unused)]
#[context("Failed to load bitcoin wasm.")]
pub fn bitcoin_wasm(logger: &slog::Logger) -> DfxResult<Vec<u8>> {
    if let Ok(dfx_assets_wasm) = std::env::var("DFX_BITCOIN_WASM") {
        info!(logger, "Using wasm at path: {}", dfx_assets_wasm);
        Ok(dfx_core::fs::read(dfx_assets_wasm.as_ref())?)
    } else {
        let mut canister_assets =
            btc_canister().context("Failed to load bitcoin canister archive.")?;
        for file in canister_assets
            .entries()
            .context("Failed to read bitcoin canister archive entries.")?
        {
            let mut file = file.context("Failed to read bitcoin canister archive entry.")?;
            if file
                .header()
                .path()
                .context("Failed to read archive entry path.")?
                .ends_with("ic-btc-canister.wasm.gz")
            {
                let mut wasm = vec![];
                file.read_to_end(&mut wasm)
                    .context("Failed to read archive entry.")?;
                return Ok(wasm);
            }
        }
        bail!("Failed to find bitcoin canister archive entry");
    }
}

pub fn management_idl() -> DfxResult<String> {
    // FIXME get idl from replica when it's available
    Ok(r##"
type canister_id = principal;
type wasm_module = blob;

type canister_settings = record {
    controllers : opt vec principal;
    compute_allocation : opt nat;
    memory_allocation : opt nat;
    freezing_threshold : opt nat;
};

type definite_canister_settings = record {
    controllers : vec principal;
    compute_allocation : nat;
    memory_allocation : nat;
    freezing_threshold : nat;
};

type change_origin = variant {
    from_user : record {
    user_id : principal;
    };
    from_canister : record {
    canister_id : principal;
    canister_version : opt nat64;
    };
};

type change_details = variant {
    creation : record {
    controllers : vec principal;
    };
    code_uninstall;
    code_deployment : record {
    mode : variant {install; reinstall; upgrade};
    module_hash : blob;
    };
    controllers_change : record {
    controllers : vec principal;
    };
};

type change = record {
    timestamp_nanos : nat64;
    canister_version : nat64;
    origin : change_origin;
    details : change_details;
};

type http_header = record { name: text; value: text };

type http_response = record {
    status: nat;
    headers: vec http_header;
    body: blob;
};

type ecdsa_curve = variant { secp256k1; };

type satoshi = nat64;

type bitcoin_network = variant {
    mainnet;
    testnet;
};

type bitcoin_address = text;

type block_hash = blob;

type outpoint = record {
    txid : blob;
    vout : nat32
};

type utxo = record {
    outpoint: outpoint;
    value: satoshi;
    height: nat32;
};

type get_utxos_request = record {
    address : bitcoin_address;
    network: bitcoin_network;
    filter: opt variant {
    min_confirmations: nat32;
    page: blob;
    };
};

type get_current_fee_percentiles_request = record {
    network: bitcoin_network;
};

type get_utxos_response = record {
    utxos: vec utxo;
    tip_block_hash: block_hash;
    tip_height: nat32;
    next_page: opt blob;
};

type get_balance_request = record {
    address : bitcoin_address;
    network: bitcoin_network;
    min_confirmations: opt nat32;
};

type send_transaction_request = record {
    transaction: blob;
    network: bitcoin_network;
};

type millisatoshi_per_byte = nat64;

service ic : {
    create_canister : (record {
    settings : opt canister_settings;
    sender_canister_version : opt nat64;
    }) -> (record {canister_id : canister_id});
    update_settings : (record {
    canister_id : principal;
    settings : canister_settings;
    sender_canister_version : opt nat64;
    }) -> ();
    install_code : (record {
    mode : variant {install; reinstall; upgrade};
    canister_id : canister_id;
    wasm_module : wasm_module;
    arg : blob;
    sender_canister_version : opt nat64;
    }) -> ();
    uninstall_code : (record {
    canister_id : canister_id;
    sender_canister_version : opt nat64;
    }) -> ();
    start_canister : (record {canister_id : canister_id}) -> ();
    stop_canister : (record {canister_id : canister_id}) -> ();
    canister_status : (record {canister_id : canister_id}) -> (record {
        status : variant { running; stopping; stopped };
        settings: definite_canister_settings;
        module_hash: opt blob;
        memory_size: nat;
        cycles: nat;
        idle_cycles_burned_per_day: nat;
    });
    canister_info : (record {
        canister_id : canister_id;
        num_requested_changes : opt nat64;
    }) -> (record {
        total_num_changes : nat64;
        recent_changes : vec change;
        module_hash : opt blob;
        controllers : vec principal;
    });
    delete_canister : (record {canister_id : canister_id}) -> ();
    deposit_cycles : (record {canister_id : canister_id}) -> ();
    raw_rand : () -> (blob);
    http_request : (record {
    url : text;
    max_response_bytes: opt nat64;
    method : variant { get; head; post };
    headers: vec http_header;
    body : opt blob;
    transform : opt record {
        function : func (record {response : http_response; context : blob}) -> (http_response) query;
        context : blob
    };
    }) -> (http_response);

    // Threshold ECDSA signature
    ecdsa_public_key : (record {
    canister_id : opt canister_id;
    derivation_path : vec blob;
    key_id : record { curve: ecdsa_curve; name: text };
    }) -> (record { public_key : blob; chain_code : blob; });
    sign_with_ecdsa : (record {
    message_hash : blob;
    derivation_path : vec blob;
    key_id : record { curve: ecdsa_curve; name: text };
    }) -> (record { signature : blob });

    // bitcoin interface
    bitcoin_get_balance: (get_balance_request) -> (satoshi);
    bitcoin_get_utxos: (get_utxos_request) -> (get_utxos_response);
    bitcoin_send_transaction: (send_transaction_request) -> ();
    bitcoin_get_current_fee_percentiles: (get_current_fee_percentiles_request) -> (vec millisatoshi_per_byte);

    // provisional interfaces for the pre-ledger world
    provisional_create_canister_with_cycles : (record {
    amount: opt nat;
    settings : opt canister_settings;
    specified_id: opt canister_id;
    sender_canister_version : opt nat64;
    }) -> (record {canister_id : canister_id});
    provisional_top_up_canister :
    (record { canister_id: canister_id; amount: nat }) -> ();
}
"##.to_string())
}
