# dfx deps

Use the `dfx deps` command with flags and subcommands to pull dependencies from mainnet and deploy locally.

The basic syntax for running `dfx deps` commands is:

``` bash
dfx deps [subcommand] [options]
```

Depending on the `dfx deps` subcommand you specify, additional arguments, options, and flags might apply. For reference information and examples that illustrate using `dfx deps` commands, select an appropriate command.

| Command                      | Description                                    |
| ---------------------------- | ---------------------------------------------- |
| [`pull`](#dfx-deps-pull)     | Pull canisters upon which the project depends. |
| [`init`](#dfx-deps-init)     | Set init arguments for pulled dependencies.    |
| [`deploy`](#dfx-deps-deploy) | Deploy pulled dependencies.                    |

To view usage information for a specific subcommand, specify the subcommand and the `--help` flag. For example, to see usage information for `dfx deps pull`, you can run the following command:

``` bash
dfx deps pull --help
```

## dfx deps pull

Use the `dfx deps pull` command to pull dependencies as defined in `dfx.json`.

### Basic usage

``` bash
dfx deps pull [options]
```

### Arguments

You can specify the following argument for the `dfx deps delete` command.

| Command   | Description                                                     |
| --------- | --------------------------------------------------------------- |
| `network` | Specify the network to pull dependencies from, default is "ic". |

### Examples

You can use the `dfx deps pull` command to pull the dependencies as defined in `dfx.json` from mainnet. It will resolve all indirect dependencies.

``` bash
dfx deps pull
```

For testing, you may want to pull from local replica, then run:

```bash
dfx deps pull --network local
```

## dfx deps init

Use the `dfx deps init` command to set init arguments for pulled dependencies.

### Basic usage

``` bash
dfx deps init [options] [canister]
```

### Examples

You can use the `dfx deps init` command to set empty init arguments for all pulled dependencies.

``` bash
dfx deps init
```

If any of the dependencies require init arguments, the above command will alarm you with their canister ID and names if exist. Then you can specify canister ID or name to set init argument for individual dependency.

```bash
`dfx deps init <CANISTER_ID/NAME> --argument <ARGUMENT> [--argument-type <TYPE>]`
```

The command below set number `1` for canister `dep_a` as the argument type is the default `idl` (candid).

```bash
dfx deps init dep_a --argument 1
```

The command below set the hex-encoded raw bytes for canister `dep_b`.

```bash
dfx deps init dep_b --argument "4449444c00017103616263" --argument-type raw
```

## dfx deps deploy

Use the `dfx deps deploy` command to deploy all dependencies.

### Basic usage

``` bash
dfx deps deploy [flag]
```

### Examples

You can use the `dfx deps deploy` command to deploy dependencies on local replica.

``` bash
dfx deps deploy
```

If some of the dependencies haven't been pulled or set init arguments, the command will fail. And the error message will help you to fix it.
