
Use the `dfx sns` subcommands to simulate decentralizing a dapp.

The basic syntax for running `dfx sns` commands is:

``` bash
dfx sns [subcommand] [flag]
```

Depending on the `dfx sns` subcommand you specify, additional arguments, options, and flags might apply. For reference information and examples that illustrate using `dfx sns` commands, select an appropriate command.

| Command                             | Description                                                                   |
|-------------------------------------|-------------------------------------------------------------------------------|
| [`create`](#_dfx_sns_create)        | Creates an SNS configuration template.                                        |
| [`validate`](#_dfx_sns_validate)    | Checks whether the sns config file is valid.                                  |
| [`deploy`](#_dfx_sns_deploy)        | Deploys SNS canisters according to the local config.                          |
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
dfx sns config create
```
Fill in the blank fields, then:
``` bash
dfx sns config validate
```

## dfx sns deploy

Use the `dfx sns deploy` command to create SNS canisters according to the local configuration file.

Note:  Deploying SNS canisters does not require a proposal, however there is a hefty fee.  Please don't create canisters on mainnet until you have tested your configuration locally and are sure that you are happy with it.

### Basic usage

``` bash
dfx sns deploy
```

### Flags

You can use the following optional flags with the `dfx sns deploy` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Examples

Create an SNS on the local testnet:
``` bash
dfx sns config create
```
Fill in the blank fields, then:
``` bash
dfx sns config validate
dfx sns deploy
```
You can now verify that the sns canisters have been created.  E.g.:
```
dfx canister info sns_root
dfx canister info sns_ledger
```

