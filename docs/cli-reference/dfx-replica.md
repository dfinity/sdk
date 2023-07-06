# dfx replica

> **NOTE**: The replica command has been removed. Please use the [dfx start](./dfx-start.md) command instead. If you have a good reason to use the replica command, please contribute to the [discussion](https://github.com/dfinity/sdk/discussions/3163).

Use the `dfx replica` command to start a local canister execution environment (without a web server). This command enables you to deploy canisters locally and to test your dapps during development.

By default, all local dfx projects will use a single shared local canister execution environment, and you can run `dfx replica` from any directory.  See [Local Server Configuration](#local-server-configuration) and [Project-Specific Local Networks](dfx-start.md#project-specific-local-networks) for exceptions.

## Basic usage

``` bash
dfx replica [option] [flag]
```

## Flags

You can use the following optional flags with the `dfx replica` command.

| Flag              | Description                                                                                   |
|-------------------|-----------------------------------------------------------------------------------------------|
| `--emulator`      | Starts the [IC reference emulator](https://github.com/dfinity/ic-hs) rather than the replica. |
| `--enable-bitcoin`| Enables bitcoin integration.                                                                  |
| `--enable-canister-http` | Enables canister HTTP requests. (deprecated: now enabled by default)                   |

## Options

You can use the following option with the `dfx replica` command.

| Option                    | Description                                                                   |
|---------------------------|-------------------------------------------------------------------------------|
| `--port port`             | Specifies the port the local canister execution environment should listen to. |
| `--bitcoin-node host:port` | Specifies the address of a bitcoind node.  Implies `--enable-bitcoin`. |
| `--artificial-delay milliseconds` | Specifies the delay that an update call should incur. Default: 600ms  |

## Examples

You can start the local canister execution environment by running the following command:

``` bash
dfx replica
```
