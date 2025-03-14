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
    "backend": ["pocketic", "replica"],
    "os": ["macos-13-large", "ubuntu-22.04"],
    "exclude": [
        {
            "backend": "pocketic",
            "test": "dfx/canister_http_adapter"
        }
    ]
}

print(json.dumps(matrix))
