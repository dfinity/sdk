# Extension-Defined Canister Types

## Overview

An extension can define a canister type.

# Specification

The `canister_type` field in an extension's `extension.json` defines the
characteristics of the canister type.  It has the following fields:

| Field | Type | Description                                      |
|-------|------|--------------------------------------------------|
| `defaults` | Object | Default values for canister fields.       |
| `evaluation_order` | Array | Default fields to evaluate first.  |

The `canister_type.defaults` field is an object that defines canister properties,
as if they were found in dfx.json.  Any fields present in dfx.json
override those found in the extension-defined canister type.

The `metadata` and `tech_stack` fields have special handling.

All elements defined in the `metadata` array in the canister type are appended
to the `metadata` array found in dfx.json. This has the effect that any
metadata specified in dfx.json will take precedence over those that the
extension defines.

If the `tech_stack` field is present in both extension.json and dfx.json,
then dfx merges the two together.  Individual items found in dfx.json
will take precedence over those found in extension.json.

## Handlebar replacement

dfx will perform [handlebars] string replacement on every string field in the
canister type definition. The following data are available for replacement:

| Handlebar              | Description                                                                                       |
|------------------------|---------------------------------------------------------------------------------------------------|
| `{{canister_name}}`    | The name of the canister.                                                                         |
| `{{canister.<field>}}` | Any field from the canister definition in dfx.json, or `canister_type.defaults` in extension.json |

# Examples

Suppose a fictional extension called `addyr` defined a canister type in its
extension.json as follows:
```json
{
  "name": "addyr",
  "canister_type": {
    "defaults": {
      "build": "python -m addyr {{canister_name}} {{canister.main}} {{canister.candid}}",
      "gzip": true,
      "post_install": ".addyr/{{canister_name}}/post_install.sh",
      "wasm": ".addyr/{{canister_name}}/{{canister_name}}.wasm"
    }
  }
}
```

And dfx.json contained this canister definition:
```json
{
  "canisters": {
    "backend": {
      "type": "addyr",
      "candid": "src/hello_backend/hello_backend.did",
      "main": "src/hello_backend/src/main.py"
    }
  }
}
```
This would be treated as if dfx.json defined the following custom canister:
```json
{
  "canisters": {
    "hello_backend": {
      "build": "python -m addyr hello_backend src/hello_backend/src/main.py src/hello_backend/hello_backend.did",
      "candid": "src/hello_backend/hello_backend.did",
      "gzip": true,
      "post_install": ".addyr/hello_backend/post_install.sh",
      "type": "custom",
      "wasm": ".addyr/hello_backend/hello_backend.wasm"
    }
  }
}
```

[handlebars]: https://handlebarsjs.com/
