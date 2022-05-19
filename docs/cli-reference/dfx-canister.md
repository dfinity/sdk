# dfx canister

Use the `dfx canister` command with flags and subcommands to manage canister operations and interaction with the {platform} or the local canister execution environment. In most cases, you use `dfx canister` subcommands after you compile a program to manage the canister lifecycle and to perform key tasks such as calling program functions.

The basic syntax for running `dfx canister` commands is:

``` bash
dfx canister [subcommand] [flag]
```

Depending on the `dfx canister` subcommand you specify, additional arguments, options, and flags might apply or be required. To view usage information for a specific `dfx canister` subcommand, specify the subcommand and the `--help` flag. For example, to see usage information for `dfx canister call`, you can run the following command:

``` bash
dfx canister call --help
```

For reference information and examples that illustrate using `dfx canister` commands, select an appropriate command.

| Command                               | Description               |
|---------------------------------------|---------------------------|
| [`call`](#dfx-canister-call)         | Calls a specified method on a deployed |
| [`create`](#dfx-canister-create)     | Creates a new "empty" canister by registering a canister identifier on the {platform} or the local canister execution environment.|
| [`delete`](#dfx-canister-delete)     | Deletes a currently stopped canister.                                                                                          |
| `help`  | Displays usage information message for a specified subcommand.       |
| [`id`](#dfx-canister-id)                         | Displays the identifier for a canister.   |
| [`install`](#dfx-canister-install)               | Installs compiled code as a canister on the {platform} or the local canister execution environment. |
| [`request-status`](#dfx-canister-request-status) | Requests the status of a call to a canister. |
| [`set-controller`](#dfx-canister-set-controller) | Specifies the identity name or principal to use as the new controller for a specified canister on the {platform}.|
| [`send`](#dfx-canister-send)                     | Send a previously-signed `message.json` to a specified canister identifier. For example, if you want to send a message that calls the network nervous system (NNS) governance canister to manage neurons, you might want to separate message signing from message delivery for security reasons.|
| [`sign`](#dfx-canister-send)                     | Create a signed `message.json` file before making a call to a specified canister identifier. For example, if you want to send a message that calls the network nervous system (NNS) governance canister to manage neurons, you might want to separate message signing from message delivery for security reasons. |
| [`start`](#dfx-canister-start)                   | Restarts a stopped canister. |
| [`status`](#dfx-canister-status)                 | Requests the running status of a canister.|
| [`stop`](#dfx-canister-stop)                     | Stops a currently running canister.|

## Overriding the default deployment environment

By default, `dfx canister` commands run on the local canister execution environment specified in the `dfx.json` file. If you want to send a `dfx canister` subcommand to the {platform} or a testnet without changing the settings in your `dfx.json` configuration file, you can explicitly specify the URL to connect to using the `--network` option.

For example, to register unique canister identifiers for a project on the local canister execution environment, you can run the following command:

``` bash
dfx canister create --all
```

If you want to register unique canister identifiers for the same project on the {platform}, you can run the following command:

``` bash
dfx canister --network ic create --all
```

The SDK comes with an alias of `ic`, which is configured to point to the {platform}. You can also pass a URL as a network option, or you can configure additional aliases in `dfx.json` under the `networks` configuration.

To illustrate, you can call a canister and function running on a testnet using a command similar to the following:

``` bash
dfx canister --network \http://192.168.3.1:5678 call counter get
```

Note that you must specify the `--network` parameter before the canister operation (`create` or `call`) and any additional arguments such as the canister name (`counter`), and function (`get`).

## dfx canister call 

Use the `dfx canister call` command to call a specified method on a deployed canister.

### Basic usage

``` bash
dfx canister call [option] canister_name method_name [argument] [flag]
```

### Flags

You can use the following optional flags with the `dfx canister call` command.

| Flag              | Description                                                                                                                                                                                                                                                                                                                                              |
|-------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--async`         | Enables you to continue without waiting for the result of the call to be returned by polling the local canister execution environment or the {platform}.                                                                                                                                                                                                 |
| `-h`, `--help`    | Displays usage information.                                                                                                                                                                                                                                                                                                                              |
| `--query`         | Enables you to send a query request to a deployed canister. For best performance, you should use this flag when you explicitly want to use the query method to retrieve information. For information about the difference between query and update calls, see [Canisters include both program and state](../../concepts/canisters-code#canister-state). |
| `--update`        | Enables you to send an update request to a deployed canister. By default, canister calls use the update method.                                                                                                                                                                                                                                          |
| `-V`, `--version` | Displays version information.                                                                                                                                                                                                                                                                                                                            |

### Options

You can use the following options with the `dfx canister call` command.

| Option              | Description                                                                                                              |
|---------------------|--------------------------------------------------------------------------------------------------------------------------|
| `--output <output>` | Specifies the output format to use when displaying a method’s return result. The valid values are `idl` and `raw`.       |
| `--type <type>`     | Specifies the data format for the argument when making the call using an argument. The valid values are `idl` and `raw`. |

### Arguments

You can specify the following arguments for the `dfx canister call` command.

| Argument        | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
|-----------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `canister_name` | Specifies the name of the canister to call. The canister name is a required argument and should match the name you have configured for a project in the `canisters` section of the `dfx.json` configuration file.                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `method_name`   | Specifies the method name to call on the canister. The canister method is a required argument.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `argument`      | Specifies the argument to pass to the method. Depending on your program logic, the argument can be a required or optional argument. You can specify a data format type using the `--type` option if you pass an argument to the canister. By default, you can specify arguments using the [Candid](../../developer-docs/build/languages/candid/candid-intro) (`idl`) syntax for data values. For information about using Candid and its supported types, see [Interact with a service in a terminal](../../developer-docs/build/languages/candid/candid-howto#idl-syntax) and [Supported types](../candid-ref.md). You can use `raw` as the argument type if you want to pass raw bytes to a canister. |

### Examples

You can use the `dfx canister call` command to invoke specific methods—with or without arguments—after you have deployed the canister using the `dfx canister install` command. For example, to invoke the `get` method for a canister with a `canister_name` of `counter`, you can run the following command:

``` bash
dfx canister call counter get --async
```

In this example, the command includes the `--async` option to indicate that you want to make a separate `request-status` call rather than waiting to poll the local canister execution environment or the {platform} for the result. The `--async` option is useful when processing an operation might take some time to complete. The option enables you to continue performing other operations then check for the result using a separate `dfx canister request-status` command. The returned result will be displayed as the IDL textual format.

#### Using the IDL syntax

You can explicitly specify that you are passing arguments using the IDL syntax by running commands similar to the following for a Text data type:

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

Use the `dfx canister create` command to register one or more canister identifiers without compiled code. You must be connected to the local canister execution environment or the {platform} to run this command.

Note that you can only run this command from within the project directory structure. For example, if your project name is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its subdirectories.

The first time you run the `dfx canister create` command to register an identifier, your public/private key pair credentials are used to create a `default` user identity. The credentials for the `default` user are migrated from `$HOME/.dfinity/identity/creds.pem` to `$HOME/.config/dfx/identity/default/identity.pem`.

### Basic usage

``` bash
dfx canister create [option] [flag] [--all | canister_name]
```

### Flags

You can use the following optional flags with the `dfx canister create` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Options

You can use the following options with the `dfx canister create` command.

| Option                             | Description                                                                                          |
|------------------------------------|------------------------------------------------------------------------------------------------------|
| `--with-cycles <number-of-cycles>` | Enables you to specify the initial number of cycles in a canister when it is created by your wallet. |

### Arguments

You can use the following argument with the `dfx canister create` command.

| Argument        | Description|
|-----------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--all`         | Enables you to create multiple canister identifiers at once if you have a project `dfx.json` file that defines multiple canisters. Note that you must specify `--all` or an individual canister name.                                                                                                          |
| `canister_name` | Specifies the name of the canister for which you want to register an identifier. If you are not using the `--all` option, the canister name is a required argument and must match at least one name that you have configured in the `canisters` section of the `dfx.json` configuration file for your project. |

### Examples

You can use the `dfx canister create` command to register canister identifiers without first compiling any code. For example, if you want to create the canister identifier for the project `my_counter` before writing the program, you can run the following command:

``` bash
dfx canister create my_counter
```

You can use the `dfx canister create` command with the `--with-cycles` option to specify the initial balance upon the creation of one canister or all canisters in a project. For example, to specify an initial balance of 8000000000000 cycles for all canisters, run the following command:

``` bash
dfx canister create --with-cycles 8000000000000 --all
```

## dfx canister delete

Use the `dfx canister delete` command to delete a stopped canister from the local canister execution environment or on the {platform}.

Note that you can only run this command from within the project directory structure. For example, if your project name is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its subdirectories.

### Basic usage

``` bash
dfx canister delete [flag] [--all | canister_name]
```

### Flags

You can use the following optional flags with the `dfx canister delete` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Arguments

You can use the following arguments with the `dfx canister delete` command.

| Argument        | Description                                                                                                                        |
|-----------------|------------------------------------------------------------------------------------------------------------------------------------|
| `--all`         | Deletes all of the canisters configured in the `dfx.json` file. Note that you must specify `--all` or an individual canister name. |
| `canister_name` | Specifies the name of the canister you want to delete. Note that you must specify either a canister name or the `--all` option.    |

### Examples

You can use the `dfx canister delete` command to delete a specific canister or all canisters.

To delete the `hello_world` canister, you can run the following command:

``` bash
dfx canister delete hello_world
```

To delete all of the canisters you have deployed on the `ic` {platform}, you can run the following command:

``` bash
dfx canister --network=ic delete --all
```

## dfx canister id

Use the `dfx canister id` command to output the canister identifier for a specific canister name.

Note that you can only run this command from within the project directory structure. For example, if your project name is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its subdirectories.

### Basic usage

``` bash
dfx canister id [flag] canister_name
```

### Flags

You can use the following optional flags with the `dfx canister id` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

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

## dfx canister install

Use the `dfx canister install` command to install compiled code as a canister on the {platform} or on the local canister execution environment.

### Basic usage

``` bash
dfx canister install [flag] [option] [--all | canister_name]
```

### Flags

You can use the following optional flags with the `dfx canister install` command.

| Flag              | Description                                                                                                                                                      |
|-------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--async`         | Enables you to continue without waiting for the result of the installation to be returned by polling the {platform} or the local canister execution environment. |
| `-h`, `--help`    | Displays usage information.                                                                                                                                      |
| `-V`, `--version` | Displays version information.                                                                                                                                    |

### Options

You can use the following options with the `dfx canister install` command.

| Option                                            | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
|---------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--argument <argument>`                           | Specifies an argument to pass to the canister during installation.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `--argument-type <argument-type>`                 | Specifies the data format for the argument when you install using the `--argument` option. The valid values are `idl` and `raw`. By default, you can specify arguments using the [Candid](../../developer-docs/build/languages/candid/candid-intro) (`idl`) syntax for data values. For information about using Candid and its supported types, see [Interact with a service in a terminal](../../developer-docs/build/languages/candid/candid-howto#idl-syntax) and [Supported types](../candid-ref.md). You can use `raw` as the argument type if you want to pass raw bytes to a canister. |
| `-c`, `--compute-allocation <compute-allocation>` | Defines a compute allocation—essentially the equivalent of setting a CPU allocation—for canister execution. You can set this value as a percentage in the range of 0 to 100.                                                                                                                                                                                                                                                                                                                                                                                           |
| `--memory-allocation <memory-allocation>`         | Specifies how much memory the canister is allowed to use in total. You can set this value in the range of 0 to 8MB.                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `-m`, `--mode <mode>`                             | Specifies whether you want to `install`, `reinstall`, or `upgrade` canisters. For more information about installation modes and canister management, see [Managing canisters](../../developer-docs/build/project-setup/manage-canisters).                                                                                                                                                                                                                                                                                                                                                          |

### Arguments

You can use the following arguments with the `dfx canister install` command.

| Argument        | Description                                                                                                                                                                                                                                                  |
|-----------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--all`         | Enables you to install multiple canisters at once if you have a project `dfx.json` file that includes multiple canisters. Note that you must specify `--all` or an individual canister name.                                                                 |
| `canister_name` | Specifies the name of the canister to deploy. If you are not using the `--all` option, the canister name is a required argument and should match the name you have configured for a project in the `canisters` section of the `dfx.json` configuration file. |

### Examples

You can use the `dfx canister install` command to deploy WebAssembly you have compiled using the `dfx build` command as a canister on the {platform} or on the local canister execution environment. The most common use case is to install all of the canisters by running the following command:

``` bash
dfx canister install --all
```

#### Installing a specific canister

You can also use the `dfx canister install` command to deploy a specific canister instead of all of the canisters in your project. For example, if you have a project with a `hello_world` canister and a `hello_world_assets` canister but only want to deploy the `hello_world` canister, you can deploy just that the canister by running the following command:

``` bash
dfx canister install hello_world
```

#### Sending an asynchronous request

If you want to submit a request to install the canister and return a request identifier to check on the status of your request later instead of waiting for the command to complete, you can run a command similar to the following:

``` bash
dfx canister install hello_world --async
```

This command submits a request to install the canister and returns a request identifier similar to the following:

``` bash
0x58d08e785445dcab4ff090463b9e8b12565a67bf436251d13e308b32b5058608
```

You can then use the request identifier to check the status of the request at a later time, much like a tracking number if you were shipping a package.

#### Overriding the default deployment options

If you want to deploy a canister on a testnet without changing the settings in your `dfx.json` configuration file, you can explicitly specify the testnet you want to connect to by using the `+--network` option.

For example, you can specify a testnet URL by running a command similar to the following:

``` bash
dfx canister --network \http://192.168.3.1:5678 install --all
```

Note that you must specify the network parameter before the canister operation (`install`) and before the canister name or `--all` flag.

#### Allocating message processing

The `--compute-allocation` options allows you to allocate computing resources as a percentage in the range of 0 to 100 to indicate how often your canister should be scheduled for execution.

For example, assume you run the following command:

``` bash
dfx canister install --all --compute-allocation 50
```

With this setting, all of the canisters in the current projects are assigned a 50% allocation. When canisters in the project receive input messages to process, the messages are scheduled for execution. Over 100 execution cycles, each canister’s messages will be scheduled for processing at least 50 times.

The default value for this option is 0—indicating that no specific allocation or scheduling is in effect. If all of your canisters use the default setting, processing occurs in a round-robin fashion.

## dfx canister request-status

Use the `dfx canister request-status` command to request the status of a specified call to a canister. This command requires you to specify the request identifier you received after invoking a method on the canister. The request identifier is an hexadecimal string starting with `0x`.

### Basic usage

``` bash
dfx canister request-status request_id
```

### Flags

You can use the following optional flags with the `dfx canister request-status` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Arguments

You can specify the following argument for the `dfx canister request-status` command.

| Argument     | Description                                                                                                                                                                  |
|--------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `request_id` | Specifies the hexadecimal string returned in response to a `dfx canister call` or `dfx canister install` command. This identifier is an hexadecimal string starting with 0x. |

### Examples

You can use the `dfx canister request-status` command to check on the status of a canister state change or to verify that a call was not rejected by running a command similar to the following:

``` bash
dfx canister request-status 0x58d08e785445dcab4ff090463b9e8b12565a67bf436251d13e308b32b5058608
```

This command displays an error message if the request identifier is invalid or refused by the canister.

## dfx canister set-controller

Use the `dfx canister set-controller` command to specify the identity name or principal to use as the new **controller** for a specified canister on the {platform}. A controller identity has special rights to manage the canister it controls. For example, only a controlling identity can be used to install, upgrade, or delete the canister under its control.

Note that you can specify either a user identity or a canister as a controller. You can also specify the controller identity by using its name or its principal.

### Basic usage

``` bash
dfx canister set-controller [flag] canister new-controller
```

### Flags

You can use the following optional flags with the `dfx canister set-controller` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Arguments

You must use the following arguments with the `dfx canister set-controller` command.

| Argument           | Description                                                                                                                          |
|--------------------|--------------------------------------------------------------------------------------------------------------------------------------|
| `<canister>`       | Specifies the canister name or canister identifier to be controlled by the identity you specify using the *new_controller* argument. |
| `<new_controller>` | Specifies the identity name or principal of the controller.                                                                          |

### Examples

You can use the `dfx canister set-controller` command to specify a user or canister as the controlling identity for a specific canister.

For example, you might create a new identity called `pubsadmin` then run the `dfx canister set-controller` to specify that you want the `pubsadmin` identity to be the controller of the `hello_world` canister by running the following commands:

    dfx identity new pubsadmin
    dfx canister set-controller hello_world pubsadmin

To set the controlling identity using the textual representation of the identity principal, you might run a command similar to the following:

    dfx canister set-controller hello_world wcp5u-pietp-k5jz4-sdaaz-g3x4l-zjzxa-lxnly-fp2mk-j3j77-25qat-pqe

Although specifying a user identity name or principal is one potential use case, a more common scenario is to specify the wallet canister that you want to use to send cycles to the canister. The following steps illustrate this scenario when you are doing local development. For this example, assume you have created a project called `open_sf` with two canisters deployed on the local canister execution environment.

1.  Create an identity—for example, `sf-controller`—to act as the controller.

        dfx identity new sf-controller

        Creating identity: "sf-controller".
        Created identity: "sf-controller".

2.  Make the new identity the active identity.

        dfx identity use sf-controller

        Using identity: "sf-controller".

3.  Generate a wallet canister identifier for the new identity.

        dfx identity get-wallet

        Creating a wallet canister on the local canister execution environment.
        r7inp-6aaaa-aaaaa-aaabq-cai
        The wallet canister on the  the local canister execution environment for user "sf-controller" is "r7inp-6aaaa-aaaaa-aaabq-cai"

4.  Switch the active identity to the current controller of the canister. For example, if the default identity was used to create the canister, you would run the following command:

        dfx identity use default

        Using identity: "default".

5.  Set the controller for a specified canister to use the wallet associated wit the sf-controller identity.

        dfx canister set-controller open_sf_assets r7inp-6aaaa-aaaaa-aaabq-cai

        Set "r7inp-6aaaa-aaaaa-aaabq-cai" as controller of "open_sf_assets".

    You can now use the wallet canister `r7inp-6aaaa-aaaaa-aaabq-cai` to send cycles or add custodians to the `open_sf_assets` canister.

## dfx canister send

Use the `dfx canister send` command after signing a message with the `dfx canister sign` command when you want to separate these steps, rather than using the single `dfx canister call` command. Using separate calls can add security to the transaction.

For example, when creating your neuron stake, you might want to use the `dfx canister sign` command to create a signed `message.json` file using an air-gapped computer, then use the `dfx canister send` command to deliver the signed message.

### Basic usage

``` bash
dfx canister send file_name
```

### Flags

You can use the following optional flags with the `dfx canister request-status` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Arguments

You can specify the following argument for the `dfx canister send` command.

| Argument    | Description                             |
|-------------|-----------------------------------------|
| `file_name` | Specifies the file name of the message. |

### Examples

Use the `dfx canister send` command to send a signed message created using the `dfx canister sign` command to the genesis token canister (GTC) to create a neuron on your behalf by running the following command:

`dfx canister send message.json`

## dfx canister sign

Use the `dfx canister sign` command before sending a message with the `dfx canister send` command when you want to separate these steps, rather than using the single `dfx canister call` command. Using separate calls can add security to the transaction. For example, when creating your neuron stake, you might want to use the `dfx canister sign` command to create a signed `message.json` file using an air-gapped computer, then use the `dfx canister send` command to deliver the signed message from a computer connected to the {platform}.

### Basic usage

``` bash
dfx canister sign [flag] [option] canister-name method-name [argument]
```

### Flags

You can use the following optional flags with the `dfx canister sign` command.

| Flag              | Description                                                                                              |
|-------------------|----------------------------------------------------------------------------------------------------------|
| `-h`, `--help`    | Displays usage information.                                                                              |
| `--query`         | Sends a query request to a canister.                                                                     |
| `--update`        | Sends an update request to the canister. This is the default method if the `--query` method is not used. |
| `-V`, `--version` | Displays version information.                                                                            |

### Options

You can specify the following options for the `dfx canister sign` command.

<!-- <table>
<colgroup>
<col style="width: 32%" />
<col style="width: 68%" />
</colgroup>
<thead>
<tr class="header">
<th style="text-align: left;">Option</th>
<th style="text-align: left;">Description</th>
</tr>
</thead>
<tbody>
<tr class="odd">
<td style="text-align: left;"><p><code>--expire-after &lt;expire-after&gt;</code></p></td>
<td style="text-align: left;"><p>Specifies how long will will be valid before it expires and cannot be sent. Specify in seconds. If not defined, the default is 300s (5m)</p></td>
</tr>
<tr class="even">
<td style="text-align: left;"><p><code>--file &lt;file&gt;</code></p></td>
<td style="text-align: left;"><p>Specifies the output file name. The default is <code>message.json</code>.</p></td>
</tr>
<tr class="odd">
<td style="text-align: left;"><p><code>--random &lt;random&gt;</code></p></td>
<td style="text-align: left;"><p>Specifies the configuration for generating random arguments.</p></td>
</tr>
<tr class="even">
<td style="text-align: left;"><p><code>--type &lt;type&gt;</code></p></td>
<td style="text-align: left;"><p>Specifies the data type for the argument when making a call using an argument.</p>
<p>By default, you can specify arguments using the  (<code>idl</code>) syntax for data values. For information about using Candid and its supported types, see Interact with a service in a terminal</a> and . You can use <code>raw</code> as the argument type if you want to pass raw bytes.</p></td>
</tr>
</tbody>
</table> -->

### Arguments

You can specify the following arguments for the `dfx canister sign` command.

| Argument        | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
|-----------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `canister_name` | Specifies the name of the canister to call. The canister name is a required argument and should match the name you have configured for a project in the `canisters` section of the `dfx.json` configuration file.                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `method_name`   | Specifies the method name to call on the canister. The canister method is a required argument.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `argument`      | Specifies the argument to pass to the method. Depending on your program logic, the argument can be a required or optional argument. You can specify a data format type using the `--type` option if you pass an argument to the canister. By default, you can specify arguments using the [Candid](../candid-ref.md) (`idl`) syntax for data values. For information about using Candid and its supported types, see [Interact with a service in a terminal](../../developer-docs/build/languages/candid/candid-howto#idl-syntax) and [Supported types](../candid-ref#supported-types). You can use `raw` as the argument type if you want to pass raw bytes. |

### Examples

Use the `dfx canister sign` command to create a signed `message.json` file using the principal associated with the identity you created using the Privacy Enhanced Mail (PEM) file by running a command similar to the following:

`dfx canister --network=ic sign --expire-after=1h rno2w-sqaaa-aaaaa-aaacq-cai create_neurons ‘(“PUBLIC_KEY”)’`

This command illustrates how to creates a `message.json` file to create neurons on the {platform} specified by the `ic` alias, that is signed using your principal identifier as the message sender and with an expiration window that ends in one hour.

Note that the time allotted to send a signed message is a fixed 5-minute window. The `--expire-after` option enables you to specify the point in time when the 5-minute window for sending the signed message should end. For example, if you set the `--expire-after` option to one hour (`1h`), you must wait at least 55 minutes before you send the generated message and the signature for the message is only valid during the 5-minute window ending in the 60th minute.

In this example, therefore, you would need to send the message after 55 minutes and before 60 minutes for the message to be recognized as valid.

If you don’t specify the `--expire-after` option, the default expiration is five minutes.

Send the signed message to the genesis token canister (GTC) to create a neuron on your behalf by running the following command:

`dfx canister send message.json`

## dfx canister start

Use the `dfx canister start` command to restart a stopped canister on the {platform} or the local canister execution environment.

In most cases, you run this command after you have stopped a canister to properly terminate any pending requests as a prerequisite to upgrading the canister.

Note that you can only run this command from within the project directory structure. For example, if your project name is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its subdirectories.

### Basic usage

``` bash
dfx canister start [flag] [--all | canister_name]
```

### Flags

You can use the following optional flags with the `dfx canister start` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

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

To start all of the canisters you have deployed on the `ic` {platform}, you can run the following command:

``` bash
dfx canister --network=ic start --all
```

## dfx canister status

Use the `dfx canister status` command to check whether a canister is currently running, in the process of stopping, or currently stopped on the {platform} or on the local canister execution environment.

Note that you can only run this command from within the project directory structure. For example, if your project name is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its subdirectories.

### Basic usage

``` bash
dfx canister status [flag] [--all | canister_name]
```

### Flags

You can use the following optional flags with the `dfx canister status` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

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

To check the status for all of the canisters you have deployed on the `ic` {platform}, you can run the following command:

``` bash
dfx canister --network=ic status --all
```

## dfx canister stop

Use the `dfx canister stop` command to stop a canister that is currently running on the {platform} or on the local canister execution environment.

In most cases, you run this command to properly terminate any pending requests as a prerequisite to upgrading the canister.

Note that you can only run this command from within the project directory structure. For example, if your project name is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its subdirectories.

### Basic usage

``` bash
dfx canister stop [flag] [--all | canister_name]
```

### Flags

You can use the following optional flags with the `dfx canister stop` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Arguments

You can use the following arguments with the `dfx canister stop` command.

| Argument        | Description                                                                                                                      |
|-----------------|----------------------------------------------------------------------------------------------------------------------------------|
| `--all`         | Stops all of the canisters configured in the `dfx.json` file. Note that you must specify `--all` or an individual canister name. |
| `canister_name` | Specifies the name of the canister you want to stop. Note that you must specify either a canister name or the `--all` option.    |

### Examples

You can use the `dfx canister stop` command to start a specific canister or all canisters.

To stop the `hello_world` canister, you can run the following command:

``` bash
dfx canister stop hello_world
```

To stop all of the canisters you have deployed on the `ic` {platform}, you can run the following command:

``` bash
dfx canister --network=ic stop --all
```
