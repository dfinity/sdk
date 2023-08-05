#!/usr/bin/env python3

import json


def get_cargo_audit_ref():
    with open('nix/sources.json') as json_file:
        data = json.load(json_file)
        return data[ 'advisory-db']['rev']

matrix = {
    'rust': [ '1.71.1' ],
    'os': [ 'macos-latest', 'ubuntu-latest' ],
    'cargo-audit': [ '0.17.4' ],
    'advisory-db-rev': [ get_cargo_audit_ref() ]
}

print(json.dumps(matrix))