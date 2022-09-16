#!/usr/bin/env python3

import json
import os


def test_scripts(prefix):
    all = os.listdir("e2e/tests-{}".format(prefix))
    bash = filter(lambda filename: filename.endswith(".bash"), all)
    tests = list(map(lambda filename: "{}/{}".format(prefix, filename[:-5]), bash))
    return tests


test = sorted(test_scripts("dfx") + test_scripts("replica") + test_scripts("icx-asset"))

matrix = {
    "test": test,
    "backend": ["ic-ref", "replica"],
    "os": ["macos-11", "ubuntu-20.04"],
    "rust": ["1.60.0"],
    "exclude": [
        {"backend": "ic-ref", "test": "dfx/bitcoin"},
        {"backend": "ic-ref", "test": "dfx/canister_http"},
        {"backend": "ic-ref", "test": "dfx/dfx_install"},
        {"backend": "ic-ref", "test": "dfx/leak"},
        {"backend": "ic-ref", "test": "dfx/ledger"},
        {"backend": "ic-ref", "test": "dfx/new"},
        {"backend": "ic-ref", "test": "dfx/print"},
        {"backend": "ic-ref", "test": "dfx/signals"},
    ],
}

print(json.dumps(matrix))
