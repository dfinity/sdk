#!/usr/bin/env python3

import json
import os

# Only run these tests on macOS
MACOS_TESTS = ["dfx/bitcoin", "dfx/canister_http_adapter", "dfx/start"]

# All supported backends
ALL_BACKENDS = ["pocketic", "replica"]

# Skip specific test-backend combinations
EXCLUDE = [
    {
        "backend": "pocketic",
        "test": "dfx/canister_http_adapter"
    }
]

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
    for backend in ALL_BACKENDS:
        if {"backend": backend, "test": test} in EXCLUDE:
            continue

        # Ubuntu: run everything
        include.append({
            "test": test,
            "backend": backend,
            "os": "ubuntu-22.04"
        })

        # macOS: only run selected tests
        if test in MACOS_TESTS:
            include.append({
                "test": test,
                "backend": backend,
                "os": "macos-13"
            })

matrix = {
    "include": include,
}

print(json.dumps(matrix))
