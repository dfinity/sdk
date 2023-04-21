# Pull Dependencies

## Overview

The interoperability of canisters on the Internet Computer (IC) is an important feature. 

`dfx` provides a consistent developer workflow for integrating thrid party canisters.

A service provider attaches [necessary metadata](canister-metadata.md) to the canister wasm.

A service consumer then can pull dependencies directly from the IC mainnet and easily deploy them on a local replica.

This document describes the consumer workflow and explains what happens behind the scene.

## Workflow

### 1. Declare "pull" dependencies in `dfx.json`

Below is an example `dfx.json` of a project, the service consumer is developing the "app" canister which has two pull dependencies:

- "dep_b" which has canister ID of "yhgn4-myaaa-aaaaa-aabta-cai" on the mainnet.
- "dep_c" which has canister ID of "yahli-baaaa-aaaaa-aabtq-cai" on the mainnet.

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

Running `dfx deps pull --network ic` will:

1. resolve the dependency graph by fetching `dfx:deps` metadata recuresively;
2. download wasm of all direct and indirect dependencies from `dfx:wasm_url` into shared cache;
3. hash check the downloaded wasm against `dfx:wasm_hash` metadata or the hash of the mainnet running canister;
4. extract `candid:args`, `candid:service`, `dfx:init` from the downloaded wasm;
5. create `deps/` folder in project root;
6. save `candid:service` of direct dependencies as `deps/<NAME>.did`;
7. save `deps/pulled.json` which contains major info of all direct and indirect dependencies;

For the example project, you will find following files in `deps/`:

- `dep_b.did` and `dep_c.did`: candid files that can be imported by "app";
- `pulled.json` which has follwing content:

```
{
  "canisters": {
    "yofga-2qaaa-aaaaa-aabsq-cai": {
      "name": null,
      "deps": [],
      "wasm_url": "exmaple.com/a.wasm",
      "wasm_hash": "e9b8ba2ad28fa1403cf6e776db531cdd6009a8e5cac2b1097d09bfc65163d56f",
      "dfx_init": "Nat",
      "candid_args": "(nat)"
    },
    "yhgn4-myaaa-aaaaa-aabta-cai": {
      "name": "dep_b",
      "deps": [
        "yofga-2qaaa-aaaaa-aabsq-cai"
      ],
      "wasm_url": "exmaple.com/b.wasm",
      "wasm_hash": "f607c30727b0ee81317fc4547a8da3cda9bb9621f5d0740806ef973af5b479a2",
      "dfx_init": null,
      "candid_args": "()"
    },
    "yahli-baaaa-aaaaa-aabtq-cai": {
      "name": "dep_c",
      "deps": [
        "yofga-2qaaa-aaaaa-aabsq-cai"
      ],
      "wasm_url": "exmaple.com/c.wasm",
      "wasm_hash": "016df9800dc5760785646373bcb6e6bb530fc17f844600991a098ef4d486cf0b",
      "dfx_init": "Nat",
      "candid_args": "(nat)"
    }
  }
}
```

There are three dependencies:

- "yhgn4-myaaa-aaaaa-aabta-cai": "dep_b" in `dfx.json`;
- "yahli-baaaa-aaaaa-aabtq-cai": "dep_c" in `dfx.json`;
- "yofga-2qaaa-aaaaa-aabsq-cai": a indirect dependency that both "dep_b" and "dep_c" depend on;

### 3. Set init arguments using `dfx deps init`

Running `dfx deps init` will iterate all dependencies in `pulled.json`, set empty argument for the ones need no init argument and print the list of dependencies that require init argument.

Then running `dfx deps init <CANISTER> --argument <ARGUMENT>` to set init argument for individual dependency.

The init arguments will be recorded in `deps/init.json`.

For our example, we should run:

```
> dfx deps init
WARN: The following canister(s) require an init argument. Please run `dfx deps init <PRINCIPAL>` to set them individually:
yofga-2qaaa-aaaaa-aabsq-cai
yahli-baaaa-aaaaa-aabtq-cai (dep_c)
> dfx deps init yofga-2qaaa-aaaaa-aabsq-cai --argument 10
> dfx deps init deps_c --argument 20
```

The generated `init.json` has following content:

```json
{
  "canisters": {
    "yofga-2qaaa-aaaaa-aabsq-cai": {
      "arg_str": "10",
      "arg_raw": "4449444c00017d0a"
    },
    "yhgn4-myaaa-aaaaa-aabta-cai": {
      "arg_str": null,
      "arg_raw": null
    },
    "yahli-baaaa-aaaaa-aabtq-cai": {
      "arg_str": "20",
      "arg_raw": "4449444c00017d14"
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
Creating canister: yofga-2qaaa-aaaaa-aabsq-cai
Installing canister: yofga-2qaaa-aaaaa-aabsq-cai
> dfx deps deploy dep_b
Creating canister: yhgn4-myaaa-aaaaa-aabta-cai (dep_b)
Installing canister: yhgn4-myaaa-aaaaa-aabta-cai (dep_b)
```

**Note**

- `dfx deps deploy` always create the canister with the anonymous identity so that dependencies will have different controller with the application canisters;
- `dfx deps deploy` always install the canister in "reinstall" mode so that the canister status will be discarded;

## Q&A

### Why download wasm into shared cache instead of a project subfolder?

We don't want to encourage developers to include binary files in version control.

On IC, every canister only has one latest version running on the mainnet. And service consumer should only integrate with that latest version.

So `dfx deps pull` always get the latest dependencies instead of locking on a particular run.

Every pulled canister has one latest version in the shared cache and can be reused by different projects.

### Should I include `deps/` folder in version control?

Yes.

`deps/` files enable the dependent canister to build and get IDE suppport.

If the required wasm files are also available in the shared cache, all application and dependencies can be deployed and tested integrately.

Considering a canister developer team:

1.  Dev1 follows the [workflow](#workflow) and include all generated `deps/` files in source control;
2.  Dev2 pull the branch by Dev1 and run `dfx deps pull` again
    1.  If the `pulled.json` has no change, then all dependencies are still up to date. Dev2 can `dfx deps deploy` without setting init arguments again;
    2.  If there are changes in `pulled.json`, Dev2 can try `dfx deps deploy` to see if all init arguments are still valid. Then Dev2 run `dfx deps init` if necessary and update source control;

These files also helps CI to detect outdated dependencies.
