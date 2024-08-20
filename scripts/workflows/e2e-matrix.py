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
    "test": [ "dfx/canister_http", "dfx/bitcoin"],
    "iterations": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20],
    "backend": ["pocketic", "replica"],
    "os": ["macos-12", "ubuntu-20.04"],
    "exclude": [
        {
            "backend": "pocketic",
            "test": "dfx/bitcoin"
        },
        {
            "backend": "pocketic",
            "test": "dfx/canister_http"
        },
        {
            "backend": "pocketic",
            "test": "dfx/canister_logs"
        }
    ]
}

print(json.dumps(matrix))
