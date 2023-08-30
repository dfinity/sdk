# dfx cycles

> **NOTE**: The cycles ledger is in development and the dfx cycles command is not expected to work on mainnet at this time.

Use the `dfx cycles` command to manage cycles associated with an identity's principal.

The basic syntax for running `dfx cycles` commands is:

``` bash
dfx cycles [subcommand] [options]
```

The following subcommands are available:

| Command                               | Description                                                                          |
|---------------------------------------|--------------------------------------------------------------------------------------|
| [`balance`](#dfx-ledger-balance)                 | Prints the account balance of the user.                                              |
| `help`                                | Displays usage information message for a specified subcommand.                       |

To view usage information for a specific subcommand, specify the subcommand and the `--help` flag. For example, to see usage information for `dfx cycles balance`, you can run the following command:

`dfx cycles balance --help`

## dfx cycles balance

Use the `dfx cycles balance` command to print your account balance or that of another user.

### Basic usage

``` bash
dfx cycles balance [flag] --network ic
```

### Options

You can specify the following arguments for the `dfx cycles balance` command.

| Option                                      | Description                                                         |
|---------------------------------------------|---------------------------------------------------------------------|
| `--owner <principal>`                       | Display the balance of this principal                               |
| `--subaccount <subaccount>`                 | Display the balance of this subaccount                              |
| `--precise`                                 | Displays the exact balance, without scaling to trillions of cycles. |
| `--cycles-ledger-canister-id <canister id>` | Specify the ID of the cycles ledger canister.                       |

### Examples

> **NOTE**: None of the examples below specify the `--cycles-ledger-canister-id` option, but it is required until the cycles ledger canister ID is known.

Check the cycles balance of the selected identity.

```
$ dfx cycles balance --network ic
89.000 TC (trillion cycles).
```

To see the exact amount of cycles, you can use the `--precise` option:
```
$ dfx cycles balance --network ic --precise
89000000000000 cycles.
```

You can use the `dfx cycles balance` command to check the balance of another principal:

``` bash
dfx cycles balance --owner raxcz-bidhr-evrzj-qyivt-nht5a-eltcc-24qfc-o6cvi-hfw7j-dcecz-kae --network ic
```

