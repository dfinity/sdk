
Use the `dfx nns` subcommands to interact with the Network Nervous System.

The basic syntax for running `dfx nns` commands is:

``` bash
dfx nns [subcommand] [flag]
```

Depending on the `dfx nns` subcommand you specify, additional arguments, options, and flags might apply. For reference information and examples that illustrate using `dfx nns` commands, select an appropriate command.

| Command                             | Description                                                                   |
|-------------------------------------|-------------------------------------------------------------------------------|
| [`import`](#_dfx_nns_import)        | Adds the NNS canisters to the local dfx.json as remote canisters.             |
| [`install`](#_dfx_nns_install)      | Deploys NNS canisters to the local dfx server                                 |
| `help`                              | Displays usage information message for a specified subcommand.                |

To view usage information for a specific subcommand, specify the subcommand and the `--help` flag. For example, to see usage information for `dfx nns install`, you can run the following command:

``` bash
$ dfx nns install --help
```


## dfx nns import

Use the `dfx nns import` command to add the NNS canisters to the local `dfx.json`.  It also downloads the did files and sets the canister IDs of the NNS cansiters so that you can make API calls to NNS canisters.

### Basic usage

``` bash
$ dfx nns import
```

### Flags

You can use the following optional flags with the `dfx nns import` command.

| Flag                | Description                                   |
|---------------------|-----------------------------------------------|
| `-h`, `--help`      | Displays usage information.                   |
| `--network-mapping` | Renames networks when installing canister ids |
| `-V`, `--version`   | Displays version information.                 |

### Examples

You can use the `dfx nns import` command to get did files and so query NNS canisters.

``` bash
$ dfx nns import
$ dfx canister call --network ic nns-governance get_pending_proposals '()'
```

## dfx nns install

Use the `dfx nns install` command to install a local NNS.  This provides local ledger and governance canisters as well as the GUI canisters Internet Identity and NNS-Dapp.

### Basic usage
The local network needs to be set up with a very specific configuration:
```
$ cat ~/.config/dfx/networks.json
{
  "local": {
    "bind": "127.0.0.1:8080",
    "type": "ephemeral",
    "replica": {
      "subnet_type": "system"
    }
  }
}
```

This is because:

* The NNS canisters need to run on a system subnet.
* Some canisters are comiled to run on only very specific canister IDs and hostname/port pairs.


In addition, the local dfx server needs to be clean:

``` bash
$ nohup dfx start --clean
$ dfx nns install
```

This is because NNS canisters need to be installed before any others.


### Flags

You can use the following optional flags with the `dfx nns install` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Examples

An account in the local ledger is initialized with ICP that can be used for testing.  To use the ICP:

* Put this secret key into a file called `ident-1.pem`:
``` bash
$ cat <<EOF >ident-1.pem
-----BEGIN EC PRIVATE KEY-----
MHQCAQEEICJxApEbuZznKFpV+VKACRK30i6+7u5Z13/DOl18cIC+oAcGBSuBBAAK
oUQDQgAEPas6Iag4TUx+Uop+3NhE6s3FlayFtbwdhRVjvOar0kPTfE/N8N6btRnd
74ly5xXEBNSXiENyxhEuzOZrIWMCNQ==
-----END EC PRIVATE KEY-----
EOF
```
* Create an identity with that secret key:
``` bash
$ dfx identity import ident-1 ident-1.pem
```
* Now you can use the (toy) funds:
``` bash
$ dfx ledger balance
```
