# Tech Stack

## Overview

Canister authors can opt in to display the tech stack of the canister.

Providing a standard format of such information makes it easier to build tools like a Canister Explorer.

## JSON schema

`tech_stack` has 5 top optional categories.

- cdk
- language
- lib
- tool
- other

Each category contains an Array of tech stack items.

Each tech stack item must have a `"name"` field and can optionally define some custom fields, e.g. `"version"`.

### Example

```json
{
  "tech_stack": {
    "language": [
      {
        "name": "rust",
        "version": "1.75.0"
      }
    ],
    "cdk": [
      {
        "name": "ic-cdk",
        "version": "0.13.0"
      }
    ],
    "lib": [
      {
        "name": "ic-cdk-timers"
      },
      {
        "name": "ic-stable-structures"
      }
    ],
    "other": [
      {
        "name": "bitcoin"
      }
    ],
    "tool": [
      {
        "name": "dfx"
      }
    ]
  }
}
```

## How to set it in `dfx.json`

The example above was generated from the `dfx.json` configuration below.

```json
{
  "type": "motoko",
  "main": "main.mo",
  "tech_stack": {
    "cdk": [
      {
        "name": "ic-cdk",
        "custom_fields": [
          {
              "field": "version",
              "value": "0.13.0"
          }
      ]
      }
    ],
    "language": [
      {
        "name": "rust",
        "custom_fields": [
          {
            "field": "version",
            "value_command": "rustc --version | cut -d ' ' -f 2"
          }
        ]
      }
    ],
    "lib": [
      {
        "name": "ic-cdk-timers"
      },
      {
        "name": "ic-stable-structures"
      }
    ],
    "tool": [
      {
        "name": "dfx"
      }
    ],
    "other": [
      {
        "name": "bitcoin"
      }
    ]
  }
}
```

In `dfx.json`, the optional `"tech_stack"` object has 5 corresponding categories.

Each category is a JSON array, in which each element defines a tech stack item.

Each item configuration must define a `"name"` and can optionally define `"custom_fields"`.

The `"custom_fields"` is a JSON array, in which each element defines a custom field.

Each custom field must define a `"field"` name.

The value of the field can be defined in two ways:

- `"value"`: This defines the value directly.
- `"value_command"`:
  - This should be a CLI command.
  - The command will be run in the workspace root (the dir contains `dfx.json`). 
  - The stdout should be a valid UTF-8 string.
  - The stdout will be stripped to get the version.
