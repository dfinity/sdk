#!/usr/bin/env python3

import json
import glob
import os


def test_scripts(prefix):
    all = os.listdir('e2e/tests-{}'.format(prefix))
    bash = filter(lambda filename: filename.endswith('.bash'), all)
    tests = list(map(lambda filename: '{}/{}'.format(prefix, filename[:-5]), bash))
    return tests


test = sorted(test_scripts('dfx') + test_scripts('replica'))

matrix = {
    'test': test,
    'backend': [ 'ic-ref', 'replica' ],
    'os': [ 'macos-10.15', 'macos-11', 'ubuntu-20.04' ],
    'rust': [ '1.52.1' ]
}

print(json.dumps(matrix))
