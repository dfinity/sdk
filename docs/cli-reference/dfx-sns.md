
Use the `dfx sns` subcommands to simulate decentralizing a dapp.

The basic syntax for running `dfx sns` commands is:

``` bash
dfx sns [subcommand] [flag]
```

Depending on the `dfx sns` subcommand you specify, additional arguments, options, and flags might apply. For reference information and examples that illustrate using `dfx sns` commands, select an appropriate command.

| Command                             | Description                                                                   |
|-------------------------------------|-------------------------------------------------------------------------------|
| [`create`](#_dfx_sns_create)        | Creates an SNS configuration template                                        |
| [`validate`](#_dfx_sns_validate)    | Checks whether the sns config file is valid.                                  |
| `help`                              | Displays usage information message for a specified subcommand.                |

To view usage information for a specific subcommand, specify the subcommand and the `--help` flag. For example, to see usage information for `dfx sns validate`, you can run the following command:

``` bash
dfx sns validate --help
```


## dfx sns create

Use the `dfx sns create` command to create an SNS configuration file. The configuration file specifies important, legally and financially relevant details about dapp decentralization.  The file leaves blank parameters such as token name; you will need to fill these in.

### Basic usage

``` bash
dfx sns create
```

### Flags

You can use the following optional flags with the `dfx sns create` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Examples

You can use the `dfx sns create` command to create and view a configuration file:

``` bash
dfx sns create
less sns.yml
```

## dfx sns validate

Use the `dfx sns validate` command to verify that an SNS configuration file is well formed.

### Basic usage

``` bash
dfx sns validate
```

### Flags

You can use the following optional flags with the `dfx sns validate` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Examples

You can use the `dfx sns validate` command to verify that a configuration template is valid.  It is not; it needs details such as token name:

``` bash
dfx sns create
dfx sns validate
```
