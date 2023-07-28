#!/usr/bin/env bash

FILE="$1"
DFX_VERSION="$2"
NEW_HASH="$3"

awk -v dfx_version="$DFX_VERSION" -v new_hash="$NEW_HASH" '
BEGIN { OFS="" }

/const WHITELISTED_WASMS: \[&str; [0-9]+\] = \[/ {
    split($0, arr, ";")
    split(arr[2], subarr, "]")
    num = subarr[1] + 1
    print arr[1] "; " num "]" subarr[2]
    print "    \"", new_hash, "\", // dfx ", dfx_version, " frontend canister"
    next
}
{ print $0 }' "$FILE" > tmpfile && mv tmpfile "$FILE"
