#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup
}

teardown() {
  dfx_stop
  standard_teardown
}

@test "dfx deps can facade pull ICP ledger" {
  use_test_specific_cache_root # dfx deps pull will download files to cache

  dfx_new
  jq '.canisters.e2e_project_backend.dependencies=["icp_ledger"]' dfx.json | sponge dfx.json
  jq '.canisters.icp_ledger.type="pull"' dfx.json | sponge dfx.json
  jq '.canisters.icp_ledger.id="ryjl3-tyaaa-aaaaa-aaaba-cai"' dfx.json | sponge dfx.json

  dfx_start
  assert_command dfx deps pull --network local
  assert_contains "Using facade dependencies for canister ryjl3-tyaaa-aaaaa-aaaba-cai."

  dfx identity new --storage-mode plaintext minter
  assert_command_fail dfx deps init icp_ledger
  assert_contains "1. Create a 'minter' identity: dfx identity new minter
2. Run the following multi-line command:"

  assert_command dfx deps init ryjl3-tyaaa-aaaaa-aaaba-cai --argument "(variant { 
    Init = record {
        minting_account = \"$(dfx --identity minter ledger account-id)\";
        initial_values = vec {};
        send_whitelist = vec {};
        transfer_fee = opt record { e8s = 10_000 : nat64; };
        token_symbol = opt \"LICP\";
        token_name = opt \"Local ICP\"; 
    }
})"

  assert_command dfx deps deploy

  # Can mint tokens (transfer from minting_account)
  assert_command dfx --identity minter canister call icp_ledger icrc1_transfer "(
  record {
    to = record {
      owner = principal \"$(dfx --identity default identity get-principal)\";
    };
    amount = 1_000_000 : nat;
  },
)"

  assert_command dfx canister call icp_ledger icrc1_balance_of "(
  record {
    owner = principal \"$(dfx --identity default identity get-principal)\";
  },
)"
  assert_eq "(1_000_000 : nat)"
}

@test "dfx deps can facade pull ckBTC ledger" {
  use_test_specific_cache_root # dfx deps pull will download files to cache

  dfx_new
  jq '.canisters.e2e_project_backend.dependencies=["ckbtc_ledger"]' dfx.json | sponge dfx.json
  jq '.canisters.ckbtc_ledger.type="pull"' dfx.json | sponge dfx.json
  jq '.canisters.ckbtc_ledger.id="mxzaz-hqaaa-aaaar-qaada-cai"' dfx.json | sponge dfx.json

  dfx_start
  assert_command dfx deps pull --network local
  assert_contains "Using facade dependencies for canister mxzaz-hqaaa-aaaar-qaada-cai."

  dfx identity new --storage-mode plaintext minter
  assert_command_fail dfx deps init ckbtc_ledger
  assert_contains "1. Create a 'minter' identity: dfx identity new minter
2. Run the following multi-line command:"

  assert_command dfx deps init mxzaz-hqaaa-aaaar-qaada-cai --argument "(variant {
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

  assert_command dfx deps deploy

  # Can mint tokens (transfer from minting_account)
  assert_command dfx --identity minter canister call ckbtc_ledger icrc1_transfer "(
  record {
    to = record {
      owner = principal \"$(dfx --identity default identity get-principal)\";
    };
    amount = 1_000_000 : nat;
  },
)"

  assert_command dfx canister call ckbtc_ledger icrc1_balance_of "(
  record {
    owner = principal \"$(dfx --identity default identity get-principal)\";
  },
)"
  assert_eq "(1_000_000 : nat)"
}


@test "dfx deps can facade pull ckETH ledger" {
  use_test_specific_cache_root # dfx deps pull will download files to cache

  dfx_new
  jq '.canisters.e2e_project_backend.dependencies=["cketh_ledger"]' dfx.json | sponge dfx.json
  jq '.canisters.cketh_ledger.type="pull"' dfx.json | sponge dfx.json
  jq '.canisters.cketh_ledger.id="ss2fx-dyaaa-aaaar-qacoq-cai"' dfx.json | sponge dfx.json

  dfx_start
  assert_command dfx deps pull --network local
  assert_contains "Using facade dependencies for canister ss2fx-dyaaa-aaaar-qacoq-cai."

  dfx identity new --storage-mode plaintext minter
  assert_command_fail dfx deps init cketh_ledger
  assert_contains "1. Create a 'minter' identity: dfx identity new minter
2. Run the following multi-line command:"

  assert_command dfx deps init ss2fx-dyaaa-aaaar-qacoq-cai --argument "(variant {
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

  assert_command dfx deps deploy

  # Can mint tokens (transfer from minting_account)
  assert_command dfx --identity minter canister call cketh_ledger icrc1_transfer "(
  record {
    to = record {
      owner = principal \"$(dfx --identity default identity get-principal)\";
    };
    amount = 1_000_000 : nat;
  },
)"

  assert_command dfx canister call cketh_ledger icrc1_balance_of "(
  record {
    owner = principal \"$(dfx --identity default identity get-principal)\";
  },
)"
  assert_eq "(1_000_000 : nat)"
}
