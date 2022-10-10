# dfx start

Use the `dfx start` command to start a local canister execution environment and web server processes. This command enables you to deploy canisters to the local canister execution environment to test your dapps during development.

By default, all local dfx projects will use this single local canister execution environment, and you can run `dfx start` and `dfx stop` from any directory.  See [Local Server Configuration](#local-server-configuration) and [Project-Specific Local Networks](#project-specific-local-networks) below for exceptions.

## Basic usage

``` bash
dfx start [option] [flag]
```

## Flags

You can use the following optional flags with the `dfx start` command.

| Flag              | Description                                                                                                                                                                                                                                  |
|-------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--background`           | Starts the local canister execution environment and web server processes in the background and waits for a reply before returning to the shell.                                                                                              |
| `--clean`                | Starts the local canister execution environment and web server processes in a clean state by removing checkpoints from your project cache. You can use this flag to set your project cache to a new state when troubleshooting or debugging. |
| `--enable-bitcoin` | Enables bitcoin integration.                                                                                                                                                                                                                 |
| `--enable-canister-http` | Enables canister HTTP requests. (deprecated: now enabled by default)                                                                                                                                                        |
| `-h`, `--help`           | Displays usage information.                                                                                                                                                                                                                  |
| `-V`, `--version`        | Displays version information.                                                                                                                                                                                                                |

## Options

You can use the following option with the `dfx start` command.

| Option        | Description                                                                                                                                                                                                         |
|---------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--host host` | Specifies the host interface IP address and port number to bind the frontend to. The default for the local shared network is `127.0.0.1:4943`, while the default for a project-specific network is '127.0.0.1:8000'. |
| `--bitcoin-node host:port` | Specifies the address of a bitcoind node. Implies `--enable-bitcoin`.                                                                                                                                               |

## Examples

You can start the local canister execution environment and web server processes in the current shell by running the following command:

``` bash
dfx start
```

If you start the local canister execution environment in the current shell, you need to open a new terminal shell to run additional commands. Alternatively, you can start the local canister execution environment in the background by running the following command:

``` bash
dfx start --background
```

If you run the local canister execution environment in the background, however, be sure to stop the local canister execution environment before uninstalling or reinstalling the `dfx` execution environment by running the following command:

``` bash
dfx stop
```

You can view the current process identifier (`pid`) for the local canister execution environment process started by `dfx` by running the following command:

``` bash
more .dfx/pid
```

## Local Server Configuration

### The Shared Local Network

By default, `dfx start` manages a single replica that is independent of any given local project.  Running `dfx deploy` and other commands will manage canisters on this single local network, in the same way that `dfx deploy --network ic` deploys separate projects to mainnet.

If run from outside any dfx project, or if dfx.json does not define the `local` network, then `dfx start` looks for the `local` network definition in `$HOME/.config/dfx/networks.json`. If this file does not exist or does not contain a definition for the `local` network, then dfx uses the following default definition:

```
{
  "local": {
    "bind": "127.0.0.1:4943",
    "type": "ephemeral"
  }
}
```

dfx stores data for the shared local network in one of the following locations, depending on your operating system:
- `$HOME/.local/share/dfx/network/local` (Linux)
- `$HOME/Library/Application Support/org.dfinity.dfx/network/local` (Macos)

### Project-Specific Local Networks

If dfx.json defines the `local` network, then `dfx start` will use this definition and store network data files under `\<project dir\>/.dfx/network/local`. 

Such project-specific networks are deprecated, and we plan to remove support for them after February 2023.  We encourage you to remove any definitions of the `local` network from your project's dfx.json file and instead use the default shared local network.

Note that for projects that define the `local` network in dfx.json, you can only run the `dfx start` and `dfx stop` commands from within the project directory structure. For example, if your project name is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its subdirectories.
