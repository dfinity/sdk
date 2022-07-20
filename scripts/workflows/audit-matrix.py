#!/usr/bin/env python3

import json


def get_cargo_audit_ref():
    with open('nix/sources.json') as json_file:
        data = json.load(json_file)
        return data[ 'advisory-db']['rev']

matrix = {
    'rust': [ '1.60.0' ],
    'os': [ 'macos-latest', 'ubuntu-latest' ],
    'cargo-audit': [ '0.15.2' ],
    'advisory-db-rev': [ get_cargo_audit_ref() ]
}

print(json.dumps(matrix))