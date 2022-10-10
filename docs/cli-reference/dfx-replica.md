# dfx replica

Use the `dfx replica` command to start a local canister execution environment (without a web server). This command enables you to deploy canisters locally and to test your dapps during development.

By default, all local dfx projects will use a single shared local canister execution environment, and you can run `dfx replica` from any directory.  See [Local Server Configuration](#local-server-configuration) and [Project-Specific Local Networks](dfx-start.md#project-specific-local-networks) for exceptions.

## Basic usage

``` bash
dfx replica [option] [flag]
```

## Flags

You can use the following optional flags with the `dfx replica` command.

| Flag              | Description                                                          |
|-------------------|----------------------------------------------------------------------|
| `-h`, `--help`    | Displays usage information.                                          |
| `-V`, `--version` | Displays version information.                                        |
| `--enable-bitcoin`| Enables bitcoin integration.                                         |
| `--enable-canister-http` | Enables canister HTTP requests. (deprecated: now enabled by default) |

## Options

You can use the following option with the `dfx replica` command.

| Option                    | Description                                                                   |
|---------------------------|-------------------------------------------------------------------------------|
| `--port port`             | Specifies the port the local canister execution environment should listen to. |
| `--bitcoin-node host:port` | Specifies the address of a bitcoind node.  Implies `--enable-bitcoin`. |

## Examples

You can start the local canister execution environment by running the following command:

``` bash
dfx replica
```
