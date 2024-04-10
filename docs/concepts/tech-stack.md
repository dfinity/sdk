# Tech Stack

## Overview

Canister authors can opt in to display the tech stack of the canister.

Providing a standard format of such information makes it easier to build tools like a Canister Explorer.

## JSON schema

`tech_stack` has 5 top-level optional categories.

- cdk
- language
- lib
- tool
- other

Each category is a map keyed by the names of tech stack items, where each value is a map containing optional fields.

### Example

```json
{
  "tech_stack": {
    "language": {
      "rust": {
        "version": "1.75.0"
      }
    },
    "cdk": {
      "ic-cdk": {
        "version": "0.13.0"
      }
    },
    "lib": {
      "ic-cdk-timers": {},
      "ic-stable-structures": {}
    },
    "other": {
      "bitcoin": {
        "address": "bcrt1qfe264m0ycx2vcqvqyhs0gpxk6tw8ug6hqeps2d"
      }
    },
    "tool": {
      "dfx": {}
    }
  }
}
```

## Configuration in `dfx.json`

While the only way to configure this is in `dfx.json`, we don't envision that canister developers will define these values in `dfx.json` by themselves.

Upcoming work will enable CDK providers to set `tech_stack` fields for their users. Please check the Q&A below for more explanation.

The example above was generated from the `dfx.json` configuration below.

```json
{
  "canisters": {
    "canister_foo": {
      "type": "custom",
      "tech_stack": {
        "cdk": {
          "ic-cdk": {
            "version": "0.13.0"
          }
        },
        "language": {
          "rust": {
            "version": "$(rustc --version | cut -d ' ' -f 2)"
          }
        },
        "lib": {
          "ic-cdk-timers": {},
          "ic-stable-structures": {}
        },
        "tool": {
          "dfx": {}
        },
        "other": {
          "bitcoin": {
            "address": "bcrt1qfe264m0ycx2vcqvqyhs0gpxk6tw8ug6hqeps2d"
          }
        }
      }
    }
  }
}
```

The `"tech_stack"` object in `dfx.json` is almost the same as the generated metadata.

The only difference is that the `language->rust->version` field is `"$(rustc --version | cut -d ' ' -f 2)"` instead of `"1.75.0"`.

Besides directly setting the value of custom fields, it's also possible to obtain the value by executing a command.

If the content of a custom field value begins with the prefix `$(` and ends with the postfix `)`, the inner text will be interpreted as a command.

- The command will be executed in the workspace root directory, which contains the `dfx.json` file.
- The stdout should be a valid UTF-8 string.
- The field value will be obtained by trimming the stdout, removing any leading and trailing whitespace.

## Q&A

### Who should set `tech_stack`?

In the near future, CDK will be able to set `tech_stack` without requiring extra configuration in `dfx.json`.

Currently, `dfx` sets `tech_stack` for Rust and Motoko canisters if they don't define `tech_stack` explicitly in `dfx.json`.

For Azle and Kybra projects created with `dfx new`, the corresponding `tech_stack` configuration will be added `dfx.json` by default.

Canister developers can always add/overwrite/remove the `tech_stack` fields set by CDK.
