#!/usr/bin/env python3

import json
import os

# Only run these tests on macOS
MACOS_TESTS = ["dfx/bitcoin", "dfx/canister_http_adapter", "dfx/start"]

# Run these tests in serial
SERIAL_TESTS = ["dfx/start", "dfx/bitcoin", "dfx/cycles-ledger", "dfx/ledger", "dfx/serial_misc"]

def test_scripts(prefix):
    all_files = os.listdir(f"e2e/tests-{prefix}")
    bash_files = filter(lambda f: f.endswith(".bash"), all_files)
    return [f"{prefix}/{filename[:-5]}" for filename in bash_files]

all_tests = sorted(
    test_scripts("dfx") +
    test_scripts("replica") +
    test_scripts("icx-asset")
)

include = []

for test in all_tests:
    serial = test in SERIAL_TESTS
    # Ubuntu: run everything
    include.append({
        "test": test,
        "os": "ubuntu-22.04",
        "serial": serial,
    })

    # macOS: only run selected tests
    if test in MACOS_TESTS:
        include.append({
            "test": test,
            "os": "macos-13",
            "serial": serial,
        })

matrix = {
    "include": include,
}

print(json.dumps(matrix))
