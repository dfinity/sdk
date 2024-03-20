# Tech Stack

## Overview

Canister authors can opt in to display the tech stack of the canister.

The tech stack includes but not limit to the programming languages, CDKs, libraries, tools.

Providing a standard format of such information makes it easier to build tools like a Canister Explorer.

## JSON schema

`tech_stack` is a non-nested JSON object.

Each key-value pair is a tech stack item.

* The key is the name of the tech stack item.
* The value is the optional version of the tech stack item. It can be a string or `null`.

### Example

```json
{
  "tech_stack": {
    "rust": "1.76.0",
    "ic-cdk": "0.13.0",
    "wasm-tools": null
  }
}
```

## How to set it in `dfx.json`

The example above was generated from the `dfx.json` configuration below.

```json
{
  "canisters": {
    "foo": {
      "type": "custom",
      "tech_stack": [
        {
          "name": "ic-cdk",
          "version": "0.13.0"
        },
        {
          "name": "rust",
          "version_command": "rustc --version | cut -d \" \" -f 2"
        },
        {
          "name": "wasm-tools"
        }
      ]  
    }
  }
}
```

In `dfx.json`, the optional `"tech_stack"` field is a JSON array.

Each element corresponds to one tech stack item.

An element may be defined in three forms:

* `"name"` and `"version"`:
  * Their value directly map to the key/value above.
* `"name"` and `"version"`:
  * The value of `"version_command"` should be a CLI command.
  * The command will be run in workspace root (dir contains `dfx.json`).
  * The stdout should be a valid UTF-8 string.
  * The stdout will be stripped to get the version.
* `"name"` only:
  * The version will be `null`.

Defining both `"version"` and `"version_command"` will result in an error.
