# Pull Dependencies

## Overview

The interoperability of canisters on the Internet Computer (IC) is an important feature. 

`dfx` provides a consistent developer workflow for integrating third-party canisters.

A service provider prepares the canister to be `pullable` and deploys it on the IC mainnet.

A service consumer then can pull dependencies directly from mainnet and easily deploy them on a local replica.

This document describes the workflow and explains what happens behind the scenes.

## Service Provider Workflow

Below is an example provider `dfx.json` which has a `pullable` "service" canister:

```json
{
    "canisters": {
        "service": {
            "type": "motoko",
            "main": "src/main.mo",
            "pullable": {
                "wasm_url": "http://example.com/a.wasm",
                "wasm_hash": "d180f1e232bafcee7d4879d8a2260ee7bcf9a20c241468d0e9cf4aa15ef8f312",
                "dependencies": [
                    "yofga-2qaaa-aaaaa-aabsq-cai"
                ],
                "init_guide": "A natural number, e.g. 10."
            }
        }
    }
}
```

The `pullable` object will be serialized as a part of the [`dfx` metadata](canister-metadata.md#dfx) and attached to the wasm.

Let's go through the properties of the `pullable` object.

### `wasm_url`

A URL to download canister wasm module which will be deployed locally.

### `wasm_hash` 

SHA256 hash of the wasm module located at `wasm_url`.

This field is optional.

In most cases, the wasm module at `wasm_url` will be the same as the onchain wasm module. This means that dfx can read the state tree to obtain and verify the module hash.

In other cases, the wasm module at `wasm_url` is not the same as the onchain wasm module. For example, the Internet Identity canister provides Development flavor to be integrated locally. In these cases, `wasm_hash` provides the expected hash, and dfx verifies the downloaded wasm against this.

### `wasm_hash_url` 

A URL to get the SHA256 hash of the wasm module located at `wasm_url`.

The content of this URL can be the SHA256 hash only.

It can also be the output of `shasum` or `sha256sum` which contains the hash and the file name.

This field is optional.

Aside from specifying SHA256 hash of the wasm module directly using `wasm_hash`, providers can also specify the hash with this URL. If both are defined, the `wasm_hash_url` field will be ignored.

### `dependencies`

An array of Canister IDs (`Principal`) of direct dependencies.

### `init_guide`

A message to guide consumers how to initialize the canister.

### `init_arg`

A default initialization argument for the canister that consumers can use.

This field is optional.

## Fetch the metadata

To retrieve the metadata and validate the generated information, execute the following command:

```sh
> dfx canister metadata <canister_name> dfx
```

Please note that the `"pullable"` object is part of the public metadata under the key [`"dfx"`](canister-metadata.md#dfx).

Given that the content is in JSON format, you can utilize tools such as `jq` to manipulate the output and extract the pertinent information.

```sh
> dfx canister metadata <canister_name> dfx | jq -r ".pullable.wasm_url"
http://example.com/a.wasm
```

## Canister Metadata Requirements

The "production" canister running on the mainnet should have public `dfx` metadata.

The canister wasm downloaded from `wasm_url` should have the following [metadata](canister-metadata.md#canister-metadata-standard) (public or private):

- `candid:service`
- `candid:args`
- `dfx`

All metadata sections are handled by `dfx` during canister building.

## Service Consumer Workflow

### 1. Declare "pull" dependencies in `dfx.json`

Below is an example `dfx.json` in which the service consumer is developing the "app" canister which has two pull dependencies:

- "dep_b" which has canister ID of "yhgn4-myaaa-aaaaa-aabta-cai" on mainnet.
- "dep_c" which has canister ID of "yahli-baaaa-aaaaa-aabtq-cai" on mainnet.

```json
{
    "canisters": {
        "app": {
            "type": "motoko",
            "main": "src/main.mo",
            "dependencies": [
                "dep_b", "dep_c"
            ]
        },
        "dep_b": {
            "type": "pull",
            "id": "yhgn4-myaaa-aaaaa-aabta-cai"
        },
        "dep_c": {
            "type": "pull",
            "id": "yahli-baaaa-aaaaa-aabtq-cai"
        }
    }
}
```

### 2. Pull the dependencies using `dfx deps pull`

Running `dfx deps pull` will:

1. resolve the dependency graph by fetching `dependencies` field in `dfx` metadata recursively;
2. fetch the expected hash from `wasm_hash`,  `wasm_hash_url` in `dfx` metadata or canister status call;
3. download wasm of all direct and indirect dependencies from their `wasm_url` into shared cache, skip if cached wasm has match hash;
4. extract `candid:args`, `candid:service`, `dfx` metadata from the downloaded wasm;
5. create `deps/` folder in project root;
6. save `candid:service` of direct dependencies as `deps/candid/<CANISTER_ID>.did`;
7. save `deps/pulled.json` which contains major info of all direct and indirect dependencies;

For the example project, you will find following files in `deps/`:

- `candid/yhgn4-myaaa-aaaaa-aabta-cai.did` and `candid/yahli-baaaa-aaaaa-aabtq-cai.did`: candid files that can be imported by "app";
- `pulled.json` which has following content:

```json
{
  "canisters": {
    "yofga-2qaaa-aaaaa-aabsq-cai": {
      "dependencies": [
        "yofga-2qaaa-aaaaa-aabsq-cai"
      ],
      "wasm_hash": "616af3b750c80787f5f123cf7860206db3bb352ef1efe77afcce4d3ee9f2c7ab",
      "wasm_hash_download": "6f053bb3d53d64409c6bddc9355eea658f3d79d510cf57c587bcc809c804bdea",
      "init_guide": "A natural number, e.g. 10.",
      "init_arg": "10",
      "candid_args": "(nat)",
      "gzip": false
    },
    "yhgn4-myaaa-aaaaa-aabta-cai": {
      "name": "dep_b",
      "dependencies": [
        "yofga-2qaaa-aaaaa-aabsq-cai"
      ],
      "wasm_hash": "5642fc8c6fcc0e975a48c87d8e5f21ad0781cb740e5230647754bde14e7f1569",
      "wasm_hash_download": "5642fc8c6fcc0e975a48c87d8e5f21ad0781cb740e5230647754bde14e7f1569",
      "init_guide": "No init arguments required",
      "init_arg": null,
      "candid_args": "()",
      "gzip": true
    },
    "yahli-baaaa-aaaaa-aabtq-cai": {
      "name": "dep_c",
      "dependencies": [
        "yofga-2qaaa-aaaaa-aabsq-cai"
      ],
      "wasm_hash": "6f053bb3d53d64409c6bddc9355eea658f3d79d510cf57c587bcc809c804bdea",
      "wasm_hash_download": "6f053bb3d53d64409c6bddc9355eea658f3d79d510cf57c587bcc809c804bdea",
      "init_guide": "An optional natural number, e.g. \"(opt 20)\".",
      "init_arg": null,
      "candid_args": "(opt nat)",
      "gzip": false
    }
  }
}
```

There are three dependencies:

- "yhgn4-myaaa-aaaaa-aabta-cai": "dep_b" in `dfx.json`;
- "yahli-baaaa-aaaaa-aabtq-cai": "dep_c" in `dfx.json`;
- "yofga-2qaaa-aaaaa-aabsq-cai": an indirect dependency that both "dep_b" and "dep_c" depend on;

**Note**

- In `pulled.json`, every dependency canister has the `wasm_hash` and `wasm_hash_download` fields.
  - They are likely to be the same which means that the downloaded wasm passed integrity check.
  - They can be different in one major circumstance:
    the canister provider serves a customized wasm at `wasm_url` to be deployed locally.
    But the corresponding `wasm_hash` or `wasm_hash_url` is not provided (or the content is wrong).
    `dfx deps` is designed to accept the mismatch hash and will proceed in the following `dfx deps init/deploy`.
-  `dfx deps pull` connects to the IC mainnet by default (`--network ic`).
You can choose other network as usual, e.g. `--network local`.

### 3. Set init arguments using `dfx deps init`

Running `dfx deps init` will iterate over all dependencies in `pulled.json`, try to set init arguments in the following order:

- For canisters that require no init argument, set empty
- For canisters that do require init arguments:
  - Use `init_arg` in `pulled.json` if it is set
  - use `"(null)"` if the canister's init type has a top-level `opt`

The command will also print the list of dependencies that do require an init argument.

Then running `dfx deps init <CANISTER> --argument <ARGUMENT>` will set the init argument for an individual dependency.

The init arguments will be recorded in `deps/init.json`.

For the example, simply running `dfx deps init` to set init arguments for all three pulled canisters.

- "yofga-2qaaa-aaaaa-aabsq-cai" ("a"): set with `init_arg`;
- "yhgn4-myaaa-aaaaa-aabta-cai" ("dep_b"): requires no argument, set empty;
- "yahli-baaaa-aaaaa-aabtq-cai" ("dep_c"): init type `(opt nat)` which has a top-level `opt`, set `"(null)"`;

The init arguments can be overwritten:

```
> dfx deps init yofga-2qaaa-aaaaa-aabsq-cai --argument 11
> dfx deps init deps_c --argument "(opt 22)"
```

The generated `init.json` has following content:

```json
{
  "canisters": {
    "yofga-2qaaa-aaaaa-aabsq-cai": {
      "arg_str": "11",
      "arg_raw": "4449444c00017d0b"
    },
    "yhgn4-myaaa-aaaaa-aabta-cai": {
      "arg_str": null,
      "arg_raw": null
    },
    "yahli-baaaa-aaaaa-aabtq-cai": {
      "arg_str": "(opt 22)",
      "arg_raw": "4449444c016e7d01000116"
    }
  }
}
```

### 4. Deploy pull dependencies on local replica using `dfx deps deploy`

Running `dfx deps deploy` will:

1. create the dependencies on the local replica with the same mainnet canister ID;
2. install the downloaded wasm with the init arguments in `init.json`;

You can also specify the name or principal to deploy one particular dependency.

For our example:
```
> dfx deps deploy
Creating canister: yofga-2qaaa-aaaaa-aabsq-cai
Installing canister: yofga-2qaaa-aaaaa-aabsq-cai
Creating canister: yhgn4-myaaa-aaaaa-aabta-cai (dep_b)
Installing canister: yhgn4-myaaa-aaaaa-aabta-cai (dep_b)
Creating canister: yahli-baaaa-aaaaa-aabtq-cai (dep_c)
Installing canister: yahli-baaaa-aaaaa-aabtq-cai (dep_c)
> dfx deps deploy yofga-2qaaa-aaaaa-aabsq-cai
Installing canister: yofga-2qaaa-aaaaa-aabsq-cai
> dfx deps deploy dep_b
Installing canister: yhgn4-myaaa-aaaaa-aabta-cai (dep_b)
```

**Note**

- `dfx deps deploy` always creates the canister with the anonymous identity so that dependencies and application canisters will have different controllers;
- `dfx deps deploy` always installs the canister in "reinstall" mode so that the canister status will be discarded;

## Q&A

### Why download wasm into shared cache instead of a project subfolder?

We don't want to encourage including binary files in version control.

On the Internet Computer, every canister only has one latest version running on mainnet. Service consumers should integrate with that latest version.

So `dfx deps pull` always gets the latest dependencies instead of locking on a particular run.

Every pulled canister has the latest version in the shared cache and can be reused by different projects.

### Should I include `deps/` folder in version control?

Yes.

`deps/` files enable the dependent canister to build and get IDE support.

If the required wasm files are also available in the shared cache, all application and dependencies can be deployed and tested integrally.

Considering a canister developer team:

1.  Dev1 follows the [workflow](#workflow) and include all generated `deps/` files in source control;
2.  Dev2 pulls the branch by Dev1 and runs `dfx deps pull` again
    1.  If the `pulled.json` has no change, then all dependencies are still up to date. Dev2 can `dfx deps deploy` without setting init arguments again;
    2.  If there are changes in `pulled.json`, Dev2 can try `dfx deps deploy` to see if all init arguments are still valid. Then Dev2 run `dfx deps init` if necessary and update source control;

These files also helps CI to detect outdated dependencies.
