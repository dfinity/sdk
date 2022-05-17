# dfx build

Use the `dfx build` command to compile your program into a WebAssembly module that can be deployed on the IC. You can use this command to compile all of the programs that are defined for a project in the project’s `dfx.json` configuration file or a specific canister.

Note that you can only run this command from within the project directory structure. For example, if your project name is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its subdirectories.

The `dfx build` command looks for the source code to compile using the information you have configured under the `canisters` section in the `dfx.json` configuration file.

## Basic usage

``` bash
dfx build [flag] [option] [--all | canister_name]
```

## Flags

You can use the following optional flags with the `dfx build` command.

| Flag              | Description                                                                                                                                                      |
|-------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--check`         | Builds canisters using a temporary, hard-coded, locally-defined canister identifier for testing that your program compiles without connecting to the IC. |
| `-h`, `--help`    | Displays usage information.                                                                                                                                      |
| `-V`, `--version` | Displays version information.                                                                                                                                    |

## Options

You can specify the following option for the `dfx build` command.

| Option                | Description                                                                                                                                                |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--network <network>` | Specifies the network alias or URL you want to connect to. You can use this option to override the network specified in the `dfx.json` configuration file. |

## Arguments

You can specify the following arguments for the `dfx build` command.

| Argument        | Description                                                                                                                                                                                                                                                                                                                              |
|-----------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--all`         | Builds all of the canisters configured in the project’s `dfx.json` file.                                                                                                                                                                                                                                                                 |
| `canister_name` | Specifies the name of the canister you want to build. If you are not using the `--all` option, you can continue to use `dfx build` or provide a canister name as an argument (the canister name must match at least one name that you have configured in the `canisters` section of the `dfx.json` configuration file for your project.) |

## Examples

You can use the `dfx build` command to build one or more WebAssembly modules from the programs specified in the `dfx.json` configuration file under the `canisters` key. For example, if your `dfx.json` configuration file defines one `hello_world` canister and one `hello_world_assets` canister [like this](../_attachments/sample-dfx.json), then running `dfx build` compiles two WebAssembly modules.

Note that the file name and path to the programs on your file system must match the information specified in the `dfx.json` configuration file.

In this example, the `hello_world` canister contains the main program code and the `hello_world_assets` canister store frontend code and assets. If you want to keep the `hello_world_assets` canister defined in the `dfx.json` file, but only build the backend program, you could run the following command:

``` bash
dfx build hello_world
```

Building a specific canister is useful when you have multiple canisters defined in the dfx.json file, but want to test and debug operations for canisters independently.

To test whether a canister compiles without connecting to the IC or the local canister execution environment, you would run the following command:

``` bash
dfx build --check
```
