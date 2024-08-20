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
    "backend": ["pocketic", "replica"],
    "os": ["macos-12", "ubuntu-20.04"],
    "iterations": [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
        17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
        33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48,
        49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64
    ],
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
