# dfx wallet

Use the `dfx wallet` command with subcommands and flags to manage the cycles wallets of your identities and to send cycles to the wallets of other account cycles wallet canisters.

The basic syntax for running the `dfx wallet` commands is:


```bash
dfx wallet [option] <subcommand> [flag]
```

Depending on the `dfx wallet` subcommand you specify, additional arguments, options, and flags might apply or be required.
To view usage information for a specific `dfx wallet` subcommand, specify the subcommand and the `--help` flag.
For example, to see usage information for `dfx wallet send`, you can run the following command:

```bash
dfx wallet send --help
```

For reference information and examples that illustrate using `dfx wallet` commands, select an appropriate command.


|Command |Description
---------|-----------------------------------
|[`add-controller`](#dfx-wallet-add-controller) | Add a controller using the selected identity's principal. |
|[`addresses`](#dfx-wallet-addresses)|Displays the address book of the cycles wallet.|
|[`authorize`](#dfx-wallet-authorize)|Authorize a custodian by principal for the selected identity's cycles wallet|
|[`balance`](#dfx-wallet-balance)|Displays the cycles wallet balance of the selected identity.
|[`controllers`](#dfx-wallet-controllers) |Displays a list of the selected identity's cycles wallet controllers. 
|[`custodians`](#dfx-wallet-custodians) |Displays a list of the selected identity's cycles wallet custodians.
|[`deauthorize`](#dfx-wallet-deauthorize) | Deauthorize a cycles wallet custodian using the custodian's principal.
|`help`|Displays a usage message and the help of the given subcommand(s).
|[`name`](#dfx-wallet-name) |Returns the name of the cycles wallet if you've used the `dfx wallet set-name` command.
|[`redeem-faucet-coupon`](#redeem-faucet-coupon) | Redeem a code at the cycles faucet. |
|[`remove-controller`](#dfx-wallet-remove-controller) |Removes a specified controller from the selected identity's cycles wallet. 
|[`send`](#dfx-wallet-send) |Sends a specified amount of cycles from the selected identity's cycles wallet to another cycles wallet using the destination wallet canister ID.
|[`set-name`](#dfx-wallet-set-name) |Specify a name for your cycles wallet. 
|[`upgrade`](#dfx-wallet-upgrade) |Upgrade the cycles wallet's Wasm module to the current Wasm bundled with DFX.


## Using your wallet

After you have used the `dfx identity deploy-wallet` command to create a cycles wallet canister tied to an identity, you can use `dfx wallet` commands to modify your cycles wallet settings, send cycles to other cycles wallets, and add or remove controllers and custodians. 

## dfx wallet add-controller

Use the `dfx wallet add-controller` to add a controller to the wallet. An identity assigned the role of Controller has the most privileges and can perform the following actions on the selected identity's cycles wallet:

* Rename the cycles wallet.

* Add entries to the address book.

* Add and remove controllers.

* Authorize and deauthorize custodians.

A controller is also a custodian and can perform the following actions associated with that role:

* Access wallet information.

* Send cycles.

* Forward calls.

* Create canisters. 


### Basic usage


```
dfx wallet add-controller [option] <controller> [flag]
```

### Flags

You can use the following optional flags with the `dfx wallet add-controller` command.


|Flag |Description |
------|---------------
|`-h`, `--help` |Displays usage information.
|`-V`, `--version` |Displays version information.

### Options

You can use the following options with the `dfx canister call` command.


|Option |Description|
--------|------------|
|`--network <network>` |Specifies the environment (e.g., Internet Computer or testnet) of the controller you want to add.

### Arguments

You can specify the following arguments for the `dfx wallet add-controller` command.


|Argument |Description
----------|-------------
|`controller` |Specifies the principal of the controller to add to the wallet. 

### Examples

You can use the `dfx wallet add-controller` command to add a controller to your wallet. If the controller you want to add is on a different environment, specify it using the `--network` option. For example:


```
dfx wallet add-controller b5quc-npdph-l6qp4-kur4u-oxljq-7uddl-vfdo6-x2uo5-6y4a6-4pt6v-7qe
```

The command displays output similar to the following:

```
Added b5quc-npdph-l6qp4-kur4u-oxljq-7uddl-vfdo6-x2uo5-6y4a6-4pt6v-7qe as a controller.
```

## dfx wallet addresses

Use the `dfx wallet addresses` command to display the wallet's address book.The address entries contain the principal and `role` (`Contact`, `Custodian`, or `Controller`), and might contain a `name`, and `kind` (`Unknown`, `User`, or `Canister`) associated with the address.

### Basic usage

```
dfx wallet addresses
```

### Flags

You can use the following optional flags with the `dfx wallet add-controller` command.


|Flag |Description
-------|--------------
|`-h`, `--help` |Displays usage information.
|`-V`, `--version` |Displays version information.


### Examples

You can use the `dfx wallet addresses` command to retrieve information on the addresses in your wallet's address book. For example:

```
dfx wallet addresses --network ic
```

The command displays the controllers and custodians for the cycles wallet with output similar to the following:

```
dfx wallet addresses
Id: hpff-grjfd-tg7cj-hfeuj-olrjd-vbego-lpcax-ou5ld-oh7kr-kl9kt-yae, Kind: Unknown, Role: Controller, Name: ic_admin.
Id: e7ptl-4x43t-zxcvh-n6s6c-k2dre-doy7l-bbo6h-ok8ik-msiz3-eoxhl-6qe, Kind: Unknown, Role: Custodian, Name: alice_auth.
```

## dfx wallet authorize

Use the `dfx wallet authorize` command to authorize a custodian for the wallet. An identity authorized as a custodian can perform the following actions on the cycles wallet:

* Access wallet information.

* Send cycles.

* Forward calls.

* Create canisters. 

### Basic usage


```
dfx wallet authorize <custodian> [flag]
```

## Flags

You can use the following optional flags with the `dfx wallet authorize` command.


|Flag |Description
-------|-------
|`-h`, `--help` |Displays usage information.
|`-V`, `--version` |Displays version information.

### Arguments

Use the following necessary argument with the `dfx wallet authorize` command.

|Argument |Description
----------|--------
|`<custodian>` | Specify the principal of the identity you would like to add as a custodian to the selected identity's cycles wallet.

### Example

For example, to add alice_auth as a custodian, specify her principal in the following command:

```
dfx wallet authorize dheus-mqf6t-xafkj-d3tuo-gh4ng-7t2kn-7ikxy-vvwad-dfpgu-em25m-2ae
```

This command outputs something similar to the following:

```
Authorized dheus-mqf6t-xafkj-d3tuo-gh4ng-7t2kn-7ikxy-vvwad-dfpgu-em25m-2ae as a custodian.
```

## dfx wallet balance

Use the `dfx wallet balance` command to display the balance of the cycles wallet of the selected identity. 

### Basic usage


```
dfx wallet balance
```

### Flags

You can use the following optional flags with the `dfx wallet balance` command.

|Flag |Description
-------|---------
|`-h`, `--help` |Displays usage information.
|`-V`, `--version` |Displays version information.
|`--precise` |Displays the exact balance, without scaling to trillions of cycles.

### Examples

Check the balance of the selected identity's cycles wallet.

```
dfx wallet balance
```

This command displays the number of cycles in your cycles wallet. For example: 

```
89.000 TC (trillion cycles).
```

## dfx wallet controllers

Use the `dfx wallet controllers` command to list the principals of the identities that are controllers of the selected identity's cycles wallet. 

### Basic usage


```
dfx wallet controllers
```

### Flags

You can use the following optional flags with the `dfx wallet controllers` command.

|Flag |Description
|`-h`, `--help` |Displays usage information.
|`-V`, `--version` |Displays version information.

### Examples

List the controllers of your selected identity's cycles wallet. 


```
dfx wallet controllers
```

The information returned should look similar to the following if there are two controllers:

```
dheus-mqf6t-xafkj-d3tuo-gh4ng-7t2kn-7ikxy-vvwad-dfpgu-em25m-2ae
hpnmi-qgxsv-tgecj-hmjyn-gmfft-vbego-lpcax-ou4ld-oh7kr-l3nu2-yae
```

## dfx wallet custodians

Use the `dfx wallet custodians` command to list the principals of the identities that are custodians of the selected identity's cycles wallet. Identities that are added as controllers are also listed as custodians.

### Basic usage


```
dfx wallet custodians
```

### Flags

You can use the following optional flags with the `dfx wallet custodians` command.

|Flag |Description
-------|------------
|`-h`, `--help` |Displays usage information.


### Examples

List the custodians of your selected identity's cycles wallet. 


```
dfx wallet custodians
```

The information returned should look similar to the following if there are two custodians:

```
dheus-mqf6t-xafkj-d3tuo-gh4ng-7t2kn-7ikxy-vvwad-dfpgu-em25m-2ae
hpnmi-qgxsv-tgecj-hmjyn-gmfft-vbego-lpcax-ou4ld-oh7kr-l3nu2-yae
```


## dfx wallet deauthorize

Use the `dfx wallet deauthorize` command to remove a custodian from the cycles wallet. 

NOTE:  that this will also remove the role of controller if the custodian is also a controller.

### Basic usage


```
dfx wallet deauthorize <custodian> [flag]
```

### Flags

You can use the following optional flags with the `dfx wallet deauthorize` command.


|Flag |Description
----------|--------------
|`-h`, `--help` |Displays usage information.
|`-V`, `--version` |Displays version information.

### Arguments

Use the following necessary argument with the `dfx wallet deauthorize` command.


|Argument |Description
----------|--------------
|`<custodian>` | Specify the principal of the custodian you want to remove.

### Example

For example, to remove "alice_auth" as a custodian, specify her principal in the following command:


```
dfx wallet deauthorize dheus-mqf6t-xafkj-d3tuo-gh4ng-7t2kn-7ikxy-vvwad-dfpgu-em25m-2ae
```

This command will output something similar to:

```
Deauthorized dheus-mqf6t-xafkj-d3tuo-gh4ng-7t2kn-7ikxy-vvwad-dfpgu-em25m-2ae as a custodian.
```

## dfx wallet name

Use the `dfx wallet name` command to display the name of the selected identity's cycles wallet if it has been set using the `dfx wallet set-name` command. 

### Basic usage


```
dfx wallet name [flag] 
```

### Flags

You can use the following optional flags with the `dfx wallet name` command.

|Flag |Description
-------|-------------
|`-h`, `--help` |Displays usage information.
|`-V`, `--version` |Displays version information.

### Example

If you have named your cycles wallet "Terrances_wallet", then the command would return the following:

```
Terrances_wallet
```

## dfx wallet redeem-faucet-coupon

Use the `dfx wallet redeem-faucet-coupon` command to redeem a cycles faucet coupon.
If you have no wallet set, this will create a wallet for you.
If you have a wallet set already, this will add the coupon's cycles to your existing wallet.

### Basic usage
```
dfx wallet redeem-faucet-coupon <your faucet coupon>
```

### Arguments

Use the following necessary argument with the `dfx wallet redeem-faucet-coupon` command.


|Argument |Description
----------|--------------
|`<your faucet coupon>` | The coupon code to redeem at the faucet.|


### Flags

You can use the following optional flags with the `dfx wallet redeem-faucet-coupon` command.


|Flag |Description|
|-----|-----------|
|`--faucet`|Alternative faucet address. If not set, this uses the DFINTITY faucet.|
|`-h`, `--help` |Displays usage information.|
|`-V`, `--version` |Displays version information.|

### Example

If you have no wallet yet and a coupon code `ABCDE-ABCDE-ABCDE`, you can redeem it like this:
``` bash
dfx wallet redeem-faucet-coupon 'ABCDE-ABCDE-ABCDE'
```

This will print something similar to this:
```
Redeemed coupon ABCDE-ABCDE-ABCDE for a new wallet: rdmx6-jaaaa-aaaaa-aaadq-cai
New wallet set.
```

If you have a wallet already and a coupon code `ABCDE-ABCDE-ABCDE`, you can redeem it like this:
``` bash
dfx wallet redeem-faucet-coupon 'ABCDE-ABCDE-ABCDE'
```

This will print something similar to this:
```
Redeemed coupon code ABCDE-ABCDE-ABCDE for 20.000 TC (trillion cycles).
```

## dfx wallet remove-controller

Use the `dfx wallet remove-controller` command to remove a controller of your selected identity's cycles wallet.

### Basic usage


```
dfx wallet remove-controller <controller> [flag]
```

### Flags

You can use the following optional flags with the `dfx wallet remove-controller` command.

|Flag |Description
-----------|----------
|`-h`, `--help` |Displays usage information.
|`-V`, `--version` |Displays version information.


### Arguments

Use the following necessary argument with the `dfx wallet remove-controller` command.

|Argument |Description

|`<controller>` | Specify the principal of the controller you want to remove.


### Example

For example, to remove alice_auth as a controller, specify her principal in the following command:


```
dfx wallet remove-controller dheus-mqf6t-xafkj-d3tuo-gh4ng-7t2kn-7ikxy-vvwad-dfpgu-em25m-2ae
```
The command outputs something similar to the following:
```
Removed dheus-mqf6t-xafkj-d3tuo-gh4ng-7t2kn-7ikxy-vvwad-dfpgu-em25m-2ae as a controller.
```

## dfx wallet send

Use the `dfx wallet send` command to send cycles from the selected identity's cycles wallet to another cycles wallet using the destination cycle wallet's Canister ID. Keep in mind that the receiving canister must be a cycles wallet or have a `wallet_receive` method to accept the cycles.

### Basic usage


```
dfx wallet [network] send [flag] <destination> <amount> 
```

### Flags

You can use the following optional flags with the `dfx wallet send` command.


|Flag |Description
-----------|----------
|`-h`, `--help` |Displays usage information.
|`-V`, `--version` |Displays version information.

### Options

You can use the following option with the `dfx wallet send` command.

|Option |Description
-----------|----------
|`--network` |Override the environment to connect to. By default, the local canister execution environment is used. A valid URL (starting with `http:` or `https:`) can be specified here. E.g. "http://localhost:12345/" is a valid network name.

### Arguments

You must specify the following arguments for the `dfx wallet send` command.

|Argument |Description
-----------|----------
|`<destination>` |Specify the destination cycle wallet using its Canister ID.
|`<amount>` |Specify the number of cycles to send.

### Examples

Send cycles from the selected identity's cycles wallet to another cycles wallet.

For example, to send 2,000,000,000 cycles from the cycles wallet of the selected identity, `<ic_admin>`, to the cycles wallet of the destination identity, `<buffy_standard>` with a wallet address `r7inp-6aaaa-aaaaa-aaabq-cai`, run the following command:


```
dfx wallet send r7inp-6aaaa-aaaaa-aaabq-cai 2000000000
```

If the transfer is successful, the command does not displays any output.

## dfx wallet set-name

Use the `dfx wallet set-name` command to assign a name to the selected identity's cycles wallet.

### Basic usage

```
    dfx wallet set-name [flag] <name> 
```

### Arguments

You must specify the following arguments for the `dfx wallet set-name` command.

|Argument |Description
--------|------------
|`<name>` |Specify a name for the cycles wallet.


### Flags

You can use the following optional flags with the `dfx wallet set-name` command.

|Flag |Description
-------|-----------
|`-h`, `--help` |Displays usage information.
|`-V`, `--version` |Displays version information.


### Example

If you want to set the name of the current identity's cycles wallet to "Terrances_wallet" you can run the following command:


```
dfx wallet set-name Terrances_wallet
```

## dfx wallet upgrade

Use the `dfx wallet upgrade` command to upgrade the cycle wallet's Wasm module to the current Wasm bundled with DFX.

### Basic usage


```
    dfx wallet upgrade [flag] 
```

### Flags

You can use the following optional flags with the `dfx wallet upgrade` command.

|Flag |Description
-------|------------
|`-h`, `--help` |Displays usage information.
|`-V`, `--version` |Displays version information.

### Example
To upgrade the Wasm module to the latest version, run the following command:

```
dfx wallet upgrade
```
