# Environment variables

You can configure certain properties for your SDK execution environment using environment variables.

This section lists the environment variables that are currently supported with examples of how to use them. In most cases, you can set environment variables for a session by executing an command in the terminal or by adding a line similar to the following to your `.profile` file:

    export DFX_NETWORK=ic

## CANISTER_CANDID_PATH\_{canister.name}

Use environment variables with the `CANISTER_CANDID_PATH` prefix to reference the path to the Candid description file for the canisters that are listed as dependencies in the `dfx.json` file for your project.

For example, if you have a `whoami_assets` canister that lists `whoami` under the `dependencies` key, you could use the `CANISTER_CANDID_PATH_whoami_assets` environment variable to refer to the location of the `whoami.did` file, which for local development might be:

    $PROJECT_ROOT/.dfx/local/canisters/whoami/whoami.did

## CANISTER_ID\_{canister.name}

Use environment variables with the `CANISTER_ID` prefix to reference the canister identifier for each canister in the `dfx.json` file for your project.

For example, if you have a `linkedup` project that consists of the `linkedup` and `connectd` canisters, you could use the `CANISTER_ID_linkedup` and `CANISTER_ID_connectd` environment variables to refer to the canister identifiers—for example `ryjl3-tyaaa-aaaaa-aaaba-cai` and `rrkah-fqaaa-aaaaa-aaaaq-cai`—created for your project.

## DFX_CONFIG_ROOT

Use the `DFX_CONFIG_ROOT` environment variable to specify a different location for storing the `.cache` and `.config` subdirectories for `dfx`.

By default, the `.cache` and `.config` directories are located in the home directory for your development environment. For example, on macOS the default location is in the `/Users/<YOUR-USER-NAME>` directory. Use the `DFX_CONFIG_ROOT` environment variable to specify a different location for these directories.

    DFX_CONFIG_ROOT=~/ic-root

## DFX_INSTALLATION_ROOT

Use the `DFX_INSTALLATION_ROOT` environment variable to specify a different location for the `dfx` binary if you are not using the default location for your operating system.

The `.cache/dfinity/uninstall.sh` script uses this environment variable to identify the root directory for your SDK installation.

## DFX_VERSION

Use the `DFX_VERSION` environment variable to identify a specific version of the SDK that you want to install.

    DFX_VERSION=0.10.0 sh -ci "$(curl -fsSL https://smartcontracts.org/install.sh)"
