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
| [`balance`](#dfx-cycles-balance)      | Prints the account balance of the user.                                              |
| [`transfer`](#dfx-cycles-transfer)    | Send cycles to another account.                                                      |
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

## dfx cycles transfer

Use the `dfx cycles transfer` command to transfer cycles from your account to another account.

### Basic usage

``` bash
dfx cycles transfer [options] <to> <amount>
```

### Arguments

You must specify the following arguments for the `dfx cycles transfer` command.

| Argument   | Description                       |
|------------|-----------------------------------|
| `<to>`     | The principal of the account to which you want to transfer cycles. |
| `<amount>` | The number of cycles to transfer. |

### Options

You can specify the following options for the `dfx cycles transfer` command.

| Option                           | Description                                                                            |
|----------------------------------|----------------------------------------------------------------------------------------|
| `--to-subaccount <subaccount>`   | The subaccount to which you want to transfer cycles.                                   |
| `--from-subaccount <subaccount>` | The subaccount from which you want to transfer cycles.                                 |
| `--memo <memo>`                  | Specifies a numeric memo for this transaction. |
| `--created-at-time <timestamp>`  | Specify the timestamp-nanoseconds for the `created_at_time` field on the transfer request. Useful for controlling transaction-de-duplication. https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/#transaction-deduplication- |

### Examples

Transfer 1 billion cycles to another account:

``` bash
dfx cycles transfer raxcz-bidhr-evrzj-qyivt-nht5a-eltcc-24qfc-o6cvi-hfw7j-dcecz-kae 1000000000 --network ic
```

Transfer from a subaccount:

``` bash
dfx cycles transfer raxcz-bidhr-evrzj-qyivt-nht5a-eltcc-24qfc-o6cvi-hfw7j-dcecz-kae 1000000000 --from-subaccount 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f --network ic
```

## dfx cycles top-up

Use the `dfx cycles top-up` command to send cycles from your account to a canister.

### Basic usage

``` bash
dfx cycles top-up [options] <to> <amount>
```

### Arguments

You must specify the following arguments for the `dfx cycles transfer` command.

| Argument   | Description                                                             |
|------------|-------------------------------------------------------------------------|
| `<to>`     | The name of a canister in the current project, or a canister principal. |
| `<amount>` | The number of cycles to transfer.                                       |

### Options

You can specify the following options for the `dfx cycles top-up` command.

| Option                           | Description                                                                            |
|----------------------------------|----------------------------------------------------------------------------------------|
| `--from-subaccount <subaccount>` | The subaccount from which you want to transfer cycles.                                 |
| `--created-at-time <timestamp>`  | Specify the timestamp-nanoseconds for the `created_at_time` field on the transfer request. Useful for controlling transaction deduplication. https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/#transaction-deduplication- |

### Examples

Send cycles to a canister in your project:

``` bash
dfx cycles top-up my_backend 1000000000 --network ic
```

Send cycles to a canister by principal:

``` bash
dfx cycles top-up bkyz2-fmaaa-aaaaa-qaaaq-cai 1000000000 --network ic
```
