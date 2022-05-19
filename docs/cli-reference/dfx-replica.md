# dfx replica

Use the `dfx replica` command to start a local canister execution environment (without a web server). This command enables you to deploy canisters locally and to test your dapps during development.

Note that you can only run this command from within the project directory structure. For example, if your project name is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its subdirectories.

## Basic usage

``` bash
dfx replica [option] [flag]
```

## Flags

You can use the following optional flags with the `dfx replica` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

## Options

You can use the following option with the `dfx replica` command.

| Option        | Description                                                                   |
|---------------|-------------------------------------------------------------------------------|
| `--port port` | Specifies the port the local canister execution environment should listen to. |

## Examples

You can start the local canister execution environment by running the following command:

``` bash
dfx replica
```
