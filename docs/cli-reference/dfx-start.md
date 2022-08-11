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
| `--enable-bitcoin` | Enables bitcoin integration. |
| `--enable-canister-http` | Enables canister HTTP requests. |
| `-h`, `--help`           | Displays usage information.                                                                                                                                                                                                                  |
| `-V`, `--version`        | Displays version information.                                                                                                                                                                                                                |

## Options

You can use the following option with the `dfx start` command.

| Option        | Description                                                                                                       |
|---------------|-------------------------------------------------------------------------------------------------------------------|
| `--host host` | Specifies the host interface IP address and port number to bind the frontend to. The default is `127.0.0.1:8000`. |
| `--bitcoin-node host:port` | Specifies the address of a bitcoind node. Implies `--enable-bitcoin`. |

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

By default, `dfx start` manages a single replica that is independent from any given local project.  This is similar to the management of canisters on mainnet, where dfx deploys canisters for any of your local project.

If run from outside any dfx project, or if dfx.json does not define the `local` network, then `dfx start` looks for the `local` network definition in `$HOME/.config/dfx/networks.json`.  If this file does not exist or does not contain a definition for the `local` network, then dfx will use the following default definition:

```
{
  "local": {
    "bind": "127.0.0.1:8000",
    "type": "ephemeral"
  }
}
```

### Project-Specific Local Networks

If dfx.json defines the `local` network, then `dfx start` will use this definition and store all data files relative to the project.

Such project-specific networks are deprecated, slated to be removed after February 2023.  We encourage you to remove any definitions
of the `local` network from your project's dfx.json file and instead use the default local network that is shared by all projects.

Note that for projects that define the `local` network in dfx.json, you can only run the `dfx start` and `dfx stop` commands from within the project directory structure. For example, if your project name is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its subdirectories.