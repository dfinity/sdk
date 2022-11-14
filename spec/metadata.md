# Canister Metadata Standard

This document specifies a canister metadata standard for dfx usage.

All metadata in this standard are public.
Keys are prefixed with `dfx:` to avoid conflict with other metadata usage.
Values should be valid UTF-8 text.

## `dfx:wasm_url`

A URL to download canister Wasm module which will be deployed locally.

## `dfx:deps`

A list of name:ID pairs of direct dependencies separated by semicolon.

## `dfx:init`

A message to guide consumers how to initialize the canister.
