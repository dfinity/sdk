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

The source of the transferred cycles is always the cycles ledger account associated with your identity's principal, or one of its subaccounts.

The destination of the transferred cycles is one of the following:
- A cycles ledger account associated with another identity's principal, or one of its subaccounts. This mode uses the `--to-owner` and `--to-subaccount` options.
- A canister. This mode uses the `--top-up` option.

### Basic usage

``` bash
dfx cycles transfer [options] <amount>
```

### Arguments

You must specify the following argument for the `dfx cycles transfer` command.

| Argument     | Description                       |
|--------------|-----------------------------------|
| `<amount>`   | The number of cycles to transfer. |

### Options

You can specify the following options for the `dfx cycles transfer` command.

| Option                           | Description                                                                            |
|----------------------------------|----------------------------------------------------------------------------------------|
| `--top-up <principal>`           | The canister which you want to top up.                                                 |
| `--to-owner <principal>`         | The principal of the account to which you want to transfer cycles.                     |
| `--to-subaccount <subaccount>`   | The subaccount to which you want to transfer cycles.                                   |
| `--from-subaccount <subaccount>` | The subaccount from which you want to transfer cycles.                                 |
| `--fee <fee>`                    | Specifies a transaction fee. Can only passed in `--to-owner` mode.                     |
| `--memo <memo>`                  | Specifies a numeric memo for this transaction. Can only be passed in `--to-owner mode. |
| `--created-at-time <timestamp>`  | Specify the timestamp-nanoseconds for the `created_at_time` field on the transfer request. Useful for controlling transaction-de-duplication. https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/#transaction-deduplication- |

### Examples

Transfer 1 billion cycles to another account:

``` bash
dfx cycles transfer 1000000000 --to-owner raxcz-bidhr-evrzj-qyivt-nht5a-eltcc-24qfc-o6cvi-hfw7j-dcecz-kae --network ic
```

Transfer from a subaccount:

``` bash
dfx cycles transfer 1000000000 --from-subaccount 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f --to-owner raxcz-bidhr-evrzj-qyivt-nht5a-eltcc-24qfc-o6cvi-hfw7j-dcecz-kae --network ic
```

Transfer to (top up) a canister:

``` bash
dfx cycles transfer 1000000000 --top-up bkyz2-fmaaa-aaaaa-qaaaq-cai --network ic
```
