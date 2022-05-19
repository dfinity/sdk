# dfx ping

Use the `dfx ping` command to check connectivity to the IC or a testnet. This command enables you to verify that you can connect to the environment where you want to deploy to.

Note that you can only run this command from within the project directory structure. For example, if your project name is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its subdirectories.

## Basic usage

``` bash
dfx ping [provider] [flag]
```

## Flags

You can use the following optional flags with the `dfx ping` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

## Arguments

You can specify the following argument for the `dfx ping` command.

| Argument | Description                                                   |
|----------|---------------------------------------------------------------|
| provider | Specifies the IC or testnet URL that you want to use. |

## Examples

You can use the `dfx ping` command to check whether the IC is currently available at a specific network address by running a command similar to the following:

``` bash
dfx ping https://testgw.dfinity.network
```

If the IC is running on the specified network provider address, the command returns output similar to the following:

    {
      "ic_api_version": "0.8"
    }
