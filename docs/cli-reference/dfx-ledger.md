# dfx ledger

Use the `dfx ledger` command to interact with the ledger canister.

This command can be used to make ICP utility token transactions from one canister to another, or top up canisters with cycles from ICP.

The basic syntax for running `dfx ledger` commands is:

``` bash
dfx ledger [subcommand] [options]
```

Depending on the `dfx ledger` subcommand you specify, additional arguments, options, and flags might apply. For reference information and examples that illustrate using `dfx ledger` commands, select an appropriate command.

| Command                               | Description                                                                          |
|---------------------------------------|--------------------------------------------------------------------------------------|
| [`account-id`](#dfx-ledger-account-id)           | Prints the selected identity’s Account Identifier.                                   |
| [`balance`](#dfx-ledger-balance)                 | Prints the account balance of the user.                                              |
| [`create-canister`](#dfx-ledger-create-canister) | Creates a canister from ICP.                                                         |
| [`fabricate-cycles`](#dfx-ledger-fabricate-cycles) | Local development only: Fabricate cycles out of thin air and deposit them into the specified canister(s) |
| `help`                                | Displays usage information message for a specified subcommand.                       |
| [`notify`](#dfx-ledger-notify)                   | Notifies the ledger when there is a send transaction to the cycles minting canister. |
| [`top-up`](#dfx-ledger-top-up)                   | Tops up a canister with cycles minted from ICP.                                      |
| [`transfer`](#dfx-ledger-transfer)               | Transfers ICP from the user to the destination Account Identifier.                   |

To view usage information for a specific subcommand, specify the subcommand and the `--help` flag. For example, to see usage information for `dfx ledger transfer`, you can run the following command:

`dfx ledger transfer --help`

## dfx ledger account-id

Use the `dfx ledger account-id` command to display the account identifier associated with the currently-active identity. Like the textual representation of your developer identity principal, the account identifier is derived from your private key and used to represent your identity in the ledger canister. The command can also be used to compute and display the account ids of other principals, canister aliases and subaccounts of your account.

### Basic usage

``` bash
dfx ledger account-id [flag]
```

### Flags

You can use the following optional flags with the `dfx ledger account-id` command.

| Flag                         | Description                                            |
|------------------------------|--------------------------------------------------------|
| `-h`, `--help`               | Displays usage information.                            |
| `-V`, `--version`            | Displays version information.                          |
| `--of-canister <ALIAS>`      | Alias or principal of the canister controlling the account  |
| `--of-principal <PRINCIPAL>` | Principal controlling the account                      |
| `-subaccount <SUBACCOUNT>`   | Subaccount identifier (64 character long hex string)   |

### Examples

If you have created more than one identity, check the identity you are currently using by running the `dfx identity whoami` command or the `dfx identity get-principal` command. You can then check the account identifier for your currently-selected developer identity by running the following command:

``` bash
dfx ledger account-id
```

The command displays output similar to the following:

    03e3d86f29a069c6f2c5c48e01bc084e4ea18ad02b0eec8fccadf4487183c223

## dfx ledger balance

Use the `dfx ledger balance` command to print your account balance or that of another user.

### Basic usage

``` bash
dfx ledger balance [of] [flag] --network ic
```

### Flags

You can use the following optional flags with the `dfx ledger balance` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Arguments

You can specify the following argument for the `dfx ledger balance` command.

| Argument | Description                                                                                                                                                                 |
|----------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `<of>`   | Specify an Account Identifier to get the balance. If this command is not specified, the command returns the balance of ICP tokens for the currently-selected user identity. |

### Examples

You can use the `dfx ledger balance` command to check the balance of another user. For example, you can run the following command to see the ICP utlity tokens associated with a known Account Identifier:

``` bash
dfx ledger balance 03e3d86f29a069c6f2c5c48e01bc084e4ea18ad02b0eec8fccadf4487183c223 --network ic
```

This command displays an ICP amount similar to the following:

    2.49798000 ICP

## dfx ledger create-canister

Use the `dfx ledger create-canister` command to convert ICP tokens to cycles and to register a new canister identifier on the IC.

### Basic usage

``` bash
dfx ledger create-canister <controller> [options]  [flag] --network ic
```

### Flags

You can use the following optional flags with the `dfx ledger create-canister` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Arguments

You can specify the following argument for the `dfx ledger create-canister` command.

| Argument       | Description                                                                      |
|----------------|----------------------------------------------------------------------------------|
| `<controller>` | Specifies the principal identifier to set as the controller of the new canister. |

### Options

You can specify the following argument for the `dfx ledger create-canister` command.

| Option                | Description                                                                                                                                                                                                                                          |
|-----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--amount <amount>`           | Specify the number of ICP tokens to mint into cycles and deposit into destination canister. You can specify an amount as a number with up to eight (8) decimal places.                                                                               |
| `--e8s <e8s>`                 | Specify ICP token fractional units—called e8s—as a whole number, where one e8 is smallest partition of an ICP token. For example, 1.05000000 is 1 ICP and 5000000 e8s. You can use this option on its own or in conjunction with the `--icp` option. |
| `--fee <fee>`                 | Specify a transaction fee. The default is 10000 e8s.                                                                                                                                                                                                 |
| `--icp <icp>`                 | Specify ICP tokens as a whole number. You can use this option on its own or in conjunction with `--e8s`.                                                                                                                                             |
| `--max-fee <max-fee>`         | Specify a maximum transaction fee. The default is 10000 e8s.                                                                                                                                                                                         |
| `--subnet-type <subnet-type>` | Specify the optional subnet type to create the canister on. If no subnet type is provided, the canister will be created on a random default application subnet.                                                                                      |


### Examples

To create a new canister with cycles, transfer ICP tokens from your ledger account by running a command similar to the following:

``` bash
dfx ledger create-canister tsqwz-udeik-5migd-ehrev-pvoqv-szx2g-akh5s-fkyqc-zy6q7-snav6-uqe --amount 1.25 --network ic
```

This command converts the number of ICP tokens you specify for the `--amount` argument into cycles, and associates the cycles with a new canister identifier controlled by the principal you specify.

In this example, the command converts 1.25 ICP tokens into cycles and specifies the principal identifier for the default identity as the controller of the new canister.

If the transaction is successful, the ledger records the event and you should see output similar to the following:

    Transfer sent at BlockHeight: 20
    Canister created with id: "53zcu-tiaaa-aaaaa-qaaba-cai"

You can create a new canister by specifying separate values for ICP tokens and e8s by running a command similar to the following:

``` bash
dfx ledger create-canister tsqwz-udeik-5migd-ehrev-pvoqv-szx2g-akh5s-fkyqc-zy6q7-snav6-uqe --icp 3 --e8s 5000 --network ic
```

## dfx ledger fabricate-cycles

Use the `dfx ledger fabricate-cycles` add cycles to a canister while developing locally. The cycles are created out of thin air and are not deducted from anywhere.

### Basic usage

```
dfx ledger fabricate-cycles [options]
```

### Flags

You can use the following optional flags with the `dfx ledger fabricate-cycles` command.

| Flag | Description |
|------|-------------|
|`+-h+`, `+--help+` |Displays usage information. |
|`+-V+`, `+--version+` |Displays version information. |

### Options

You can specify the following options for the `dfx ledger fabricate-cycles` command.
If no amount is specified, 10T cycles are used by default.


|Option |Description|
|-------|-----------|
|`--all` |Deposit cycles to all of the canisters configured in the dfx.json file. |
|`--amount <icp>` |ICP to mint into cycles and deposit into destination canister Can be specified as a Decimal with the fractional portion up to 8 decimal places i.e. 100.012 |
|`--canister <canister name/id>` |Specifies the name or id of the canister to receive the cycles deposit. You must specify either a canister name/id or the --all option. |
|`--cycles <cycles>` | Specifies the amount of cycles to fabricate. |
|`--e8s <e8s>` | Specify e8s as a whole number, helpful for use in conjunction with `--icp`|
|`--icp`|  Specify ICP as a whole number, helpful for use in conjunction with `--e8s`|
|`--t <trillion cycles>` |Specifies the amount of cycles to fabricate in trillion cycles. Only accepts whole numbers.|

### Examples

If you are developing locally and want to add 8T cycles to all your canisters in your procject, you can do so like this:

```
dfx ledger fabricate-cycles --all --amount 8000000000000
```

The command displays output similar to the following:

```
Fabricating 8000000000000 cycles onto hello_backend
Fabricated 8000000000000 cycles, updated balance: 11_899_662_119_932 cycles
Fabricating 8000000000000 cycles onto hello_frontend
Fabricated 8000000000000 cycles, updated balance: 11_899_075_504_924 cycles
```

If you would rather only add the cycles to the canister called 'hello' and don't want to type all the zeros, you can do it like this:

```
dfx ledger fabricate-cycles --canister hello --t 8
```

The command displays output similar to the following:

```
Fabricating 8000000000000 cycles onto hello
Fabricated 8000000000000 cycles, updated balance: 11_899_662_119_932 cycles
```


## dfx ledger notify

Use the `dfx ledger notify` command to notify the ledger about a send transaction to the cycles minting canister. This command should only be used if `dfx ledger create-canister` or `dfx ledger top-up` successfully sent a message to the ledger, and a transaction was recorded at some block height, but for some reason the subsequent notify failed.

### Basic usage

``` bash
dfx ledger notify [options] _block-height_ _destination-principal_
```

### Flags

You can use the following optional flags with the `dfx ledger notify` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Arguments

You can specify the following argument for the `dfx ledger notify` command.

| Argument                  | Description                                                                                                                                                                                                                                                                                                     |
|---------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `<block-height>`          | Specifies the block height at which the send transaction was recorded.                                                                                                                                                                                                                                          |
| `<destination-principal>` | Specifies the principal of the destination, either a canister identifier or the textual representation of a user principal. If the send transaction was for the `create-canister` command, specify the `controller` principal. If the send transaction was for the `top-up` command, specify the `canister ID`. |

### Examples

The following example illustrates sending a `notify` message to the ledger in response to a `_send+` transaction that was recorded at the block height `75948`.

``` bash
dfx ledger notify 75948 tsqwz-udeik-5migd-ehrev-pvoqv-szx2g-akh5s-fkyqc-zy6q7-snav6-uqe --network ic
```

## dfx ledger show-subnet-types

Use the `dfx ledger show-subnet-types` command to list the available subnet types that can be chosen to create a canister on.

### Basic usage

``` bash
dfx ledger show-subnet-types [options] [flag]
```

### Flags

You can use the following optional flags with the `dfx ledger show-subnet-types` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Options

You can specify the following options for the `dfx ledger show-subnet-types` command.

| Option                | Description                                                                                                                                                                                                                                                 |
|-----------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--cycles-minting-canister-id <cycles-minting-canister-id>`   | Canister id of the cycles minting canister. Useful if you want to test locally with a different id for the cycles minting canister. |

### Examples

You can use the `dfx ledger show-subnet-types` command to list the available subnet types that can be chosen to create a canister on. If a specific cycles minting canister id is not provided, then the mainnet cycles minting canister id will be used.

For example, you can run the following command to get the subnet types available on mainnet:

``` bash
dfx ledger show-subnet-types
```

This command displays output similar to the following:

    ["Type1", "Type2", ..., "TypeN"]

## dfx ledger top-up

Use the `dfx ledger top-up` command to top up a canister with cycles minted from ICP tokens.

### Basic usage

``` bash
dfx ledger top-up [options] canister [flag] --network ic
```

### Flags

You can use the following optional flags with the `dfx ledger top-up` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Arguments

You can specify the following argument for the `dfx ledger top-up` command.

| Argument   | Description                                                      |
|------------|------------------------------------------------------------------|
| `canister` | Specifies the canister identifier that you would like to top up. |

### Options

You can specify the following options for the `dfx ledger top-up` command.

| Option                | Description                                                                                                                                                                                                                                                 |
|-----------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--amount <amount>`   | Specifies the number of ICP tokens to mint into cycles and deposit into the destination canister. You can specify the amount as a number with up to eight (8) decimal places.                                                                               |
| `--e8s <e8s>`         | Specifies fractional units of an ICP token—called e8s—as a whole number, where one e8 is the smallest unit of an ICP token. For example, 1.05000000 is 1 ICP and 5000000 e8s. You can use this option on its own or in conjunction with the `--icp` option. |
| `--fee <fee>`         | Specifies the transaction fee for the operation. The default is 10000 e8s.                                                                                                                                                                                  |
| `--icp <icp>`         | Specifies ICP tokens as a whole number. You can use this option on its own or in conjunction with `--e8s`.                                                                                                                                                  |
| `--max-fee <max-fee>` | Specifies a maximum transaction fee. The default is 10000 e8s.                                                                                                                                                                                              |

### Examples

You can use the `dfx ledger top-up` command to top up the cycles of a specific canister from the balance of ICP tokens you control. The canister identifier must be associated with a cycles wallet canister that is able to receive cycles. Alternatively, you can modify a non-cycles wallet canister to implement a method to receive cycles using system APIs described in the [Internet Computer Interface Specification](../ic-interface-spec).

For example, you can run the following command to top-up a cycles wallet canister deployed on the Internet Computer with 1 ICP worth of cycles:

``` bash
dfx ledger top-up --icp 1 5a46r-jqaaa-aaaaa-qaadq-cai --network ic
```

This command displays output similar to the following:

    Transfer sent at BlockHeight: 59482
    Canister was topped up!

## dfx ledger transfer

Use the `dfx ledger transfer` command to transfer ICP tokens from your account address in the ledger canister to a destination address.

### Basic usage

``` bash
dfx ledger transfer [options] to --memo memo
```

### Flags

You can use the following optional flags with the `dfx ledger transfer` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Arguments

You can specify the following argument for the `dfx ledger transfer` command.

| Argument | Description                                                                         |
|----------|-------------------------------------------------------------------------------------|
| `<to>`   | Specify the Account Identifier or address to which you want to transfer ICP tokens. |

### Options

You can specify the following argument for the `dfx ledger transfer` command.

| Option              | Description                                                                                                                                                                                                     |
|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--amount <amount>` | Specifies the number of ICP tokens to transfer. Can be specified as a number with up to eight (8) decimal places.                                                                                               |
| `--e8s <e8s>`       | Specifies e8s as a whole number, where one e8 is smallest partition of an ICP token. For example, 1.05000000 is 1 ICP and 5000000 e8s. You can use this option alone or in conjunction with the `--icp` option. |
| `--fee <fee>`       | Specifies a transaction fee. The default is 10000 e8s.                                                                                                                                                          |
| `--icp <icp>`       | Specifies ICP as a whole number. You can use this option alone or in conjunction with `--e8s`.                                                                                                                  |
| `--memo <memo>`     | Specifies a numeric memo for this transaction.                                                                                                                                                                  |

### Examples

You can use the `dfx ledger transfer` command to send ICP to the Account Identifier of the destination.

For example, you can run the following command to check the account identifier associated with the principal you are currently using:

``` bash
dfx ledger account-id
```

This command displays output similar to the following:

    30e596fd6c5ff5ad7b7d70bbbda1187c833e646c6251464da7f82bc217bba397

You can check the balance of this account by running the following command:

``` bash
dfx ledger balance --network ic
```

This command displays output similar to the following:

    64.89580000 ICP

Use the `dfx ledger transfer` command to send some of your ICP balance to another known destination using the following command:

``` bash
dfx ledger transfer dd81336dbfef5c5870e84b48405c7b229c07ad999fdcacb85b9b9850bd60766f --memo 12345 --icp 1 --network ic
```

This command displays output similar to the following:

    Transfer sent at BlockHeight: 59513

You can then use the `dfx ledger balance --network ic` command to check that your account balance reflects the transaction you just made.
