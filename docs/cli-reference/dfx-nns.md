
Use the `dfx nns` subcommands to interact with the Network Nervous System.

The basic syntax for running `dfx nns` commands is:

``` bash
dfx nns [subcommand] [flag]
```

Depending on the `dfx nns` subcommand you specify, additional arguments, options, and flags might apply. For reference information and examples that illustrate using `dfx nns` commands, select an appropriate command.

| Command                             | Description                                                                   |
|-------------------------------------|-------------------------------------------------------------------------------|
| [`import`](#_dfx_nns_import)        | Adds the NNS canisters to the local dfx.json as remote canisters.             |
| [`install`](#_dfx_nns_install)      | Deploys NNS canisters to the local dfx server.                                 |
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

| Flag                | Description                                    |
|---------------------|------------------------------------------------|
| `--network-mapping` | Renames networks when installing canister IDs. |

### Examples

You can use the `dfx nns import` command to get did files and so query NNS canisters.

``` bash
$ dfx nns import
$ dfx canister call --network ic nns-governance get_pending_proposals '()'
```

You can rename a network on import.  For example, if you have `test-ic` set up as an alias of the `ic` network then you can set NNS canister IDs for `test-ic` with:

``` bash
$ dfx nns import --network-mapping test-ic=ic
```

## dfx nns install

Use the `dfx nns install` command to install a local NNS. This provides local ledger and governance canisters as well as the GUI canisters Internet Identity and NNS-Dapp.

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
$ dfx start --clean --background
$ dfx nns install
```

This is because NNS canisters need to be installed before any others.


### Examples

#### Example: Making API calls to the local NNS.

``` bash
$ dfx stop
$ dfx start --clean --background
$ dfx nns install
$ dfx nns import
$ dfx canister call --network ic nns-governance get_pending_proposals '()'
```

You can view the API calls that can be made for each NNS canister by looking at the interface definition files installed by `dfx nns import` in `candid/*.did`.  The API methods are in the `service` section, which is usually located at the end of a `.did` file.  It is easiest to start experimenting with methods that take no arguments.

#### Example: Accessing ICP on the command line
Two accounts in the local ledger is initialized with ICP that can be used for testing.  One uses a secp256k1 key, which is convenient for command line usage, another uses an ed25519 key, which is more convenient in web applications.



To use ICP on the command line:
* Start dfx and install the NNS, as described in [`install`](#_dfx_nns_install).
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
* Check the key: (optional)
```
$ openssl ec -in ident-1.pem -noout -text
```
* Create an identity with that secret key:
``` bash
$ dfx identity import ident-1 ident-1.pem
```
* Now you can use the (toy) funds:
``` bash
$ dfx ledger balance
```

To use ICP in an existing web application:
* Install the [@dfinity/agent npm module](https://www.npmjs.com/package/@dfinity/agent).
* Create an identity with this key pair:
```
  const publicKey = "Uu8wv55BKmk9ZErr6OIt5XR1kpEGXcOSOC1OYzrAwuk=";
  const privateKey =
    "N3HB8Hh2PrWqhWH2Qqgr1vbU9T3gb1zgdBD8ZOdlQnVS7zC/nkEqaT1kSuvo4i3ldHWSkQZdw5I4LU5jOsDC6Q==";
  const identity = Ed25519KeyIdentity.fromKeyPair(
    base64ToUInt8Array(publicKey),
    base64ToUInt8Array(privateKey)
  );

  // If using node:
  const base64ToUInt8Array = (base64String: string): Uint8Array => {
    return Buffer.from(base64String, 'base64')
  };
  // If in a browser:
  const base64ToUInt8Array = (base64String: string): Uint8Array => {
    return Uint8Array.from(window.atob(base64String), (c) => c.charCodeAt(0));
  };
```
* That identity can now make API calls, including sending ICP.
