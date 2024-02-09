# dfx canister

Use the `dfx canister` command with flags and subcommands to manage canister operations and interaction with the
Internet Computer or the local canister execution environment. In most cases, you use `dfx canister` subcommands after
you compile a program to manage the canister lifecycle and to perform key tasks such as calling program functions.

The basic syntax for running `dfx canister` commands is:

``` bash
dfx canister <subcommand> [flags]
```

Depending on the `dfx canister` subcommand you specify, additional arguments, options, and flags might apply or be
required. To view usage information for a specific `dfx canister` subcommand, specify the subcommand and the `--help`
flag. For example, to see usage information for `dfx canister call`, you can run the following command:

``` bash
dfx canister call --help
```

For reference information and examples that illustrate using `dfx canister` commands, select an appropriate command.

| Command                                            | Description                                                                                                                                            |
|----------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------|
| [`call`](#dfx-canister-call)                       | Calls a specified method on a deployed canister.                                                                                                       |
| [`create`](#dfx-canister-create)                   | Creates an empty canister and associates the assigned Canister ID to the canister name.                                                                |
| [`delete`](#dfx-canister-delete)                   | Deletes a currently stopped canister.                                                                                                                  |
| [`deposit-cycles`](#dfx-canister-deposit-cycles)   | Deposit cycles into the specified canister.                                                                                                            |
| `help`                                             | Displays usage information message for a specified subcommand.                                                                                         |
| [`id`](#dfx-canister-id)                           | Displays the identifier of a canister.                                                                                                                 |
| [`info`](#dfx-canister-info)                       | Get the hash of a canister’s WASM module and its current controller.                                                                                   |
| [`install`](#dfx-canister-install)                 | Installs compiled code in a canister.                                                                                                                  |
| [`metadata`](#dfx-canister-metadata)               | Displays metadata in a canister.                                                                                                                       |
| [`request-status`](#dfx-canister-request-status)   | Requests the status of a call to a canister.                                                                                                           |
| [`send`](#dfx-canister-send)                       | Send a previously-signed message.                                                                                                                      |
| [`sign`](#dfx-canister-send)                       | Sign a canister call and generate message file.                                                                                                        |
| [`start`](#dfx-canister-start)                     | Starts a stopped canister.                                                                                                                             |
| [`status`](#dfx-canister-status)                   | Returns the current status of a canister as defined [here](https://internetcomputer.org/docs/current/references/ic-interface-spec#ic-canister_status). |
| [`stop`](#dfx-canister-stop)                       | Stops a currently running canister.                                                                                                                    |
| [`uninstall-code`](#dfx-canister-uninstall-code)   | Uninstalls a canister, removing its code and state. Does not delete the canister.                                                                      |
| [`update-settings`](#dfx-canister-update-settings) | Update one or more of a canister's settings (i.e its controller, compute allocation, or memory allocation.).                                           |

## Overriding the default deployment environment

By default, `dfx canister` commands run on the local canister execution environment specified in the `dfx.json` file. If
you want to send a `dfx canister` subcommand to the Internet Computer or a testnet without changing the settings in
your `dfx.json` configuration file, you can explicitly specify the URL to connect to using the `--network` option.

For example, to register unique canister identifiers for a project on the local canister execution environment, you can
run the following command:

``` bash
dfx canister create --all
```

If you want to register unique canister identifiers for the same project on the Internet Computer, you can run the
following command:

``` bash
dfx canister create --all --network ic
```

The SDK comes with an alias of `ic`, which is configured to point to the Internet Computer. You can also pass a URL as a
network option, or you can configure additional aliases in `dfx.json` under the `networks` configuration, or
in `$HOME/.config/dfx/networks.json`.

To illustrate, you can call a canister and function running on a testnet using a command similar to the following:

``` bash
dfx canister call counter get --network http://192.168.3.1:5678
```

## Performing a call through the wallet

By default, most `dfx canister` commands to the Internet Computer are signed by and sent from your own principal. (
Exceptions are commands that require cycles: `dfx canister create` and `dfx canister deposit-cycles`. Those
automatically go through the wallet.) Occasionally, you may want to make a call from your wallet, e.g. when only your
wallet is allowed to call a certain function. To send a call through your wallet, you can use the `--wallet` flag like
this:

``` bash
dfx canister status <canister name> --wallet <wallet id>
```

As a concrete example, if you want to request the status of a canister on the ic that is only controlled by your wallet,
you would do the following:

``` bash
dfx identity get-wallet --network ic
```

This command outputs your wallet's principal (e.g. `22ayq-aiaaa-aaaai-qgmma-cai`) on the `ic` network. Using this id,
you can then query the status of the canister (let's assume the canister is called `my_canister_name`) as follows:

``` bash
dfx canister status --network ic --wallet 22ayq-aiaaa-aaaai-qgmma-cai
```

## dfx canister call

Use the `dfx canister call` command to call a specified method on a deployed canister.

### Basic usage

``` bash
dfx canister call [option] canister_name method_name [argument] [flag]
```

### Flags

You can use the following optional flags with the `dfx canister call` command.

| Flag       | Description                                                                                                                                                                                                               |
|------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--async`  | Specifies not to wait for the result of the call to be returned by polling the replica. Instead return a response ID.                                                                                                     |
| `--query`  | Sends a query request instead of an update request. For information about the difference between query and update calls, see [Canisters include both program and state](../../concepts/canisters-code.md#canister-state). |
| `--update` | Sends an update request to a canister. This is the default if the method is not a query method.                                                                                                                           |

### Options

You can use the following options with the `dfx canister call` command.

| Option                   | Description                                                                                                                                                                             |
|--------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--argument-file`        | Specifies the file from which to read the argument to pass to the method.  Stdin may be referred to as `-`.                                                                             |
| `--candid <file.did>`    | Provide the .did file with which to decode the response. Overrides value from dfx.json for project canisters.                                                                           |
| `--output <output>`      | Specifies the output format to use when displaying a method’s return result. The valid values are `idl`, `pp` and `raw`. The `pp` option is equivalent to `idl`, but is pretty-printed. |
| `--random <random>`      | Specifies the config for generating random arguments.                                                                                                                                   |
| `--type <type>`          | Specifies the data format for the argument when making the call using an argument. The valid values are `idl` and `raw`.                                                                |
| `--with-cycles <amount>` | Specifies the amount of cycles to send on the call. Deducted from the wallet. Requires `--wallet` as a flag to `dfx canister`.                                                          |

### Arguments

You can specify the following arguments for the `dfx canister call` command.

| Argument        | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
|-----------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `canister_name` | Specifies the name of the canister to call. The canister name is a required argument and should match the name you have configured for a project in the `canisters` section of the `dfx.json` configuration file.                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `method_name`   | Specifies the method name to call on the canister. The canister method is a required argument.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `argument`      | Specifies the argument to pass to the method. Depending on your program logic, the argument can be a required or optional argument. You can specify a data format type using the `--type` option if you pass an argument to the canister. By default, you can specify arguments using the [Candid](../../developer-docs/backend/candid/index.md) (`idl`) syntax for data values. For information about using Candid and its supported types, see [Interact with a service in a terminal](../../developer-docs/backend/candid/candid-howto.md#idl-syntax) and [Supported types](../candid-ref.md). You can use `raw` as the argument type if you want to pass raw bytes to a canister. |

### Examples

You can use the `dfx canister call` command to invoke specific methods—with or without arguments—after you have deployed
the canister using the `dfx canister install` command. For example, to invoke the `get` method for a canister with
a `canister_name` of `counter`, you can run the following command:

``` bash
dfx canister call counter get --async
```

In this example, the command includes the `--async` option to indicate that you want to make a separate `request-status`
call rather than waiting to poll the local canister execution environment or the Internet Computer for the result.
The `--async` option is useful when processing an operation might take some time to complete. The option enables you to
continue performing other operations then check for the result using a separate `dfx canister request-status` command.
The returned result will be displayed as the IDL textual format.

#### Using the IDL syntax

You can explicitly specify that you are passing arguments using the IDL syntax by running commands similar to the
following for a Text data type:

``` bash
dfx canister call hello greet --type idl '("Lisa")'
("Hello, Lisa!")

dfx canister call hello greet '("Lisa")' --type idl
("Hello, Lisa!")
```

You can also implicitly use the IDL by running a command similar to the following:

``` bash
dfx canister call hello greet '("Lisa")'
("Hello, Lisa!")
```

To specify multiple arguments using the IDL syntax, use commas between the arguments. For example:

``` bash
dfx canister call contacts insert '("Amy Lu","01 916-335-2042")'

dfx canister call hotel guestroom '("Deluxe Suite",42,true)'
```

You can pass raw data in bytes by running a command similar to the following:

``` bash
dfx canister call hello greet --type raw '4449444c00017103e29883'
```

This example uses the raw data type to pass a hexadecimal to the `greet` function of the `hello` canister.

## dfx canister create

Use the `dfx canister create` command to register one or more canister identifiers without compiled code. The new
canister principals are then recorded in `canister_ids.json` for non-local networks. You must be connected to the local
canister execution environment or the Internet Computer to run this command.

Note that you can only run this command from within the project directory structure. For example, if your project name
is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its
subdirectories.

### Basic usage

``` bash
dfx canister create [option] [flag] [--all | canister_name]
```

### Options

You can use the following options with the `dfx canister create` command.

| Option                                    | Description                                                                                                                                                                                                                                                                                                                                                                              |
|-------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `-c`, `--compute-allocation <allocation>` | Specifies the canister's compute allocation. This should be a percent in the range [0..100].                                                                                                                                                                                                                                                                                             |
| `--controller <principal>`                | Specifies the identity name or the principal of the new controller.                                                                                                                                                                                                                                                                                                                      |
| `--memory-allocation <memory>`            | Specifies how much memory the canister is allowed to use in total. This should be a value in the range [0..12 GiB]. A setting of 0 means the canister will have access to memory on a “best-effort” basis: It will only be charged for the memory it uses, but at any point in time may stop running if it tries to allocate more memory when there isn’t space available on the subnet. |
| `--reserved-cycles-limit <limit>`         | Specifies the upper limit for the canister's reserved cycles. |
| `--no-wallet`                             | Performs the call with the user Identity as the Sender of messages. Bypasses the Wallet canister. Enabled by default.                                                                                                                                                                                                                                                                    |
| `--with-cycles <number-of-cycles>`        | Specifies the initial cycle balance to deposit into the newly created canister. The specified amount needs to take the canister create fee into account. This amount is deducted from the wallet's cycle balance.                                                                                                                                                                        |
| `--specified-id <PRINCIPAL>`              | Attempts to create the canister with this Canister ID |
| `--subnet-type <subnet-type>`             | Specify the subnet type to create the canister on. If no subnet type is provided, the canister will be created on a random default application subnet.                      |
| `--subnet <subnet-principal>`             | Specify the subnet to create the canister on. If no subnet is provided, the canister will be created on a random default application subnet.                                |
| `--next-to <canister-principal>`          | Create canisters on the same subnet as this canister. |

### Arguments

You can use the following argument with the `dfx canister create` command.

| Argument        | Description                                                                                                                                                                                                                                                                                                    |
|-----------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--all`         | Enables you to create multiple canister identifiers at once if you have a project `dfx.json` file that defines multiple canisters. Note that you must specify `--all` or an individual canister name.                                                                                                          |
| `canister_name` | Specifies the name of the canister for which you want to register an identifier. If you are not using the `--all` option, the canister name is a required argument and must match at least one name that you have configured in the `canisters` section of the `dfx.json` configuration file for your project. |

### Examples

You can use the `dfx canister create` command to register canister identifiers without first compiling any code. For
example, if you want to create the canister identifier for the project `my_counter` before writing the program, you can
run the following command:

``` bash
dfx canister create my_counter
```

You can use the `dfx canister create` command with the `--with-cycles` option to specify the initial balance upon the
creation of one canister or all canisters in a project. For example, to specify an initial balance of 8000000000000
cycles for all canisters, run the following command:

``` bash
dfx canister create --with-cycles 8000000000000 --all
```

## dfx canister delete

Use the `dfx canister delete` command to delete a stopped canister from the local canister execution environment or the
Internet Computer. By default, this withdraws remaining cycles to your wallet before deleting the canister.

Note that you can only run this command from within the project directory structure. For example, if your project name
is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its
subdirectories.

### Basic usage

``` bash
dfx canister delete [flag] [--all | canister_name]
```

### Flags

You can use the following optional flags with the `dfx canister delete` command.

| Flag                        | Description                                         |
|-----------------------------|-----------------------------------------------------|
| `--no-withdrawal`           | Do not withdrawal cycles, just delete the canister. |
| `--withdraw-cycles-to-dank` | Withdraw cycles to dank with the current principal. |

### Arguments

You can use the following arguments with the `dfx canister delete` command.

| Argument                                          | Description                                                                                                                        |
|---------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------|
| `--all`                                           | Deletes all of the canisters configured in the `dfx.json` file. Note that you must specify `--all` or an individual canister name. |
| `canister_name`                                   | Specifies the name of the canister you want to delete. Note that you must specify either a canister name or the `--all` option.    |
| `--withdraw-cycles-to-canister <principal>`       | Withdraw cycles from canister(s) to the specified canister/wallet before deleting.                                                 |
| `--withdraw-cycles-to-dank-principal <principal>` | Withdraw cycles to dank with the given principal.                                                                                  |

### Examples

You can use the `dfx canister delete` command to delete a specific canister or all canisters.

To delete the `hello_world` canister, you can run the following command:

``` bash
dfx canister delete hello_world
```

To delete all of the canisters you have deployed on the `ic` Internet Computer and configured in your `dfx.json`, you
can run the following command:

``` bash
dfx canister  delete --all--network=ic
```

## dfx canister deposit-cycles

Use the `dfx canister deposit-cycles` command to deposit cycles from your configured wallet into a canister.

Note that you must have your cycles wallet configured for this to work.

### Basic usage

``` bash
dfx canister deposit-cycles [amount of cycles] [--all | canister_name]
```

### Arguments

You can use the following arguments with the `dfx canister deposit-cycles` command.

| Argument        | Description                                                                                                                                             |
|-----------------|---------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--all`         | Deposits the specified amount of cycles into all canisters configured in `dfx.json`. Note that you must specify `--all` or an individual canister name. |
| `canister_name` | Specifies the name of the canister you want to deposit cycles into. Note that you must specify either a canister name or the `--all` option.            |

### Examples

You can use the `dfx canister deposit-cycles` command to add cycles to a specific canister or all canisters.

To add 1T cycles to the canister called `hello`, you can run the following command:

``` bash
dfx canister deposit-cycles 1000000000000 hello
```

To add 2T cycles to each individual canister specified in `dfx.json`, you can run the following command:

``` bash
dfx canister deposit-cycles 2000000000000 --all
```

## dfx canister id

Use the `dfx canister id` command to output the canister identifier/principal for a specific canister name.

Note that you can only run this command from within the project directory structure. For example, if your project name
is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its
subdirectories.

If the canister has been deployed by the local user, the locally stored canister ID will be provided.

If a canister has been deployed by a third party, the user may set the `.canisters[$CANISTER_NAME].remote[$NETWORK]`
entry in `dfx.json` to the canister ID. In this case, the third party is responsible for maintaining the canister and
the local user must ensure that they have the correct canister ID.  `dfx` will return the provided canister ID with no
further checks.

If a canister is typically deployed to the same canister ID on mainnet and all testnets, the user may set a remote
canister ID for the `__default` network. In this case, `dfx canister id $CANISTER_NAME` will return the default canister
ID for all networks that don't have a dedicated entry.

### Basic usage

``` bash
dfx canister id [flag] canister_name
```

### Arguments

You can use the following argument with the `dfx canister id` command.

| Argument        | Description                                                                     |
|-----------------|---------------------------------------------------------------------------------|
| `canister_name` | Specifies the name of the canister for which you want to display an identifier. |

### Examples

You can use the `dfx canister id` command to display the canister identifier for a specific canister name.

To display the canister identifier for the `hello_world` canister, you can run the following command:

``` bash
dfx canister id hello_world
```

The command displays output similar to the following:

``` bash
75hes-oqbaa-aaaaa-aaaaa-aaaaa-aaaaa-aaaaa-q
```

## dfx canister info

Use the `dfx canister info` command to output a canister's controller and installed WASM module hash.

### Basic usage

``` bash
dfx canister info canister
```

### Arguments

You can use the following argument with the `dfx canister info` command.

| Argument   | Description                                                                  |
|------------|------------------------------------------------------------------------------|
| `canister` | Specifies the name or id of the canister for which you want to display data. |

### Examples

You can use the `dfx canister info` command to display the canister controller and installed WASM module.

To the data about the `hello_world` canister, you can run the following command:

``` bash
dfx canister info hello_world
```

The command displays output similar to the following:

```
Controllers: owdog-wiaaa-aaaad-qaaaq-cai
Module hash: 0x2cfb6f216fd6ab367364c02960afbbc5c444f5481225ee676992ac9058fd41e3
```

## dfx canister install

Use the `dfx canister install` command to install compiled code as a canister on the Internet Computer or on the local
canister execution environment.

### Basic usage

``` bash
dfx canister install [flag] [option] [--all | canister_name]
```

### Flags

You can use the following optional flags with the `dfx canister install` command.

| Flag                  | Description                                                                                                                                                             |
|-----------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--argument-file`     | Specifies the file from which to read the argument to pass to the init method.  Stdin may be referred to as `-`.                                                        |
| `--async-call`        | Enables you to continue without waiting for the result of the installation to be returned by polling the Internet Computer or the local canister execution environment. |
| `--upgrade-unchanged` | Upgrade the canister even if the .wasm did not change.                                                                                                                  |

### Options

You can use the following options with the `dfx canister install` command.

| Option                                            | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
|---------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--argument <argument>`                           | Specifies an argument to pass to the canister during installation.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `--argument-type <argument-type>`                 | Specifies the data format for the argument when you install using the `--argument` option. The valid values are `idl` and `raw`. By default, you can specify arguments using the [Candid](../../developer-docs/backend/candid/index.md) (`idl`) syntax for data values. For information about using Candid and its supported types, see [Interact with a service in a terminal](../../developer-docs/backend/candid/candid-howto.md#idl-syntax) and [Supported types](../candid-ref.md). You can use `raw` as the argument type if you want to pass raw bytes to a canister. |
| `-c`, `--compute-allocation <compute-allocation>` | Defines a compute allocation—essentially the equivalent of setting a CPU allocation—for canister execution. You can set this value as a percentage in the range of 0 to 100.                                                                                                                                                                                                                                                                                                                                                                                                 |
| `--memory-allocation <memory-allocation>`         | Specifies how much memory the canister is allowed to use in total. You can set this value in the range of 0 to 8MB.                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| `-m`, `--mode <mode>`                             | Specifies whether you want to `install`, `reinstall`, or `upgrade` canisters. Defaults to `install`. For more information about installation modes and canister management, see [Managing canisters](../../developer-docs/setup/manage-canisters.md).                                                                                                                                                                                                                                                                                                                        |
| `--wasm <file.wasm>`                              | Specifies a particular WASM file to install, bypassing the dfx.json project settings.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |

### Arguments

You can use the following arguments with the `dfx canister install` command.

| Argument        | Description                                                                                                                                                                                                                                                  |
|-----------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--all`         | Enables you to install multiple canisters at once if you have a project `dfx.json` file that includes multiple canisters. Note that you must specify `--all` or an individual canister name.                                                                 |
| `canister_name` | Specifies the name of the canister to deploy. If you are not using the `--all` option, the canister name is a required argument and should match the name you have configured for a project in the `canisters` section of the `dfx.json` configuration file. |

### Examples

You can use the `dfx canister install` command to deploy WebAssembly you have compiled using the `dfx build` command as
a canister on the Internet Computer or on the local canister execution environment. The most common use case is to
install all of the canisters by running the following command:

``` bash
dfx canister install --all
```

#### Installing a specific canister

You can also use the `dfx canister install` command to deploy a specific canister instead of all of the canisters in
your project. For example, if you have a project with a `hello_world_backend` canister and a `hello_world_frontend`
canister but only want to deploy the `hello_world_backend` canister, you can deploy just that the canister by running
the following command:

``` bash
dfx canister install hello_world_backend
```

#### Sending an asynchronous request

If you want to submit a request to install the canister and return a request identifier to check on the status of your
request later instead of waiting for the command to complete, you can run a command similar to the following:

``` bash
dfx canister install hello_world_backend --async
```

This command submits a request to install the canister and returns a request identifier similar to the following:

``` bash
0x58d08e785445dcab4ff090463b9e8b12565a67bf436251d13e308b32b5058608
```

You can then use the request identifier to check the status of the request at a later time, much like a tracking number
if you were shipping a package.

#### Overriding the default deployment options

If you want to deploy a canister on a testnet without changing the settings in your `dfx.json` configuration file, you
can explicitly specify the testnet you want to connect to by using the `--network` option.

For example, you can specify a testnet URL by running a command similar to the following:

``` bash
dfx canister install --all --network http://192.168.3.1:5678
```

#### Allocating message processing

The `--compute-allocation` options allows you to allocate computing resources as a percentage in the range of 0 to 100
to indicate how often your canister should be scheduled for execution.

For example, assume you run the following command:

``` bash
dfx canister install --all --compute-allocation 50
```

With this setting, all of the canisters in the current projects are assigned a 50% allocation. When canisters in the
project receive input messages to process, the messages are scheduled for execution. Over 100 execution cycles, each
canister’s messages will be scheduled for processing at least 50 times.

The default value for this option is 0—indicating that no specific allocation or scheduling is in effect. If all of your
canisters use the default setting, processing occurs in a round-robin fashion.

## dfx canister metadata

Use the `dfx canister metadata` command to display metadata stored in a canister's WASM module.

### Basic usage

``` bash
dfx canister metadata canister metadata-name
```

### Arguments

You can use the following argument with the `dfx canister metadata` command.

| Argument        | Description                                                                      |
|-----------------|----------------------------------------------------------------------------------|
| `canister`      | Specifies the name or id of the canister for which you want to display metadata. |
| `metadata-name` | Specifies the name of the metadata which you want to display.                    |

### Examples

To display the candid service metadata for the `hello_world` canister, you can run the following command:

``` bash
dfx canister metadata hello_world candid:service
```

The command displays output similar to the following:

```
service : {
  greet: (text) -> (text);
}
```

## dfx canister request-status

Use the `dfx canister request-status` command to request the status of a specified call to a canister. This command
requires you to specify the request identifier you received after invoking a method on the canister. The request
identifier is an hexadecimal string starting with `0x`.

### Basic usage

``` bash
dfx canister request-status request_id canister [option]
```

### Options

You can use the following options with the `dfx canister request-status` command.

| Option              | Description                                                                                                                                                          |
|---------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--output <output>` | Specifies the format for displaying the method's return result. Possible values are `idl`, `raw` and `pp`, where `pp` is equivalent to `idl`, but is pretty-printed. |

### Arguments

You can specify the following argument for the `dfx canister request-status` command.

| Argument     | Description                                                                                                                                                                  |
|--------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `request_id` | Specifies the hexadecimal string returned in response to a `dfx canister call` or `dfx canister install` command. This identifier is an hexadecimal string starting with 0x. |

### Examples

You can use the `dfx canister request-status` command to check on the status of a canister state change or to verify
that a call was not rejected by running a command similar to the following:

``` bash
dfx canister request-status 0x58d08e785445dcab4ff090463b9e8b12565a67bf436251d13e308b32b5058608 backend
```

This command displays an error message if the request identifier is invalid or refused by the canister.

## dfx canister send

Use the `dfx canister send` command after signing a message with the `dfx canister sign` command when you want to
separate these steps, rather than using the single `dfx canister call` command. Using separate calls can add security to
the transaction.

For example, when creating your neuron stake, you might want to use the `dfx canister sign` command to create a
signed `message.json` file using an air-gapped computer, then use the `dfx canister send` command to deliver the signed
message.

### Basic usage

``` bash
dfx canister send file_name
```

### Flags

You can use the following optional flags with the `dfx canister request-status` command.

| Flag       | Description                                         |
|------------|-----------------------------------------------------|
| `--status` | Send the signed request-status call in the message. |

### Arguments

You can specify the following argument for the `dfx canister send` command.

| Argument    | Description                             |
|-------------|-----------------------------------------|
| `file_name` | Specifies the file name of the message. |

### Examples

Use the `dfx canister send` command to send a signed message created using the `dfx canister sign` command to the
genesis token canister (GTC) to create a neuron on your behalf by running the following command:

`dfx canister send message.json`

## dfx canister sign

Use the `dfx canister sign` command before sending a message with the `dfx canister send` command when you want to
separate these steps, rather than using the single `dfx canister call` command. Using separate calls can add security to
the transaction. For example, when creating your neuron stake, you might want to use the `dfx canister sign` command to
create a signed `message.json` file using an air-gapped computer, then use the `dfx canister send` command to deliver
the signed message from a computer connected to the Internet Computer.

### Basic usage

``` bash
dfx canister sign [flag] [option] canister-name method-name [argument]
```

### Flags

You can use the following optional flags with the `dfx canister sign` command.

| Flag       | Description                                                                                              |
|------------|----------------------------------------------------------------------------------------------------------|
| `--query`  | Sends a query request to a canister.                                                                     |
| `--update` | Sends an update request to the canister. This is the default method if the `--query` method is not used. |

### Options

You can specify the following options for the `dfx canister sign` command.

| Option                     | Description                                                                                                                                      |
|----------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------|
| `--argument-file <file>`   | Specifies the file from which to read the argument to pass to the method.  Stdin may be referred to as `-`.                                      |
| `--expire-after <seconds>` | Specifies how long the message will be valid before it expires and cannot be sent. Specify in seconds. If not defined, the default is 300s (5m). |
| `--file <output>`          | Specifies the output file name. The default is `message.json`.                                                                                   |
| `--random <random>`        | Specifies the configuration for generating random arguments.                                                                                     |
| `--type <type>`            | Specifies the data type for the argument when making a call using an argument. Possible values are `idl` and `raw`.                              |

### Arguments

You can specify the following arguments for the `dfx canister sign` command.

| Argument        | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
|-----------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `canister_name` | Specifies the name of the canister to call. The canister name is a required argument and should match the name you have configured for a project in the `canisters` section of the `dfx.json` configuration file.                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `method_name`   | Specifies the method name to call on the canister. The canister method is a required argument.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `argument`      | Specifies the argument to pass to the method. Depending on your program logic, the argument can be a required or optional argument. You can specify a data format type using the `--type` option if you pass an argument to the canister. By default, you can specify arguments using the [Candid](../candid-ref.md) (`idl`) syntax for data values. For information about using Candid and its supported types, see [Interact with a service in a terminal](../../developer-docs/backend/candid/candid-howto.md#idl-syntax) and [Supported types](../candid-ref#supported-types). You can use `raw` as the argument type if you want to pass raw bytes. |

### Examples

Use the `dfx canister sign` command to create a signed `message.json` file using the selected identity by running a
command similar to the following:

`dfx canister sign --network=ic --expire-after=1h rno2w-sqaaa-aaaaa-aaacq-cai create_neurons ‘(“PUBLIC_KEY”)’`

This command illustrates how to creates a `message.json` file to create neurons on the Internet Computer specified by
the `ic` alias, that is signed using your principal identifier as the message sender and with an expiration window that
ends in one hour.

Note that the time allotted to send a signed message is a fixed 5-minute window. The `--expire-after` option enables you
to specify the point in time when the 5-minute window for sending the signed message should end. For example, if you set
the `--expire-after` option to one hour (`1h`), you must wait at least 55 minutes before you send the generated message
and the signature for the message is only valid during the 5-minute window ending in the 60th minute.

In this example, therefore, you would need to send the message after 55 minutes and before 60 minutes for the message to
be recognized as valid.

If you don’t specify the `--expire-after` option, the default expiration is five minutes.

Send the signed message to the genesis token canister (GTC) to create a neuron on your behalf by running the following
command:

`dfx canister send message.json`

## dfx canister start

Use the `dfx canister start` command to restart a stopped canister on the Internet Computer or the local canister
execution environment.

In most cases, you run this command after you have stopped a canister to properly terminate any pending requests as a
prerequisite to upgrading the canister.

Note that you can only run this command from within the project directory structure. For example, if your project name
is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its
subdirectories.

### Basic usage

``` bash
dfx canister start [--all | canister_name]
```

### Arguments

You can use the following arguments with the `dfx canister start` command.

| Argument        | Description                                                                                                                       |
|-----------------|-----------------------------------------------------------------------------------------------------------------------------------|
| `--all`         | Starts all of the canisters configured in the `dfx.json` file. Note that you must specify `--all` or an individual canister name. |
| `canister_name` | Specifies the name of the canister you want to start. Note that you must specify either a canister name or the `--all` option.    |

### Examples

You can use the `dfx canister start` command to start a specific canister or all canisters.

To start the `hello_world` canister, you can run the following command:

``` bash
dfx canister start hello_world
```

To start all of the canisters you have deployed on the `ic` Internet Computer, you can run the following command:

``` bash
dfx canister start --all --network=ic
```

## dfx canister status

Use the `dfx canister status` command to check whether a canister is currently running, in the process of stopping, or
currently stopped on the Internet Computer or on the local canister execution environment.

Note that you can only run this command from within the project directory structure. For example, if your project name
is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its
subdirectories.

### Basic usage

``` bash
dfx canister status [--all | canister_name]
```

### Arguments

You can use the following arguments with the `dfx canister status` command.

| Argument        | Description                                                                                                                                               |
|-----------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--all`         | Returns status information for all of the canisters configured in the `dfx.json` file. Note that you must specify `--all` or an individual canister name. |
| `canister_name` | Specifies the name of the canister you want to return information for. Note that you must specify either a canister name or the `--all` option.           |

### Examples

You can use the `dfx canister status` command to check the status of a specific canister or all canisters.

To check the status of the `hello_world` canister, you can run the following command:

``` bash
dfx canister status hello_world
```

To check the status for all of the canisters you have deployed on the `ic` Internet Computer, you can run the following
command:

``` bash
dfx canister status --all --network=ic
```

## dfx canister stop

Use the `dfx canister stop` command to stop a canister that is currently running on the Internet Computer or on the
local canister execution environment.

In most cases, you run this command to properly terminate any pending requests as a prerequisite to upgrading the
canister.

Note that you can only run this command from within the project directory structure. For example, if your project name
is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its
subdirectories.

### Basic usage

``` bash
dfx canister stop [--all | canister_name]
```

### Arguments

You can use the following arguments with the `dfx canister stop` command.

| Argument        | Description                                                                                                                      |
|-----------------|----------------------------------------------------------------------------------------------------------------------------------|
| `--all`         | Stops all of the canisters configured in the `dfx.json` file. Note that you must specify `--all` or an individual canister name. |
| `canister_name` | Specifies the name of the canister you want to stop. Note that you must specify either a canister name or the `--all` option.    |

### Examples

You can use the `dfx canister stop` command to stop a specific canister or all canisters.

To stop the `hello_world` canister, you can run the following command:

``` bash
dfx canister stop hello_world
```

To stop all of the canisters you have deployed on the `ic` Internet Computer, you can run the following command:

``` bash
dfx canister stop --all --network=ic
```

## dfx canister uninstall-code

Use the `dfx canister uninstall-code` command to uninstall the code that a canister that is currently running on the
Internet Computer or on the local canister execution environment.

This method removes a canister’s code and state, making the canister empty again. Only the controller of the canister
can uninstall code. Uninstalling a canister’s code will reject all calls that the canister has not yet responded to, and
drop the canister’s code and state. Outstanding responses to the canister will not be processed, even if they arrive
after code has been installed again. The canister is now empty.

Note that you can only run this command from within the project directory structure. For example, if your project name
is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its
subdirectories.

### Basic usage

``` bash
dfx canister uninstall-code [flag] [--all | canister_name]
```

### Arguments

You can use the following arguments with the `dfx canister uninstall-code` command.

| Argument        | Description                                                                                                                           |
|-----------------|---------------------------------------------------------------------------------------------------------------------------------------|
| `--all`         | Uninstalls all of the canisters configured in the `dfx.json` file. Note that you must specify `--all` or an individual canister name. |
| `canister_name` | Specifies the name of the canister you want to uninstall. Note that you must specify either a canister name or the `--all` option.    |

### Examples

You can use the `dfx canister uninstall-code` command to uninstall a specific canister or all canisters.

To uninstall the `hello_world` canister, you can run the following command:

``` bash
dfx canister uninstall-code hello_world
```

To uninstall all of the canisters you have deployed on the `ic` Internet Computer, you can run the following command:

``` bash
dfx canister uninstall-code --all --network=ic
```

## dfx canister update-settings

Use the `dfx canister update-settings` command to update the settings of a canister running in the local execution
environment.

In most cases, you run this command to tune the amount of resources allocated to your canister.

Note that you can only run this command from within the project directory structure. For example, if your project name
is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its
subdirectories.

### Basic usage

``` bash
dfx canister update-settings [flags] [options] [canister_name | --all]
```

### Flags

You can use the following optional flags with the `dfx canister update-settings` command.

| Flag                                     | Description                                                             |
|------------------------------------------|-------------------------------------------------------------------------|
| `--confirm-very-long-freezing-threshold` | Freezing thresholds above ~1.5 years require this flag as confirmation. |

### Options

You can specify the following options for the `dfx canister update-settings` command.

| Option                                    | Description                                                                                                                                                                                                                                                                                                                                                                              |
|-------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--add-controller <principal>`            | Add a principal to the list of controllers of the canister.                                                                                                                                                                                                                                                                                                                              |
| `-c`, `--compute-allocation <allocation>` | Specifies the canister's compute allocation. This should be a percent in the range [0..100].                                                                                                                                                                                                                                                                                             |
| `--set-controller <principal>`            | Specifies the identity name or the principal of the new controller. Can be specified more than once, indicating the canister will have multiple controllers. If any controllers are set with this parameter, any other controllers will be removed.                                                                                                                                      |
| `--memory-allocation <allocation>`        | Specifies how much memory the canister is allowed to use in total. This should be a value in the range [0..12 GiB]. A setting of 0 means the canister will have access to memory on a “best-effort” basis: It will only be charged for the memory it uses, but at any point in time may stop running if it tries to allocate more memory when there isn’t space available on the subnet. |
| `--reserved-cycles-limit <limit>`         | Specifies the upper limit of the canister's reserved cycles. |
| `--remove-controller <principal>`         | Removes a principal from the list of controllers of the canister.                                                                                                                                                                                                                                                                                                                        |
| `--freezing-threshold <seconds>`          | Set the [freezing threshold](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-create_canister) in seconds for a canister. This should be a value in the range [0..2^64^-1]. Very long thresholds require the `--confirm-very-long-freezing-threshold` flag.                                                                                                    |
| `-y`, `--yes`                             | Skips yes/no checks by answering 'yes'. Such checks can result in loss of control, so this is not recommended outside of CI.                                                                                                                                                                                                                                                             |

### Arguments

You can use the following arguments with the `dfx canister update-settings` command.

| Argument        | Description                                                                                                           |
|-----------------|-----------------------------------------------------------------------------------------------------------------------|
| `--all`         | Updates all canisters you have specified in `dfx.json`. You must specify either canister name/id or the --all option. |
| `canister_name` | Specifies the name of the canister you want to update. You must specify either canister name/id or the --all option.  |

### Examples

You can use the `dfx canister update-settings` command to update settings of a specific canister.

To update the settings of the `hello_world` canister, you can run the following command:

``` bash
dfx canister update-settings --freezing-threshold 2592000 --compute-allocation 99 hello_world
```
