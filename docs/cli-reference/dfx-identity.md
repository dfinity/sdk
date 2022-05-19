# dfx identity

Use the `dfx identity` command with subcommands and flags to manage the identities used to execute commands and communicate with the IC or the local canister execution environment. Creating multiple user identities enables you to test user-based access controls.

The basic syntax for running `dfx identity` commands is:

``` bash
dfx identity [subcommand] [flag]
```

Depending on the `dfx identity` subcommand you specify, additional arguments, options, and flags might apply or be required. To view usage information for a specific `dfx identity` subcommand, specify the subcommand and the `--help` flag. For example, to see usage information for `dfx identity new`, you can run the following command:

``` bash
dfx identity new --help
```

For reference information and examples that illustrate using `dfx identity` commands, select an appropriate command.

| Command                                         | Description                                                                                                               |
|-------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------|
| [`get-principal`](#dfx-identity-get-principal) | Shows the textual representation of the principal associated with the current identity.                                   |
| [`get-wallet`](#dfx-identity-get-wallet)       | Shows the canister identifier for the wallet associated with your current identity principal.                             |
| `help`                                          | Displays this usage message or the help of the given subcommand(s).                                                       |
| [`import`](#dfx-identity-import)               | Creates a new identity by importing a PEM file that contains the key information or security certificate for a principal. |
| [`list`](#dfx-identity-list)                   | Lists existing identities.                                                                                                |
| [`new`](#dfx-identity-new)                     | Creates a new identity.                                                                                                   |
| [`remove`](#dfx-identity-remove)               | Removes an existing identity.                                                                                             |
| [`rename`](#dfx-identity-rename)               | Renames an existing identity.                                                                                             |
| [`set-wallet`](#dfx-identity-set-wallet)       | Sets the wallet canister identifier to use for your current identity principal.                                           |
| [`use`](#dfx-identity-use)                     | Specifies the identity to use.                                                                                            |
| [`whoami`](#dfx-identity-whoami)               | Displays the name of the current identity user context.                                                                   |

## Creating a default identity

The first time you run the `dfx canister create` command to register an identifier, your public/private key pair credentials are used to create a `default` user identity. The credentials for the `default` user are migrated from `$HOME/.dfinity/identity/creds.pem` to `$HOME/.config/dfx/identity/default/identity.pem`.

You can then use `dfx identity new` to create new user identities and store credentials for those identities in `$HOME/.config/dfx/identity/<identity_name>/identity.pem` files. For example, you can create an identity named `ic_admin` by running the following command:

    dfx identity new ic_admin

This command adds a private key for the `ic_admin` user identity in the `~/.config/dfx/identity/ic_admin/identity.pem` file.

## dfx identity get-principal

Use the `dfx identity get-principal` command to display the textual representation of a principal associated with the current user identity context.

If you haven’t created any user identities, you can use this command to display the principal for the `default` user. The textual representation of a principal can be useful for establishing and testing role-based authorization scenarios.

### Basic usage

``` bash
dfx identity get-principal [flag]
```

### Flags

You can use the following optional flags with the `dfx identity get-principal` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Example

If you want to display the textual representation of a principal associated with a specific user identity context, you can run commands similar to the following:

``` bash
dfx identity use ic_admin
dfx identity get-principal
```

In this example, the first command sets the user context to use the `ic_admin` identity. The second command then returns the principal associated with the `ic_admin` identity.

## dfx identity get-wallet

Use the `dfx identity get-wallet` command to display the canister identifier for the wallet associated with your current identity principal.

Note that you must be connected to the IC or the local canister execution environment to run this command. In addition, you must be in a project directory to run the command. For example, if your project name is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its subdirectories to run the `dfx identity get-wallet` command.

### Basic usage

``` bash
dfx identity get-wallet [flag]
```

### Flags

You can use the following optional flags with the `dfx identity get-wallet` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Example

If you want to display the canister identifier for the wallet canister associated with your identity, you can run the following command:

``` bash
dfx identity get-wallet
```

To display the canister identifier for the wallet canister associated with your identity on a specific testnet, you might run a command similar to the following:

``` bash
dfx identity --network=https://192.168.74.4 get-wallet
```

## dfx identity import

Use the `dfx identity import` command to create a user identity by importing the user’s key information or security certificate from a PEM file.

### Basic usage

``` bash
dfx identity import [options] identity-name pem_file-name
```

### Flags

You can use the following optional flags with the `dfx identity import` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Options

You can specify the following options for the `dfx identity import` command.

|Argument|Description|
|--------|-----------|
|`--disable-encryption` |DANGEROUS: By default, PEM files are encrypted with a password when writing them to disk. I you want the convenience of not having to type your password (but at the risk of having your PEM file compromised), you can disable the encryption with this flag.|
|`--force` |If the identity already exists, remove and re-import it.|

### Examples

You can use the `dfx identity import` command to import a PEM file that contains the security certificate to use for an identity. For example, you can run the following command to import the `generated-id.pem` file to create the user identity `alice`:

``` bash
dfx identity import alice generated-id.pem
```

The command adds the `generated-id.pem` file to the `~/.config/dfx/identity/alice` directory.

## dfx identity list

Use the `dfx identity list` command to display the list of user identities available. When you run this command, the list displays an asterisk (\*) to indicate the currently active user context. You should note that identities are global. They are not confined to a specific project context. Therefore, you can use any identity listed by the `dfx identity list` command in any project.

### Basic usage

``` bash
dfx identity list [flag]
```

### Flags

You can use the following optional flags with the `dfx identity list` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Examples

You can use the `dfx identity list` command to list all of the identities you have currently available and to determine which identity is being used as the currently-active user context for running `dfx` commands. For example, you can run the following command to list the identities available:

``` bash
dfx identity list
```

This command displays the list of identities found similar to the following:

``` bash
alice_auth
anonymous
bob_standard *
default
ic_admin
```

In this example, the `bob_standard` identity is the currently-active user context. After you run this command to determine the active user, you know that any additional `dfx` commands you run are executed using the principal associated with the `bob_standard` identity.

## dfx identity new

Use the `dfx identity new` command to add new user identities. You should note that the identities you add are global. They are not confined to a specific project context. Therefore, you can use any identity you add using the `dfx identity new` command in any project.

### Basic usage

``` bash
dfx identity new [options] _identity-name_
```

### Flags

You can use the following optional flags with the `dfx identity new` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Arguments

You must specify the following argument for the `dfx identity new` command.

| Argument          | Description                                                              |
|-------------------|--------------------------------------------------------------------------|
| `<identity_name>` | Specifies the name of the identity to create. This argument is required. |

### Options

You can specify the following options for the `+dfx identity new+` command.

|Argument|Description|
|--------|-----------|
|`--disable-encryption` |DANGEROUS: By default, PEM files are encrypted with a password when writing them to disk. I you want the convenience of not having to type your password (but at the risk of having your PEM file compromised), you can disable the encryption with this flag.|
|`--force` |If the identity already exists, remove and re-import it.|
|`--hsm-key-id <hsm key id>` |A sequence of pairs of hex digits.|
|`--hsm-pkcs11-lib-path <hsm pkcs11 lib path>` |The file path to the opensc-pkcs11 library e.g. "/usr/local/lib/opensc-pkcs11.so"|

### Examples

You can then use `dfx identity new` to create new user identities and store credentials for those identities in `$HOME/.config/dfx/identity/<identity_name>/identity.pem` files. For example, you can create an identity named `ic_admin` by running the following command:

    dfx identity new ic_admin

This command adds a private key for the `ic_admin` user identity in the `~/.config/dfx/identity/ic_admin/identity.pem` file.

After adding the private key for the new identity, the command displays confirmation that the identity has been created:

    Creating identity: "ic_admin".
    Created identity: "ic_admin".

## dfx identity remove

Use the `dfx identity remove` command to remove an existing user identity. You should note that the identities you add are global. They are not confined to a specific project context. Therefore, any identity you remove using the `dfx identity remove` command will no longer be available in any project.

### Basic usage

``` bash
dfx identity remove [flag] _identity-name_
```

### Flags

You can use the following optional flags with the `dfx identity remove` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Arguments

You must specify the following argument for the `dfx identity remove` command.

| Argument          | Description                                                              |
|-------------------|--------------------------------------------------------------------------|
| `<identity_name>` | Specifies the name of the identity to remove. This argument is required. |

### Examples

You can use the `dfx identity remove` command to remove any previously-created identity, including the `default` user identity. For example, if you have added named user identities and want to remove the `default` user identity, you can run the following command:

    dfx identity remove default

The command displays confirmation that the identity has been removed:

    Removing identity "default".
    Removed identity "default".

Although you can delete the `default` identity if you have created other identities to replace it, you must always have at least one identity available. If you attempt to remove the last remaining user context, the `dfx identity remove` command displays an error similar to the following:

    Identity error:
      Cannot delete the default identity

## dfx identity rename

Use the `dfx identity rename` command to rename an existing user identity. You should note that the identities you add are global. They are not confined to a specific project context. Therefore, any identity you rename using the `dfx identity rename` command is available using the new name in any project.

### Basic usage

``` bash
dfx identity rename [flag] _from_identity-name_ _to_identity-name_
```

### Flags

You can use the following optional flags with the `dfx identity rename` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Arguments

You must specify the following arguments for the `dfx identity rename` command.

| Argument               | Description                                                                               |
|------------------------|-------------------------------------------------------------------------------------------|
| `<from_identity_name>` | Specifies the current name of the identity you want to rename. This argument is required. |
| `<to_identity_name>`   | Specifies the new name of the identity you want to rename. This argument is required.     |

### Example

You can rename the `default` user or any identity you have previously created using the `dfx identity rename` command. For example, if you want to rename a `test_admin` identity that you previously created, you would specify the current identity name you want to change **from** and the new name you want to change **to** by running a command similar to the following:

    dfx identity rename test_admin devops

## dfx identity set-wallet

Use the `dfx identity set-wallet` command to specify the wallet canister identifier to use for your identity.

### Basic usage

``` bash
dfx identity set-wallet [flag] [--canister-name canister-name]
```

### Flags

You can use the following optional flags with the `dfx identity set-wallet` command.

| Flag              | Description                                                                                                                                                |
|-------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `force`           | Skips verification that the canister you specify is a valid wallet canister. This option is only useful if you are connecting to the IC running locally. |
| `-h`, `--help`    | Displays usage information.                                                                                                                                |
| `-V`, `--version` | Displays version information.                                                                                                                              |

### Example

If you use more than one principal for your identity, you might have access to more than one wallet canister identifier. You can use the `dfx identity set-wallet` command to specify the wallet canister identifier to use for a given identity.

For example, you might store the wallet canister identifier in an environment variable, then invoke the `dfx identity set-wallet` command to use that wallet canister for additional operations by running the following:

    export WALLET_CANISTER_ID=$(dfx identity get-wallet)
    dfx identity --network=https://192.168.74.4 set-wallet --canister-name ${WALLET_CANISTER_ID}

## dfx identity use

Use the `dfx identity use` command to specify the user identity you want to active. You should note that the identities you have available to use are global. They are not confined to a specific project context. Therefore, you can use any identity you have previously created in any project.

### Basic usage

``` bash
dfx identity use [flag] _identity-name_
```

### Flags

You can use the following optional flags with the `dfx identity use` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Arguments

You must specify the following argument for the `dfx identity use` command.

| Argument          | Description                                                                                                    |
|-------------------|----------------------------------------------------------------------------------------------------------------|
| `<identity_name>` | Specifies the name of the identity you want to make active for subsequent commands. This argument is required. |

### Examples

If you want to run multiple commands with the same user identity context, you can run a command similar to the following:

    dfx identity use ops

After running this command, subsequent commands use the credentials and access controls associated with the `ops` user.

## dfx identity whoami

Use the `dfx identity whoami` command to display the name of the currently-active user identity context.

### Basic usage

``` bash
dfx identity whoami [flag]
```

### Flags

You can use the following optional flags with the `dfx identity whoami` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Example

If you want to display the name of the currently-active user identity, you can run the following command:

``` bash
dfx identity whoami
```

The command displays the name of the user identity. For example, you had previously run the command `dfx identity use bob_standard`, the command would display:

    bob_standard
