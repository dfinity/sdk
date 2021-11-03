#!/usr/bin/env python3

import json
import glob
import os


def test_scripts(prefix):
    all = os.listdir('e2e/tests-{}'.format(prefix))
    bash = filter(lambda filename: filename.endswith('.bash'), all)
    tests = list(map(lambda filename: '{}/{}'.format(prefix, filename[:-5]), bash))
    return tests


test = test_scripts('dfx') + test_scripts('replica')
# test = [
#     'dfx/assetscanister',
#     'dfx/base',
#     'dfx/basic-project',
#     'dfx/bootstrap',
#     'dfx/build',
#     'dfx/build_granular',
#     'dfx/call',
#     'dfx/candid_ui',
#     'dfx/certificate',
#     'dfx/certified_info',
#     'dfx/config',
#     'dfx/create',
#     'dfx/dfx_install',
#     'dfx/frontend',
#     'dfx/id',
#     'dfx/identity',
#     'dfx/identity_command',
#     'dfx/install',
#     'dfx/leak',
#     'dfx/network',
#     'dfx/new',
#     'dfx/packtool',
#     'dfx/ping',
#     'dfx/print',
#     'dfx/provider',
#     'dfx/request_status',
#     'dfx/secp256k1',
#     'dfx/sign_send',
#     'dfx/signals',
#     'dfx/start',
#     'dfx/update_settings',
#     'dfx/upgrade',
#     'dfx/usage',
#     'dfx/usage_env',
#     'dfx/wallet',
#     'replica/deploy',
#     'replica/lifecycle'
# ]

matrix = {
    'test': test,
    'backend': [ 'ic-ref', 'replica' ],
    'os': [ 'macos-latest', 'ubuntu-latest' ],
    'rust': [ '1.52.1' ]
}

print(json.dumps(matrix))
