# dfx changelog

# UNRELEASED

## DFX

### fix(generate): add missing typescript types and fix issues with bindings array in dfx.json

### chore: update Candid UI canister with commit 79d55e7f568aec00e16dd0329926cc7ea8e3a28b

### refactor: Factor out code for calling arbitrary bundled binaries

The function for calling sns can now call any bundled binary.

### fix: Only kill main process on `dfx stop`
Removes misleading panics when running `dfx stop`.

### feat: `dfx nns install` works offline if all assets have been cached.

### feat: Initialise the nns with an account controlled by a secp256k1 key

This enables easy access to toy ICP using command line tools and this key:
```
-----BEGIN EC PRIVATE KEY-----
MHQCAQEEICJxApEbuZznKFpV+VKACRK30i6+7u5Z13/DOl18cIC+oAcGBSuBBAAK
oUQDQgAEPas6Iag4TUx+Uop+3NhE6s3FlayFtbwdhRVjvOar0kPTfE/N8N6btRnd
74ly5xXEBNSXiENyxhEuzOZrIWMCNQ==
-----END EC PRIVATE KEY-----
```
For example, you can create an identity in dfx by putting this key in the file `ident-1.pem` and importing it:
```
dfx identity import ident-1 ident-1.pem
dfx --identity ident-1 ledger balance
```

### feat: default to run ic-wasm shrink when build canisters
This behavior applies to Motoko, Rust and Custom canisters.
If you want to disable this behavior, you can config it in dfx.json:

    "canisters" : {
        "app" : {
            "shrink" : false,
        }
    }

### fix: Valid canister-based env vars

Hyphens are not valid in shell environment variables, but do occur in canister names such as `smiley-dapp`. This poses a problem for vars with names such as `CANISTER_ID_${CANISTER_NAME}`.  With this change, hyphens are replaced with underscores in environment variables.  The canister id of `smiley-dapp` will be available as `CANISTER_ID_smiley_dapp`.  Other environment variables are unaffected.

### feat: Add dfx sns deploy

This allows users to deploy a set of SNS canisters.

### fix: `cargo run -p dfx -- --version` prints correct version

### feat: add --mode=auto

When using `dfx canister install`, you can now pass `auto` for the `--mode` flag, which will auto-select `install` or `upgrade` depending on need, the same way `dfx deploy` does. The default remains `install` to prevent mistakes.

### feat: add `--network` flag to `dfx generate`

`dfx generate`'s generated bindings use network-specific canister IDs depending on the generated language, but there was previously no way to configure which network this was, so it defaulted to local. A `--network` flag has been added for this purpose.

### feat: sns config validate

There is a new command that verifies that an SNS initialization config is valid.

### feat: sns config create

There is a new command that creates an sns config template.

### fix: remove $ from wasms dir

The wasms dir path had a $ which is unwanted and now gone.

### fix: Correct wasm for the SNS swap canister

Previously the incorrect wasm canister was installed.

### fix: Use NNS did files that matches the wasms

Previously the did files and wasms could be incompatible.

### fix: allow users to skip compatibility check if parsing fails

### feat: canister HTTP support is now enabled by default.

`dfx start` and `dfx replica` now ignore the `--enable-canister-http` parameter.

You can still disable the canister http feature through configuration:
- ~/.config/dfx/networks.json: `.local.canister_http.enabled=false`
- dfx.json (project-specific networks) : `.networks.local.canister_http.enabled=false`

### feat: custom canister `wasm` field can now specify a URL from which to download

### feat: custom canister `candid` field can now specify a URL from which to download

### feat: deploy NNS canisters

A developer is now able to install NNS canisters, including back end canisters such as ledger and governance, and front end canisters such as nns-dapp and internet-identity, on their local DFX server.  Usage:
```
dfx start --clean --background
dfx nns install
```

This feature currently requires that the network 'local' is used and that it runs on port 8080.
The network's port can be controlled by using the field `"provider"` in the network's definition, e.g. by setting it to `"127.0.0.1:8080"`.

### feat: configure logging level of http adapter

It is now possible to set the http adapter's log level in dfx.json or in networks.json:

    "http": {
      "enabled": true,
      "log_level": "info"
    }

By default, a log level of "error" is used, in order to keep the output of a first-time `dfx start` minimal. Change it to "debug" for more verbose logging.

### fix(typescript): add index.d.ts file for type safety when importing generated declarations

Adds an index.d.ts file to the generated declarations, allowing for better type safety in TypeScript projects.

### chore: reduce verbosity of dfx start

`dfx start` produces a lot of log output that is at best irrelevant for most users.
Most output is no longer visible unless either `--verbose` is used with dfx or the relevant part's (e.g. http adapter, btc adapter, or replica) log level is changed in dfx.json or networks.json.

### feat: generate secp256k1 keys by default

When creating a new identity with `dfx identity new`, whereas previously it would have generated an Ed25519 key, it now generates a secp256k1 key. This is to enable users to write down a BIP39-style seed phrase, to recover their key in case of emergency, which will be printed when the key is generated and can be used with a new `--seed-phrase` flag in `dfx identity import`. `dfx identity import` is however still capable of importing an Ed25519 key.

### chore: update Candid UI canister with commit 528a4b04807904899f67b919a88597656e0cd6fa

* Allow passing did files larger than 2KB.
* Better integration with Motoko Playground.

### feat: simplify verification of assets served by asset canister

* SHA256 hashes of all assets are displayed when deploying the asset canister.
* A query method is added to the asset canister that returns the entire asset hash tree together with the certificate containing the certified variables of the asset canister.

### breaking change: dfx canister update-settings --compute-allocation always fails

See https://forum.dfinity.org/t/fixing-incorrect-compute-allocation-fee/14830

Until the rollout is complete, `dfx canister update-settings --compute-allocation <N>`
will fail with an error from the replica such as the following:
```
The Replica returned an error: code 1, message: "Canister requested a compute allocation of 1% which cannot be satisfied because the Subnet's remaining compute capacity is 0%"
```

### fix: For default node starter template: copy `ic-assets.json5` file from `src` to `dist`

### fix: For `dfx start --clean --background`, the background process no longer cleans a second time.

### refactor: Move replica URL functions into a module for reuse

The running replica port and url are generally useful information. Previously the code to get the URL was embedded in the network proxy code. This moves it out into a library for reuse.

### chore: Frontend canister build process no longer depends on `dfx` or `ic-cdk-optimizer`

Instead, the build process relies on `ic-wasm` to provide candid metadata for the canister, and
shrinking the canister size by stripping debug symbols and unused fuctions.
Additionally, after build step, the `.wasm` file is archived with `gzip`. 

### chore: Move all `frontend canister`-related code into the SDK repo

| from (`repository` `path`)                  | to (path in `dfinity/sdk` repository)          | summary                                                                                     |
|:--------------------------------------------|:-----------------------------------------------|:--------------------------------------------------------------------------------------------|
| `dfinity/cdk-rs` `/src/ic-certified-assets` | `/src/canisters/frontend/ic-certified-asset`   | the core of the frontend canister                                                           |
| `dfinity/certified-assets` `/`              | `/src/canisters/frontend/ic-frontend-canister` | wraps `ic-certified-assets` to build the canister wasm                                      |
| `dfinity/agent-rs` `/ic-asset`              | `/src/canisters/frontend/ic-asset`             | library facilitating interactions with frontend canister (e.g. uploading or listing assets) |
| `dfinity/agent-rs` `/icx-asset`             | `/src/canisters/frontend/icx-asset`            | CLI executable tool - wraps `ic-asset`                                                      |

### feat: use JSON5 file format for frontend canister asset configuration

Both `.ic-assets.json` and `.ic-assets.json5` are valid filenames config filename, though both will get parsed
as if they were [JSON5](https://json5.org/) format. Example content of the `.ic-assets.json5` file:
```json5
// comment
[
  {
    "match": "*", // comment
    /*
    keys below not wrapped in quotes
*/  cache: { max_age: 999 }, // trailing comma 
  },
]
```
- Learn more about JSON5: https://json5.org/

### fix: Update nns binaries unless `NO_CLOBBER` is set

Previously existing NNS binaries were not updated regardless of the `NO_CLOBBER` setting.

### feat!: Support installing canisters not in dfx.json

`install_canister_wasm` used to fail if installing a canister not listed in dfx.json.  This use case is now supported.

### feat: print the dashboard URL on startup

When running `dfx start` or `dfx replica`, the path to the dashboard page is now printed.

### feat!: changed the default port of the shared local network from 8000 to 4943.

This is so dfx doesn't connect to a project-specific network instead of the local shared network.

In combination with the "system-wide dfx start" feature, there is a potential difference to be aware of with respect to existing projects.

Since previous versions of dfx populate dfx.json with a `networks.local` definition that specifies port 8000, the behavior for existing projects won't change.

However, if you've edited dfx.json and removed the `networks.local` definition, the behavior within the project will change: dfx will connect to the shared local network on port 4943 rather than to the project-specific network on port 8000.  You would need to edit webpack.config.js to match.  If you have scripts, you can run the new command `dfx info webserver-port` from the project directory to retrieve the port value.

### feat!: "system-wide dfx start"

By default, dfx now manages the replica process in a way that is independent of any given dfx project.  We've called this feature "system-wide dfx", even though it's actually specific to your user
(storing data files under $HOME), because we think it communicates the idea adequately.

The intended benefits:
- deploying dapps from separate projects alongside one another, similar to working with separate dapps on mainnet
- run `dfx start` from any directory
- run `dfx stop` from any directory, rather than having to remember where you last ran `dfx start`

We're calling this the "shared local network."  `dfx start` and `dfx stop` will manage this network when run outside any project directory, or when a project's dfx.json does not define the `local` network.  The dfx.json template for new projects no longer defines any networks.

We recommend that you remove the `local` network definition from dfx.json and instead use the shared local network.  As mentioned above, doing so will make dfx use port 4943 rather than port 8000.

See [Local Server Configuration](docs/cli-reference/dfx-start.md#local-server-configuration) for details.

dfx now stores data and control files in one of three places, rather than directly under `.dfx/`:
- `.dfx/network/local` (for projects in which dfx.json defines the local network)
- `$HOME/.local/share/dfx/network/local` (for the shared local network on Linux)
- `$HOME/Library/Application Support/org.dfinity.dfx/network/local` (for the shared local network on MacOS)

There is also a new configuration file: `$HOME/.config/dfx/networks.json`.  Its [schema](docs/networks-json-schema.json) is the same as the `networks` element in dfx.json.  Any networks you define here will be available from any project, unless a project's dfx.json defines a network with the same name.  See [The Shared Local Network](docs/cli-reference/dfx-start.md#the-shared-local-network) for the default definitions that dfx provides if this file does not exist or does not define a `local` network.

### fix: `dfx start` and `dfx stop` will take into account dfx/replica processes from dfx <= 0.11.x

### feat: added command `dfx info`

#### feat: `dfx info webserver-port`

This displays the port that the icx-proxy process listens on, meaning the port to connect to with curl or from a web browser.

#### feat: `dfx info replica-rev`

This displays the revision of the replica bundled with dfx, which is the same revision referenced in replica election governance proposals.

#### feat: `dfx info networks-json-path`

This displays the path to your user's `networks.json` file where all networks are defined.

### feat: added ic-nns-init, ic-admin, and sns executables to the binary cache

### fix: improved responsiveness of `greet` method call in default Motoko project template

`greet` method was marked as an `update` call, but it performs no state updates. Changing it to `query` call will result in faster execution.

### feat: dfx schema --for networks

The `dfx schema` command can now display the schema for either dfx.json or for networks.json.  By default, it still displays the schema for dfx.json.

```bash
dfx schema --for networks
```

### feat: createActor options accept pre-initialized agent

If you have a pre-initialized agent in your JS code, you can now pass it to createActor's options. Conflicts with the agentOptions config - if you pass both the agent option will be used and you will receive a warning.

```js
const plugActor = createActor(canisterId, {
  agent: plugAgent
})
```

### feat!: option for nodejs compatibility in dfx generate

Users can now specify `node_compatibility: true` in `declarations`. The flag introduces `node.js` enhancements, which include importing `isomorphic-fetch` and configuring the default actor with `isomorphic-fetch` and `host`.

```json
// dfx.json
"declarations": {
  "output": "src/declarations",
  "node_compatibility": true
}
```

#### JS codegen location deprecation

DFX new template now uses `dfx generate` instead of `rsync`. Adds deprecation warning to `index.js` in `.dfx/<network-name>/<canister-name>` encouringing developers to migrate to the `dfx generate` command instead. If you have a `package.json` file that uses `rsync` from `.dfx`, consider switching to something like this:

```json
"scripts": {
  "build": "webpack",
  "prebuild": "npm run generate",
  "start": "webpack serve --mode development --env development",
  "prestart": "npm run generate",
  // It's faster to only generate canisters you depend on, omitting the frontend canister
  "generate": "dfx generate hello_backend"
},
```

### feat: simple cycles faucet code redemption

Using `dfx wallet --network ic redeem-faucet-coupon <my coupon>` faucet users have a much easier time to redeem their codes.
If the active identity has no wallet configured, the faucet supplies a wallet to the user that this command will automatically configure.
If the active identity has a wallet configured already, the faucet will top up the existing wallet.

Alternative faucets can be used, assuming they follow the same interface. To direct dfx to a different faucet, use the `--faucet <alternative faucet id>` flag.
The expected interface looks like the following candid functions:
``` candid
redeem: (text) -> (principal);
redeem_to_wallet: (text, principal) -> (nat);
```
The function `redeem` takes a coupon code and returns the principal to an already-installed wallet that is controlled by the identity that called the function.
The function `redeem_to_wallet` takes a coupon code and a wallet (or any other canister) principal, deposits the cycles into that canister and returns how many cycles were deposited.

### feat: disable automatic wallet creation on non-ic networks

By default, if dfx is not running on the `ic` (or networks with a different name but the same configuration), it will automatically create a cycles wallet in case it hasn't been created yet.
It is now possible to inhibit automatic wallet creation by setting the `DFX_DISABLE_AUTO_WALLET` environment variable.

### fix!: removed unused --root parameter from dfx bootstrap

### feat: canister installation now waits for the replica

When installing a new WASM module to a canister, DFX will now wait for the updated state (i.e. the new module hash) to be visible in the replica's certified state tree before proceeding with post-installation tasks or producing a success status.

### feat!: remove `dfx config`

`dfx config` has been removed. Please update Bash scripts to use `jq`, PowerShell scripts to use `ConvertTo-Json`, nushell scripts to use `to json`, etc.

### feat: move all the flags to the end

Command flags have been moved to a more traditional location; they are no longer positioned per subcommand, but instead are able to be all positioned after the final subcommand. In prior versions, a command might look like:
```bash
dfx --identity alice canister --network ic --wallet "$WALLET" create --all
```
This command can now be written:
```bash
dfx canister create --all --network ic --wallet "$WALLET" --identity alice
```
The old syntax is still available, though, so you don't need to migrate your scripts.

### feat!: changed update-settings syntax

When using `dfx canister update-settings`, it is easy to mistake `--controller` for `--add-controller`. For this reason `--controller` has been renamed to `--set-controller`.

### feat!: removed the internal webserver

This is a breaking change.  The only thing this was still serving was the /_/candid endpoint.  If you need to retrieve the candid interface for a local canister, you can use `dfx canister metadata <canister> candid:service`.

### fix: dfx wallet upgrade: improved error messages:

- if there is no wallet to upgrade
- if trying to upgrade a local wallet from outside of a project directory

### fix: canister creation cost is 0.1T cycles

Canister creation fee was calculated with 1T cycles instead of 0.1T.

### fix: dfx deploy and dfx canister install write .old.did files under .dfx/

When dfx deploy and dfx canister install upgrade a canister, they ensure that the
new candid interface is compatible with the previous candid interface.  They write
a file with extension .old.did that contains the previous interface.  In some
circumstances these files could be written in the project directory.  dfx now
always writes them under the .dfx/ directory.

### fix: dfx canister install now accepts arbitrary canister ids

This fixes the following error:
``` bash
> dfx canister install --wasm ~/counter.wasm eop7r-riaaa-aaaak-qasxq-cai
Error: Failed while determining if canister 'eop7r-riaaa-aaaak-qasxq-cai' is remote on network 'ic'.
Caused by: Failed while determining if canister 'eop7r-riaaa-aaaak-qasxq-cai' is remote on network 'ic'.
  Failed to figure out if canister 'eop7r-riaaa-aaaak-qasxq-cai' has a remote id on network 'ic'.
    Invalid argument: Canister eop7r-riaaa-aaaak-qasxq-cai not found in dfx.json
```

### feat: allow replica log level to be configured

It is now possible to specify the replica's log level. Possible values are `critical`, `error`, `warning`, `info`, `debug`, and `trace`.
The log level defaults to the level 'error'. Debug prints (e.g. `Debug.print("...")` in Motoko) still show up in the console.
The log level can be specified in the following places (See [system-wide dfx start](#feat-system-wide-dfx-start) for more detailed explanations on the network types):
- In file `networks.json` in the field `<network name>.replica.log_level` for shared networks.
- In file `dfx.json` in the field `networks.<network name>.replica.log_level` for project-specific networks.
- In file `dfx.json` in the field `defaults.replica.log_level` for project-specific networks. Requires a project-specific network to be run, otherwise this will have no effect.

### feat: enable canister sandboxing

Canister sandboxing is enabled to be consistent with the mainnet.

### chore: dfx ledger account-id --of-canister also accepts principal

It is now possible to do e.g. `dfx ledger account-id --of-canister fg7gi-vyaaa-aaaal-qadca-cai` as well as `dfx ledger account-id --of-canister my_canister_name` when checking the ledger account id of a canister.
Previously, dfx only accepted canister aliases and produced an error message that was hard to understand.

### chore: dfx canister deposit-cycles uses default wallet if none is specified

Motivated by [this forum post](https://forum.dfinity.org/t/dfx-0-10-0-dfx-canister-deposit-cycles-returns-error/13251/6).

### chore: projects created with `dfx new` are not pinned to a specific dfx version anymore

It is still possible to pin the dfx version by adding `"dfx":"<dfx version to pin to>"` to a project's `dfx.json`.

### fix: print links to cdk-rs docs in dfx new

### fix: Small grammar change to identity password decryption prompt

The prompt for entering your passphrase in order to decrypt an identity password read:
    "Please enter a passphrase for your identity"
However, at that point, it isn't "a" passphrase.  It's either your passphrase, or incorrect.
Changed the text in this case to read:
    "Please enter the passphrase for your identity"

### chore: add retry logic to dfx download script

### feat: Add subnet type argument when creating canisters

`dfx ledger create-canister` gets a new option `--subnet-type` that allows users to choose a type of subnet that their canister can be created on. Additionally, a `dfx ledger show-subnet-types` is introduced which allows to list the available subnet types.

## Dependencies

### Replica

Updated replica to release candidate at commit 9173c5f1b28e140931060b90e9de65b923ee57e6.
This release candidate has not yet been elected.

This also incorporates the following executed proposals:

* [81788](https://dashboard.internetcomputer.org/proposal/81788)
* [81571](https://dashboard.internetcomputer.org/proposal/81571)
* [80992](https://dashboard.internetcomputer.org/proposal/80992)
* [79816](https://dashboard.internetcomputer.org/proposal/79816)
* [78693](https://dashboard.internetcomputer.org/proposal/78693)
* [77589](https://dashboard.internetcomputer.org/proposal/77589)
* [76228](https://dashboard.internetcomputer.org/proposal/76228)
* [75700](https://dashboard.internetcomputer.org/proposal/75700)
* [75109](https://dashboard.internetcomputer.org/proposal/75109)
* [74395](https://dashboard.internetcomputer.org/proposal/74395)
* [73959](https://dashboard.internetcomputer.org/proposal/73959)
* [73714](https://dashboard.internetcomputer.org/proposal/73714)
* [73368](https://dashboard.internetcomputer.org/proposal/73368)
* [72764](https://dashboard.internetcomputer.org/proposal/72764)

### ic-ref

Updated ic-ref to 0.0.1-1fba03ee
- introduce awaitKnown
- trivial implementation of idle_cycles_burned_per_day

### Updated Motoko to 0.7.0

### Cycles wallet

- Module hash: b944b1e5533064d12e951621d5045d5291bcfd8cf9d60c28fef02c8fdb68e783
- https://github.com/dfinity/cycles-wallet/commit/fa86dd3a65b2509ca1e0c2bb9d7d4c5be95de378

# 0.11.2

## DFX

### fix: disable asset canister redirection of all HTTP traffic from `.raw.ic0.app` to `.ic0.app`

### fix: disable asset canister's ETag HTTP headers

The feature is not yet implemented on `icx-proxy`-level, and is causing 500 HTTP response for some type of assets every second request.

# 0.11.1

## DFX

### fix: dfx now only adds candid:service metadata to custom canisters that have at least one build step

This way, if a canister uses a premade canister wasm, dfx will use it as-is.

### fix: "canister alias not defined" in the Motoko language server

It is now possible to develop multiple-canister projects using the [Motoko VSCode extension](https://marketplace.visualstudio.com/items?itemName=dfinity-foundation.vscode-motoko).

### fix: improve browser compatibility for the JavaScript language binding

Patches a JavaScript language binding compatibility issue encountered in web browsers which do not support the (?.) operator.

### feat: print dfx.json schema

dfx is now capable of displaying the schema for `dfx.json`. You can see the schema by running `dfx schema` or write the schema to a file with `dfx schema --outfile path/to/file/schema.json`.

### feat: support for configuring assets in assets canister
- The `.ic-assets.json` file should be placed inside directory with assets, or its subdirectories. Multiple config files can be used (nested in subdirectories). Example of `.ic-assets.json` file format:
``` json
[
    {
        "match": ".*",
        "cache": {
            "max_age": 20
        },
        "headers": {
            "X-Content-Type-Options": "nosniff"
        },
        "ignore": false
    },
    {
        "match": "**/*",
        "headers": null
    },
    {
        "match": "file.json",
        "ignore": true
    }
]
```
- Configuring assets works only during asset creation - any changes to `.ic-assets.json` files won't have any effect effect for assets that have already been created. We are working on follow up implementation with improvements to handle updating these properties.
- `headers` from multiple applicable rules are being stacked/concatenated, unless `null` is specified, which resets/empties the headers.
- Both `"headers": {}` and absence of `headers` field don't have any effect on end result.
- Valid JSON format is required, i.e. the array of maps, `match` field is required. Only the following fields are accepted: `cache`, `ignore`, `headers`, `match`. The glob pattern has to be valid.
- The way matching rules work:
  1. The most deeply nested config file takes precedence over the one in parent dir. In other words, properties from a rule matching a file in a subdirectory override properties from a rule matching a file in a parent directory
  2. Order of rules within file matters - last rule in config file takes precedence over the first one

- The way `ignore` field works:
  1. By default, files that begin with a `.` are ignored, while all other files are included.
  2. The `.ignore` field overrides this, if present.
  3. If a directory is ignored, file and directories within it cannot be un-ignored.
  4. A file can be ignored and un-ignored many times, as long as any of its parent directories haven't been ignored.


### fix: Allow `dfx deploy` to not take arguments for canisters not being installed

A longstanding bug with `dfx deploy` is that if an installation is skipped (usually an implicitly included dependency), it still requires arguments even if the installed canister doesn't. As of this release that bug is now fixed.

### feat: Add additional logging from bitcoin canister in replica.

Configures the replica to emit additional logging from the bitcoin canister whenever the bitcoin feature is enabled. This helps show useful information to developers, such as the bitcoin height that the replica currently sees.

### fix: make `build` field optional for custom canisters

Prior to 0.11.0, a custom canister's `build` field could be left off if `dfx build` was never invoked. To aid in deploying prebuilt canisters, this behavior is now formalized; omitting `build` is equivalent to `build: []`.

### feat: Use `--locked` for Rust canisters

`dfx build`, in Rust canisters, now uses the `--locked` flag when building with Cargo. To offset this, `dfx new --type rust` now runs `cargo update` on the resulting project.

### feat: Enable threshold ecdsa signature

ECDSA signature signing is now enabled by default in new projects, or by running `dfx start --clean`.
A test key id "Secp256k1:dfx_test_key" is ready to be used by locally created canisters.

## Dependencies

### Updated `agent-rs` to 0.20.0

### Updated `candid` to 0.7.15

### Replica

Updated replica to elected commit 6e86169e98904047833ba6133e5413d2758d90eb.
This incorporates the following executed proposals:

* [72225](https://dashboard.internetcomputer.org/proposal/72225)
* [71669](https://dashboard.internetcomputer.org/proposal/71669)
* [71164](https://dashboard.internetcomputer.org/proposal/71164)
* [70375](https://dashboard.internetcomputer.org/proposal/70375)
* [70002](https://dashboard.internetcomputer.org/proposal/70002)

# 0.11.0

## DFX

### feat: renamed canisters in new projects to <project>_frontend and <project>_backend

The names of canisters created for new projects have changed.
After `dfx new <project>`, the canister names are:

- `<project>_backend` (previously `<project>`)
- `<project>_frontend` (previously `<project>_assets`)

### feat: Enable threshold ecdsa signature

### feat: new command: dfx canister metadata <canister> <name>

For example, to query a canister's candid service definition: `dfx canister metadata hello_backend candid:service`

### refactor: deprecate /_/candid internal webserver

The dfx internal webserver now only services the /_/candid endpoint.  This
is now deprecated.  If you were using this to query candid definitions, you
can instead use `dfx canister metadata`.

### refactor: optimize from ic-wasm

Optimize Rust canister WASM module via ic-wasm library instead of ic-cdk-optimizer. A separate installation of ic-cdk-optimizer is no longer needed.

The actual optimization was kept the same.

### feat: Read dfx canister call argument from a file or stdin

Enables passing large arguments that cannot be passed directly in the command line using the `--argument-file` flag. For example:
 * Named file: `dfx canister call --argument-file ./my/argument/file.txt my_canister_name greet`
 * Stdin: `echo '( null )' | dfx canister call --argument-file - my_canister_name greet`

### fix: Use default setting for BTC adapter idle seconds

A lower threshold was no longer necessary.

### feat: Allow users to configure logging level of bitcoin adapter

The bitcoin adapter's logging can be very verbose if debug logging is enabled, making it difficult to make sense of what's going on. On the other hand, these logs are useful for triaging problems.

To get the best of both worlds, this release adds support for an additional configuration option in dfx.json:

    "bitcoin": {
      "enabled": true,
      "nodes": ["127.0.0.1:18444"],
      "log_level": "info" <------- users can now configure the log level
    }

By default, a log level of "info" is used, which is relatively quiet. Users can change it to "debug" for more verbose logging.

### chore: update Candid UI canister with commit bffa0ae3c416e8aa3c92c33722a6b1cb31d0f1c3

This includes the following changes:

* Fetch did file from canister metadata
* Better flamegraph support
* Fix bigint error for vec nat8 type

### feat: dfx will look up the port of the running webserver from .dfx/webserver-port, if present

After `dfx start --host 127.0.0.1:0`, the dfx webserver will listen on an ephemeral port.  It stores the port value in .dfx/webserver-port.  dfx will now look for this file, and if a port is contained within, use that port to connect to the dfx webserver.

### fix: dfx commands once again work from any subdirectory of a dfx project

Running `dfx deploy`, `dfx canister id`, `dfx canister call` and so forth work as expected
if run from within any subdirectory of a dfx project.  Previously, this would create
canister_ids.json or .dfx/local/canister_ids.json within the subdirectory.

### feat: Post-installation tasks

You can now add your own custom post-installation/post-deployment tasks to any canister type. The new `post-install` key for canister objects in `dfx.json` can be a command or list of commands, similar to the `build` key of `custom` canisters, and receives all the same environment variables. For example, to replicate the upload task performed with `assets` canisters, you might set `"post-install": "icx-asset sync $CANISTER_ID dist"`.

### feat: assets are no longer copied from source directories before being uploaded to asset canister

Assets are now uploaded directly from their source directories, rather than first being copied
to an output directory.

If you're using `dfx deploy`, you won't see any change in functionality.  If you're running
`dfx canister install --mode=upgrade`, changed files in asset source directories will
be detected and uploaded even without an intervening `dfx build`.

### fix: Added src/declarations to .gitignore for new projects

### fix: remove deprecated candid path environment variable

The environment variable format `CANISTER_CANDID_{name}`, used in Rust projects, was deprecated in 0.9.2, to be unified with the variables `CANISTER_CANDID_PATH_{name}` which are used in other project types. It has now been removed. Note that you will need to update `ic-cdk-macros` if you use the `#[import]` macro.

### feat: deprecate `dfx config` for removal

The `dfx config` command has several issues and is ultimately a poor replacement for [`jq`](https://stedolan.github.io/jq). The command is being deprecated, and will be removed in a later release; we recommend switching to `jq` or similar tools (e.g. `ConvertTo-Json` in PowerShell, `to json` in nushell, etc.)

### feat: Better build scripts for type:custom

Build scripts now always receive a CWD of the DFX project root, instead of wherever `dfx` was invoked from, and a bare script `script.sh` can be specified without needing to prefix with `./`.

### feat: rust, custom, and asset canisters now include candid:service metadata

Motoko canisters already included this metadata.

Also added this metadata to the asset canister wasm, which will cause the next deploy to
install this new version.

### feat: Add safeguard to freezing threshold

Some developers mistakenly think that the freezing threshold is measured in cycles, but it is actually measured in seconds. To stop them from accidentally freezing their canisters, setting a freezing threshold above 50M seconds (~1.5 years) now requires a confirmation.

### fix: restores assets to webpack devserver

### chore: updates webpack dependencies for dfx new project

Resolves an issue where `webpack-cli` was was breaking when users tried to run `npm start` in a fresh project. For affected users of 0.10.1, you can resolve this issue manually by running `npm install webpack@latest webpack-cli@latest terser-webpack-plugin@latest`.

### feat: Support for new ledger notify function

Ledger 7424ea8 deprecates the existing `notify` function with a switch parameter between creating and topping up a canister, and introduces two
functions for doing the same. This should *mostly* be invisible to users, except that previously, if `dfx ledger create-canister` or `dfx ledger top-up`
failed, you would call `dfx ledger notify` after correcting the issue. In order to support the change, this command has been changed to two subcommands:
`dfx ledger notify create-canister` and `dfx ledger notify top-up`.

### feat: `--from-subaccount`

Previously, the ledger commands assumed all transfers were made from the default subaccount for the identity principal. This feature adds a `--from-subaccount` flag to `dfx ledger transfer`, `dfx ledger create-canister`, and `dfx ledger top-up`, to enable making transfers from a selected subaccount. A `--subaccount` flag is also added to `dfx ledger balance` for convenience. Subaccounts are expected as 64-character hex-strings (i.e. 32 bytes).

### feat: cargo audit when building rust canisters

When a canister with type `rust` is built and `cargo-audit` is installed, dfx will now check for vulnerabilities in the dependencies. If a vulnerability is found, dfx will recommend that the user update to a version without known vulnerabilities.

### fix: Freezing Threshold now documented

Calls made to retrieve the help output for `canister update-settings` was missing the `freezing-threshold` parameter.

### chore: warnings and errors are more visible

`WARN` and `ERROR` messages are now clearly labelled as such, and the labels are colored accordingly.
This is now included when running `dfx canister update-settings -h`.

### fix: canister call uses candid file if canister type cannot be determined

The candid file specified in the field `canisters.<canister name>.candid` of dfx.json, or if that not exists `canisters.<canister name>.remote.candid`, is now used when running `dfx canister call`, even when dfx fails to determine the canister type.

### fix: btc/canister http adapter socket not found by replica after restart

After running `dfx start --enable-bitcoin` twice in a row (stopping dfx in between), the second
launched replica would fail to connect to the btc adapter.  This is because ic-starter
does not write a new configuration file if one already exists, so the configuration file
used by the replica referred to one socket path, while dfx passed a different socket path
to the btc adapter.

Now dfx reuses the previously-used unix domain socket path, for both the btc adapter
and for the canister http adapter.

### fix: dfx stop now waits until dfx and any child processes exit

Previously, `dfx stop` would send the TERM signal to the running dfx and its child processes,
and then exit immediately.

This avoids interference between a dfx process performing cleanup at shutdown and
a dfx process that is starting.

### fix: dfx ping no longer creates a default identity

dfx ping now uses the anonymous identity, and no longer requires dfx.json to be present.


### fix: Initialize replica with bitcoin regtest flag

When the bitcoin feature is enabled, dfx was launching the replica with the "bitcoin_testnet" feature.
The correct feature to use is "bitcoin_regtest".

### dfx bootstrap now looks up the port of the local replica

`dfx replica` writes the port of the running replica to one of these locations:

- .dfx/replica-configuration/replica-1.port
- .dfx/ic-ref.port

`dfx bootstrap` will now use this port value, so it's no longer necessary to edit dfx.json after running `dfx replica`.

### feat: dfx.json local network settings can be set on the local network, rather than defaults

In `dfx.json`, the `bootstrap`, `bitcoin`, `canister_http`, and `replica` settings can
now be specified on the local network, rather than in the `defaults` field.
If one of these four fields is set for the local network, the corresponding field
in `defaults` will be ignored.

Example:
``` json
{
  "networks": {
    "local": {
      "bind": "127.0.0.1:8000",
      "canister_http": {
        "enabled": true
      }
    }
  }
}
```

## Dependencies

### Rust Agent

Updated agent-rs to 0.18.0

### Motoko

Updated Motoko from 0.6.28 to 0.6.29.

### Replica

Updated replica to elected commit 8993849de5fab76e796d67750facee55a0bf6649.
This incorporates the following executed proposals:

* [69804](https://dashboard.internetcomputer.org/proposal/69804)
* [67990](https://dashboard.internetcomputer.org/proposal/67990)
* [67483](https://dashboard.internetcomputer.org/proposal/67483)
* [66895](https://dashboard.internetcomputer.org/proposal/66895)
* [66888](https://dashboard.internetcomputer.org/proposal/66888)
* [65530](https://dashboard.internetcomputer.org/proposal/65530)
* [65327](https://dashboard.internetcomputer.org/proposal/65327)
* [65043](https://dashboard.internetcomputer.org/proposal/65043)
* [64355](https://dashboard.internetcomputer.org/proposal/64355)
* [63228](https://dashboard.internetcomputer.org/proposal/63228)
* [62143](https://dashboard.internetcomputer.org/proposal/62143)

### ic-ref

Updated ic-ref to 0.0.1-173cbe84
 - add ic0.performance_counter system interface
 - add system API for ECDSA signing
 - allow optional "error_code" field in responses
 - support gzip-compressed canister modules
 - enable canisters to send HTTP requests

# 0.10.1

## DFX

### fix: Webpack config no longer uses CopyPlugin

Dfx already points to the asset canister's assets directory, and copying to disk could sometimes
lead to an annoying "too many open files" error.

### fix: HSMs are once again supported on Linux

On Linux, dfx 0.10.0 failed any operation with an HSM with the following error:
```
Error: IO: Dynamic loading not supported
```
The fix was to once again dynamically-link the Linux build.

### feat: error explanation and fixing instructions engine

Dfx is now capable of providing explanations and remediation suggestions for entire categories of errors at a time.
Explanations and suggestions will slowly be added over time.
To see an example of an already existing suggestion, run `dfx deploy --network ic` while using an identity that has no wallet configured.

### chore: add context to errors

Most errors that happen within dfx are now reported in much more detail. No more plain `File not found` without explanation what even was attempted.

### fix: identities with configured wallets are not broken anymore and removed only when using the --drop-wallets flag

When an identity has a configured wallet, dfx no longer breaks the identity without actually removing it.
Instead, if the --drop-wallets flag is specified, it properly removes everything and logs what wallets were linked,
and when the flag is not specified, it does not remove anything.

The behavior for identities without any configured wallets is unchanged.

### feat: bitcoin integration: dfx now generates the bitcoin adapter config file

dfx command-line parameters for bitcoin integration:
``` bash
dfx start   --enable-bitcoin  # use default node 127.0.0.1:18444
dfx start   --enable-bitcoin --bitcoin-node <node>
```

The above examples also work for dfx replica.

These default to values from dfx.json:
```
.defaults.bitcoin.nodes
.defaults.bitcoin.enabled
```

The --bitcoin-node parameter, if specified on the command line, implies --enable-bitcoin.

If --enable-bitcoin or .defaults.bitcoin.enabled is set, then dfx start/replica will launch the ic-btc-adapter process and configure the replica to communicate with it.


### feat: print wallet balance in a human readable form #2184

Default behaviour changed for `dfx wallet balance`, it will now print cycles amount upscaled to trillions.

New flag `--precise` added to `dfx wallet balance`. Allows to get exact amount of cycles in wallet (without upscaling).

### feat: canister http integration

dfx command-line parameters for canister http requests integration:
```
dfx start --enable-canister-http
dfx replica --enable-canister-http
```

This defaults to the following value in dfx.json:
```
.defaults.canister_http.enabled
```

### fix: specifying ic provider with a trailing slash is recognised correctly

Specifying the network provider as `https://ic0.app/` instead of `https://ic0.app` is now recognised as the real IC network.

### Binary cache

Added ic-canister-http-adapter to the binary cache.

## Dependencies

### Updated agent-rs to 0.17.0

## Motoko

Updated Motoko from 0.6.26 to 0.6.28.

## Replica

Updated replica to elected commit b90edb9897718730f65e92eb4ff6057b1b25f766.
This incorporates the following executed proposals:

* [61004](https://dashboard.internetcomputer.org/proposal/61004)
* [60222](https://dashboard.internetcomputer.org/proposal/60222)
* [59187](https://dashboard.internetcomputer.org/proposal/59187)
* [58479](https://dashboard.internetcomputer.org/proposal/58479)
* [58376](https://dashboard.internetcomputer.org/proposal/58376)
* [57843](https://dashboard.internetcomputer.org/proposal/57843)
* [57395](https://dashboard.internetcomputer.org/proposal/57395)

## icx-proxy

Updated icx-proxy to commit c312760a62b20931431ba45e5b0168ee79ea5cda

* Added gzip and deflate body decoding before certification validation.
* Fixed unzip and streaming bugs
* Added Prometheus metrics endpoint
* Added root and invalid ssl and dns mapping

# 0.10.0

## DFX

### feat: Use null as default value for opt arguments


Before this, `deploy`ing a canister with an `opt Foo` init argument without specifying an `--argument` would lead to an error:

``` bash
$ dfx deploy
Error: Invalid data: Expected arguments but found none.
```

With this change, this isn't an error anymore, but instead `null` is passed as a value. In general, if the user does _not_ provide an `--argument`, and if the init method expects only `opt` arguments, then `dfx` will supply `null` for each argument.

Note in particular that this does not try to match `opt` arguments for heterogeneous (`opt`/non-`opt`) signatures. Note moreover that this only impacts a case that would previously error out, so no existing (working) workflows should be affected.

### feat: dfx identity set-wallet now checks that the provided canister is actually a wallet

This check was previously performed on local networks, but not on mainnet.

### feat: dfx canister call --candid <path to candid file> ...

Allows one to provide the .did file for calls to an arbitrary canister.

### feat: Install arbitrary wasm into canisters

You no longer need a DFX project setup with a build task to install an already-built wasm module into a canister ID. The new `--wasm <path>` flag to `dfx canister install` will bypass project configuration and install the wasm module at `<path>`. A DFX project setup is still recommended for general use; this should mostly be used for installing pre-built canisters. Note that DFX will also not perform its usual checks for API/ABI/stable-memory compatibility in this mode.

### feat: Support for 128-bit cycle counts

Cycle counts can now exceed the previously set maximum of 2^64. The new limit is 2^128. A new wallet version has been bundled with this release that supports the new cycle count. You will not be able to use this feature with your existing wallets without running `dfx wallet upgrade`, but old wallets will still work just fine with old cycle counts.

### fix: dfx start will once again notice if dfx is already running

dfx will once again display 'dfx is already running' if dfx is already running,
rather than 'Address already in use'.

As a consequence, after `dfx start` failed to notice that dfx was already running,
it would replace .dfx/pid with an empty file.  Later invocations of `dfx stop`
would display no output and return a successful exit code, but leave dfx running.

### fix: dfx canister update-settings <canister id> works even if the canister id is not known to the project.

This makes the behavior match the usage text of the command:
`<CANISTER> Specifies the canister name or id to update. You must specify either canister name/id or the --all option`

### feat: dfx deploy --upgrade-unchanged or dfx canister install --mode upgrade --upgrade-unchanged

When upgrading a canister, `dfx deploy` and `dfx canister install` skip installing the .wasm
if the wasm hash did not change.  This avoids a round trip through stable memory for all
assets on every dfx deploy, for example.  By passing this argument, dfx will instead
install the wasm even if its hash matches the already-installed wasm.

### feat: Introduce DFX_CACHE_ROOT environment variable

A new environment variable, `DFX_CACHE_ROOT`, has been introduced to allow setting the cache root directory to a different location than the configuration root directory. Previously `DFX_CONFIG_ROOT` was repurposed for this which only allowed one location to be set for both the cache and configuration root directories.

This is a breaking change since setting `DFX_CONFIG_ROOT` will no longer set the cache root directory to that location.

### fix: Error if nonzero cycles are passed without a wallet proxy

Previously, `dfx canister call --with-cycles 1` would silently ignore the `--with-cycles` argument as the DFX principal has no way to pass cycles and the call must be forwarded through the wallet. Now it will error instead of silently ignoring it. To forward a call through the wallet, use `--wallet $(dfx identity get-wallet)`, or `--wallet $(dfx identity --network ic get-wallet)` for mainnet.

### feat: Configure subnet type of local replica

The local replica sets its parameters according to the subnet type defined in defaults.replica.subnet_type, defaulting to 'application' when none is specified.
This makes it less likely to accidentally hit the 'cycles limit exceeded' error in production.  Since the previous default was `system`, you may see these types errors in development instead.
Possible values for defaults.replica.subnet_type are: "application", "verifiedapplication", "system"

Example how to specify the subnet type:
``` json
{
  "defaults": {
    "replica": {
      "subnet_type": "verifiedapplication"
    }
  }
}
```

### feat: Introduce command for local cycles top-up

`dfx ledger fabricate-cycles <canister (id)> <optional amount>` can be used during local development to create cycles out of thin air and add them to a canister. Instead of supplying a canister name or id it is also possible to use `--all` to add the cycles to every canister in the current project. When no amount is supplied, the command uses 10T cycles as default. Using this command with `--network ic` will result in an error.

### feat: Private keys can be stored in encrypted format

`dfx identity new` and `dfx identity import` now ask you for a password to encrypt the private key (PEM file) when it is stored on disk.
If you decide to use a password, your key will never be written to disk in plain text.
In case you don't want to enter your password all the time and want to take the risk of storing your private key in plain text, you can use the `--disable-encryption` flag.

The `default` identity as well as already existing identities will NOT be encrypted. If you want to encrypt an existing identity, use the following commands:
``` bash
dfx identity export identity_name > identity.pem
# if you have set old_identity_name as the identity that is used by default, switch to a different one
dfx identity use other_identity
dfx identity remove identity_name
dfx identity import identity_name identity.pem
```

### feat: Identity export

If you want to get your identity out of dfx, you can use `dfx identity export identityname > exported_identity.pem`. But be careful with storing this file as it is not protected with your password.

### feat: Identity new/import now has a --force flag

If you want to script identity creation and don't care about overwriting existing identities, you now can use the `--force` flag for the commands `dfx identity new` and `dfx identity import`.

### fix: Do not automatically create a wallet on IC

When running `dfx deploy --network ic`, `dfx canister --network ic create`, or `dfx identity --network ic get-wallet` dfx no longer automatically creates a cycles wallet for the user if none is configured. Instead, it will simply report that no wallet was found for that user.

Dfx still creates the wallet automatically when running on a local network, so the typical workflow of `dfx start --clean` and `dfx deploy` will still work without having to manually create the wallet.

### fix: Identities cannot exist and not at the same time

When something went wrong during identity creation, the identity was not listed as existing.
But when trying to create an identity with that name, it was considered to be already existing.

### feat: dfx start and dfx replica can now launch the ic-btc-adapter process

Added command-line parameters:
``` bash
dfx start   --enable-bitcoin --btc-adapter-config <path>
dfx replica --enable-bitcoin --btc-adapter-config <path>
```

These default to values from dfx.json:
```
.defaults.bitcoin.btc_adapter_config
.defaults.bitcoin.enabled
```

The --btc-adapter-config parameter, if specified on the command line, implies --enable-bitcoin.

If --enable-bitcoin or .defaults.bitcoin.enabled is set, and a btc adapter configuration is specified,
then dfx start/replica will launch the ic-btc-adapter process.

This integration is not yet complete, pending upcoming functionality in ic-starter.

### fix: report context of errors

dfx now displays the context of an error in several places where previously the only error
message would be something like "No such file or directory."

### chore: updates starter project for Node 18

Webpack dev server now works for Node 18 (and should work for Node 17). A few packages are also upgraded

## updating dependencies

Updated to version 0.14.0 of agent-rs

## Cycles wallet

- Module hash: bb001d1ebff044ba43c060956859f614963d05c77bd778468fce4de095fe8f92
- https://github.com/dfinity/cycles-wallet/commit/f18e9f5c2f96e9807b6f149c975e25638cc3356b

## Replica

Updated replica to elected commit b3788091fbdb8bed7e527d2df4cc5e50312f476c.
This incorporates the following executed proposals:

* [57150](https://dashboard.internetcomputer.org/proposal/57150)
* [54964](https://dashboard.internetcomputer.org/proposal/54964)
* [53702](https://dashboard.internetcomputer.org/proposal/53702)
* [53231](https://dashboard.internetcomputer.org/proposal/53231)
* [53134](https://dashboard.internetcomputer.org/proposal/53134)
* [52627](https://dashboard.internetcomputer.org/proposal/52627)
* [52144](https://dashboard.internetcomputer.org/proposal/52144)
* [50282](https://dashboard.internetcomputer.org/proposal/50282)

Added the ic-btc-adapter binary to the cache.

## Motoko

Updated Motoko from 0.6.25 to 0.6.26.

# 0.9.3

## DFX

### feat: dfx deploy now displays URLs for the frontend and candid interface

### dfx.json

In preparation for BTC integration, added configuration for the bitcoind port:

``` json
{
  "canisters": {},
  "defaults": {
    "bitcoind": {
      "port": 18333
    }
  }
}
```

## icx-proxy

Updated icx-proxy to commit 594b6c81cde6da4e08faee8aa8e5a2e6ae815602, now static-linked.

* upgrade HTTP calls upon canister request
* no longer proxies /_/raw to the dfx internal webserver
* allows for generic StreamingCallback tokens

## Replica

Updated replica to blessed commit d004accc3904e24dddb13a11d93451523e1a8a5f.
This incorporates the following executed proposals:

* [49653](https://dashboard.internetcomputer.org/proposal/49653)
* [49011](https://dashboard.internetcomputer.org/proposal/49011)
* [48427](https://dashboard.internetcomputer.org/proposal/48427)
* [47611](https://dashboard.internetcomputer.org/proposal/47611)
* [47512](https://dashboard.internetcomputer.org/proposal/47512)
* [47472](https://dashboard.internetcomputer.org/proposal/47472)
* [45984](https://dashboard.internetcomputer.org/proposal/45984)
* [45982](https://dashboard.internetcomputer.org/proposal/45982)

## Motoko

Updated Motoko from 0.6.21 to 0.6.25.

# 0.9.2

## DFX

### feat: Verify Candid and Motoko stable variable type safety of canister upgrades

Newly deployed Motoko canisters now embed the Candid interface and Motoko stable signatures in the Wasm module.
`dfx deploy` and `dfx canister install` will automatically check

	1) the backward compatible of Candid interface in both upgrade and reinstall mode;
	2) the type safety of Motoko stable variable type in upgrade mode to avoid accidentally lossing data;

See [Upgrade compatibility](https://smartcontracts.org/docs/language-guide/compatibility.html) for more details.

### feat: Unified environment variables across build commands

The three canister types that use a custom build tool - `assets`, `rust`, and `custom` - now all support the same set of environment variables during the build task:

* `DFX_VERSION` - The version of DFX that was used to build the canister.
* `DFX_NETWORK` - The network name being built for. Usually `ic` or `local`.
* `CANISTER_ID_{canister}` - The canister principal ID of the canister `{canister}` registered in `dfx.json`.
* `CANISTER_CANDID_PATH_{canister}` - The path to the Candid interface file for the canister `{canister}` among your canister's dependencies.
* `CANISTER_CANDID_{canister}` (deprecated) - the same as `CANISTER_CANDID_PATH_{canister}`.  This is provided for backwards compatibility with `rust` and `custom` canisters, and will be removed in dfx 0.10.0.
* `CANISTER_ID` - Same as `CANISTER_ID_{self}`, where `{self}` is the name of _this_ canister.
* `CANISTER_CANDID_PATH` - Same as `CANISTER_CANDID_PATH_{self}`, where `{self}` is the name of _this_ canister.

### feat: Support for local ledger calls

If you have an installation of the ICP Ledger (see [Ledger Installation Guide](https://github.com/dfinity/ic/tree/master/rs/rosetta-api/ledger_canister#deploying-locally)), `dfx ledger balance` and `dfx ledger transfer` now support
`--ledger-canister-id` parameter.

Some examples:
``` bash
$ dfx ledger \
  --network local \
  balance \
  --ledger-canister-id  rrkah-fqaaa-aaaaa-aaaaq-cai
1000.00000000 ICP

$ dfx ledger \
  --network local \
  transfer --amount 0.1 --memo 0 \
  --ledger-canister-id  rrkah-fqaaa-aaaaa-aaaaq-cai 8af54f1fa09faeca18d294e0787346264f9f1d6189ed20ff14f029a160b787e8
Transfer sent at block height: 1
```

### feat: `dfx ledger account-id` can now compute canister addresses

The `dfx ledger account-id` can now compute addresses of principals and canisters.
The command also supports ledger subaccounts now.

``` bash
dfx ledger account-id --of-principal 53zcu-tiaaa-aaaaa-qaaba-cai
dfx ledger --network small02 account-id --of-canister ledger_demo
dfx ledger account-id --of-principal 53zcu-tiaaa-aaaaa-qaaba-cai --subaccount 0000000000000000000000000000000000000000000000000000000000000001
```

### feat: Print the full error chain in case of a failure

All `dfx` commands will now print the full stack of errors that led to the problem, not just the most recent error.
Example:

```
Error: Subaccount '00000000000000000000000000000000000000000000000000000000000000000' is not a valid hex string
Caused by:
  Odd number of digits
```

### fix: dfx import will now import pem files created by `quill generate`

`quill generate` currently outputs .pem files without an `EC PARAMETERS` section.
`dfx identity import` will now correctly identify these as EC keys, rather than Ed25519.

### fix: retry on failure for ledger create-canister, top-up, transfer

dfx now calls `transfer` rather than `send_dfx`, and sets the created_at_time field in order to retry the following commands:

* dfx ledger create-canister
* dfx ledger top-up
* dfx ledger transfer

### feat: Remote canister support

It's now possible to specify that a canister in dfx.json references a "remote" canister on a specific network,
that is, a canister that already exists on that network and is managed by some other project.

Motoko, Rust, and custom canisters may be configured in this way.

This is the general format of the configuration in dfx.json:
``` json
{
  "canisters": {
    "<canister name>": {
      "remote": {
        "candid": "<path to candid file to use when building on remote networks>",
        "id": {
          "<network name>": "<principal on network>"
        }
      }
    }
  }
}
```

The "id" field, if set for a given network, specifies the canister ID for the canister on that network.
The canister will not be created or installed on these remote networks.
For other networks, the canister will be created and installed as usual.

The "candid" field, if set within the remote object, specifies the candid file to build against when
building other canisters on a network for which the canister is remote.  This definition can differ
from the candid definitions for local builds.

For example, if have an installation of the ICP Ledger (see [Ledger Installation Guide](https://github.com/dfinity/ic/tree/master/rs/rosetta-api/ledger_canister#deploying-locally))
in your dfx.json, you could configure the canister ID of the Ledger canister on the ic network as below.  In this case,
the private interfaces would be available for local builds, but only the public interfaces would be available
when building for `--network ic`.
``` json
{
  "canisters": {
    "ledger": {
      "type": "custom",
      "wasm": "ledger.wasm",
      "candid": "ledger.private.did",
      "remote": {
        "candid": "ledger.public.did",
        "id": {
          "ic": "ryjl3-tyaaa-aaaaa-aaaba-cai"
        }
      }
    },
    "app": {
      "type": "motoko",
      "main": "src/app/main.mo",
      "dependencies": [ "ledger" ]
    }
  }
}
```

As a second example, suppose that you wanted to write a mock of the ledger in Motoko.
In this case, since the candid definition is provided for remote networks,
`dfx build` (with implicit `--network local`) will build app against the candid
definitions defined by mock.mo, but `dfx build --network ic` will build app against
`ledger.public.did`.

This way, you can define public update/query functions to aid in local testing, but
when building/deploying to mainnet, references to methods not found in `ledger.public.did`
will be reports as compilation errors.

``` json
{
  "canisters": {
    "ledger": {
      "type": "motoko",
      "main": "src/ledger/mock.mo",
      "remote": {
        "candid": "ledger.public.did",
        "id": {
          "ic": "ryjl3-tyaaa-aaaaa-aaaba-cai"
        }
      }
    },
    "app": {
      "type": "motoko",
      "main": "src/app/main.mo",
      "dependencies": [ "ledger" ]
    }
  }
}
```

### feat: Generating remote canister bindings

It's now possible to generate the interface of a remote canister using a .did file using the `dfx remote generate-binding <canister name>|--all` command. This makes it easier to write mocks for local development.

Currently, dfx can generate .mo, .rs, .ts, and .js bindings.

This is how you specify how to generate the bindings in dfx.json:
``` json
{
  "canisters": {
    "<canister name>": {
      "main": "<path to mo/rs/ts/js file that will be generated>",
      "remote": {
        "candid": "<path to candid file to use when generating bindings>"
        "id": {}
      }
    }
  }
}
```

## ic-ref

Upgraded from a432156f24faa16d387c9d36815f7ddc5d50e09f to ab8e3f5a04f0f061b8157c2889f8f5de05f952bb

* Support 128-bit system api for cycles
* Include canister_ranges in the state tree
* Removed limit on cycles in a canister

## Replica

Updated replica to blessed commit 04fe8b0a1262f07c0cec1fdfa838a37607370a61.
This incorporates the following executed proposals:

* [45091](https://dashboard.internetcomputer.org/proposal/45091)
* [43635](https://dashboard.internetcomputer.org/proposal/43635)
* [43633](https://dashboard.internetcomputer.org/proposal/43633)
* [42783](https://dashboard.internetcomputer.org/proposal/42783)
* [42410](https://dashboard.internetcomputer.org/proposal/42410)
* [40908](https://dashboard.internetcomputer.org/proposal/40908)
* [40647](https://dashboard.internetcomputer.org/proposal/40647)
* [40328](https://dashboard.internetcomputer.org/proposal/40328)
* [39791](https://dashboard.internetcomputer.org/proposal/39791)
* [38541](https://dashboard.internetcomputer.org/proposal/38541)

## Motoko

Updated Motoko from 0.6.20 to 0.6.21.

# 0.9.0

## DFX

### feat!: Remove the wallet proxy and the --no-wallet flag

Breaking change: Canister commands, except for `dfx canister create`, will make the call directly, rather than via the user's wallet. The `--no-wallet` flag is thus removed from `dfx canister` as its behavior is the default.

When working with existing canisters, use the `--wallet` flag in conjunction with `dfx identity get-wallet` in order to restore the old behavior.

You will need to upgrade your wallet and each of your existing canisters to work with the new system.  To do so, execute the following in each of your dfx projects:
``` bash
dfx wallet upgrade
dfx canister --wallet "$(dfx identity get-wallet)" update-settings --all --add-controller "$(dfx identity get-principal)"
```
To upgrade projects that you have deployed to the IC mainnet, execute the following:
``` bash
dfx wallet --network ic upgrade
dfx canister --network ic --wallet "$(dfx identity --network ic get-wallet)" update-settings --all --add-controller "$(dfx identity get-principal)"
```

### feat: Add --add-controller and --remove-controller flags for "canister update-settings"

`dfx canister update-settings` previously only let you overwrite the entire controller list; `--add-controller` and `--remove-controller` instead add or remove from the list.

### feat: Add --no-withdrawal flag for "canister delete" for when the canister is out of cycles

`dfx canister delete --no-withdrawal <canister>` can be used to delete a canister without attempting to withdraw cycles.

### fix: set RUST_MIN_STACK to 8MB for ic-starter (and therefore replica)

This matches the value used in production and is meant to exceed the configured 5 MB wasmtime stack.

### fix: asset uploads will retry failed requests as expected

Fixed a defect in asset synchronization where no retries would be attempted after the first 30 seconds overall.

## Motoko

Updated Motoko from 0.6.11 to 0.6.20.

* Implement type union/intersection
* Transform for-loops on arrays into while-loops
* Tighten typing rules for type annotations in patterns
* Candid decoding: skip vec any fast
* Bump up MAX_HP_FOR_GC from 1GB to 3GB
* Candid decoder: Trap if a principal value is too large
* Eliminate bignum calls from for-iteration on arrays
* Improve scheduling
* Improve performance of bignum equality
* Stable signatures: frontend, metadata, command-line args
* Added heartbeat support

## Cycles wallet

- Module hash: 53ec1b030f1891bf8fd3877773b15e66ca040da539412cc763ff4ebcaf4507c5
- https://github.com/dfinity/cycles-wallet/commit/57e53fcb679d1ea33cc713d2c0c24fc5848a9759

## Replica

Updated replica to blessed commit 75138bbf11e201aac47266f07bee289dc18a082b.
This incorporates the following executed proposals:

* [33828](https://dashboard.internetcomputer.org/proposal/33828)
* [31275](https://dashboard.internetcomputer.org/proposal/31275)
* [31165](https://dashboard.internetcomputer.org/proposal/31165)
* [30392](https://dashboard.internetcomputer.org/proposal/30392)
* [30078](https://dashboard.internetcomputer.org/proposal/30078)
* [29235](https://dashboard.internetcomputer.org/proposal/29235)
* [28784](https://dashboard.internetcomputer.org/proposal/28784)
* [27975](https://dashboard.internetcomputer.org/proposal/27975)
* [26833](https://dashboard.internetcomputer.org/proposal/26833)
* [25343](https://dashboard.internetcomputer.org/proposal/25343)
* [23633](https://dashboard.internetcomputer.org/proposal/23633)

# 0.8.4

## DFX

### feat: "rust" canister type

You can now declare "rust" canisters in dfx.json.
``` json
{
  "canisters": {
    "canister_name": {
      "type": "rust",
      "package": "crate_name",
      "candid": "path/to/canister_name.did"
    }
  }
}
```

Don't forget to place a `Cargo.toml` in your project root.
Then dfx will build the rust canister with your rust toolchain.
Please also make sure that you have added the WebAssembly compilation target.

``` bash
rustup target add wasm32-unknown-unknown
```

You can also create new dfx project with a default rust canister.

``` bash
dfx new --type=rust <project-name>
```

### chore: updating dfx new template

Updates dependencies to latest for Webpack, and updates config. Additionally simplifies environment variables for canister ID's in config.

Additionally adds some polish to the starter template, including a favicon and using more semantic html in the example app

### feat: environment variable overrides for executable pathnames

You can now override the location of any executable normally called from the cache by specifying
an environment variable. For example, DFX_ICX_PROXY_PATH will specify the path for `icx-proxy`.

### feat: dfx deploy --mode=reinstall <canister>

`dfx deploy` can now reinstall a single canister, controlled by a new `--mode=reinstall` parameter.
This is destructive (it resets the state of the canister), so it requires a confirmation
and can only be performed on a single canister at a time.

`dfx canister install --mode=reinstall <canister>` also requires the same confirmation,
and no longer works with `--all`.

## Replica

The included replica now supports canister_heartbeat.  This only works with rust canisters for the time being,
and does not work with the emulator (`dfx start --emulator`).

# 0.8.3

## DFX

### fix: ic-ref linux binary no longer references /nix/store

This means `dfx start --emulator` has a chance of working if nix is not installed.
This has always been broken, even before dfx 0.7.0.

### fix: replica and ic-starter linux binaries no longer reference /nix/store

This means `dfx start` will work again on linux.  This bug was introduced in dfx 0.8.2.

### feat: replaced --no_artificial_delay option with a sensible default.

The `--no-artificial-delay` option not being the default has been causing a lot of confusion.
Now that we have measured in production and already applied a default of 600ms to most subnets deployed out there,
we have set the same default for dfx and removed the option.

## Motoko

Updated Motoko from 0.6.10 to 0.6.11.

* Assertion error messages are now reproducible (#2821)

# 0.8.2

## DFX

### feat: dfx canister delete can now return cycles to a wallet or dank

By default `dfx canister delete` will return cycles to the default cycles wallet.
Cycles can be returned to a designated canister with `--withdraw-cycles-to-canister` and
cycles can be returned to dank at the current identity principal with `--withdraw-cycles-to-dank`
and to a designated principal with `--withdraw-cycles-to-dank-principal`.

### feat: dfx canister create now accepts multiple instances of --controller argument

It is now possible to create canisters with more than one controller by
passing multiple instances of the `--controller parameter to `dfx canister create`.

You will need to upgrade your wallet with `dfx wallet upgrade`, or `dfx wallet --network ic upgrade`

### feat: dfx canister update-settings now accepts multiple instance of --controller argument

It is now possible to configure a canister to have more than one controller by
passing multiple instances of the `--controller parameter to `dfx canister update-settings`.

### feat: dfx canister info and dfx canister status now display all controllers

### feat!: dfx canister create --controller <controller> named parameter

Breaking change: The controller parameter for `dfx canister create` is now passed as a named parameter,
rather than optionally following the canister name.

Old: dfx canister create [canister name] [controller]
New: dfx canister create --controller <controller> [canister name]

### fix: dfx now respects $DFX_CONFIG_ROOT when looking for legacy credentials

Previously this would always look in `$HOME/.dfinity/identity/creds.pem`.

### fix: changed dfx canister (create|update-settings) --memory-allocation limit to 12 GiB

Updated the maximum value for the --memory-allocation value to be 12 GiB (12,884,901,888 bytes)

## Cycles Wallet

- Module hash: 9183a38dd2eb1a4295f360990f87e67aa006f225910ab14880748e091248e086
- https://github.com/dfinity/cycles-wallet/commit/9ef38bb7cd0fe17cda749bf8e9bbec5723da0e95

### Added support for multiple controllers

You will need to upgrade your wallet with `dfx wallet upgrade`, or `dfx wallet --network ic upgrade`

## Replica

The included replica now supports public spec 0.18.0

* Canisters can now have more than one controller
* Adds support for 64-bit stable memory
* The replica now goes through an initialization sequence, reported in its status
as `replica_health_status`.  Until this reports as `healthy`, queries or updates will
fail.
** `dfx start --background` waits to exit until `replica_health_status` is `healthy`.
** If you run `dfx start` without `--background`, you can call `dfx ping --wait-healthy`
to wait until the replica is healthy.

## Motoko

Updated Motoko from 0.6.7 to 0.6.10

* add Debug.trap : Text -> None (motoko-base #288)
* Introduce primitives for `Int` ⇔ `Float` conversions (#2733)
* Fix crashing bug for formatting huge floats (#2737)

# 0.8.1

## DFX

### feat: dfx generate types command

``` bash
dfx generate
```

This new command will generate type declarations for canisters in dfx.json.

You can control what will be generated and how with corresponding configuration in dfx.json.

Under dfx.json → "canisters" → "<canister_name>", developers can add a "declarations" config. Options are:

* "output" → directory to place declarations for that canister | default is "src/declarations/<canister_name>"

* "bindings" → [] list of options, ("js", "ts", "did", "mo") | default is "js", "ts", "did"

* "env_override" → a string that will replace process.env.{canister_name_uppercase}_CANISTER_ID in the "src/dfx/assets/language_bindings/canister.js" template.

js declarations output

* index.js (generated from "src/dfx/assets/language_bindings/canister.js" template)

* <canister_name>.did.js - candid js binding output

ts declarations output

  * <canister_name>.did.d.ts - candid ts binding output

did declarations output

  * <canister_name>.did - candid did binding output

mo declarations output

  * <canister_name>.mo - candid mo binding output

### feat: dfx now supports the anonymous identity

Use it with either of these forms:
``` bash
dfx identity use anonymous
dfx --identity anonymous ...
```

### feat: import default identities

Default identities are the pem files generated by `dfx identity new ...` which contain Ed25519 private keys.
They are located at `~/.config/dfx/identity/xxx/identity.pem`.
Now, you can copy such pem file to another computer and import it there.

``` bash
dfx identity new alice
cp ~/.config/dfx/identity/xxx/identity.pem alice.pem
# copy the pem file to another computer, then
dfx identity import alice alice.pem
```

Before, people can manually copy the pem files to the target directory to "import". Such workaround still works.
We suggest to use the `import` subcommand since it also validate the private key.

### feat: Can now provide a nonstandard wallet module with DFX_WALLET_WASM environment variable

Define DFX_WALLET_WASM in the environment to use a different wasm module when creating or upgrading the wallet.

## Asset Canister

### fix: trust full asset SHA-256 hashes provided by the caller

When the caller provides SHA-256 hashes (which dfx does), the asset canister will no longer
recompute these hashes when committing the changes.  These recomputations were causing
canisters to run out of cycles, or to attempt to exceed the maximum cycle limit per update.

# 0.8.0

The 0.8.0 release includes updates and fixes that are primarily internal to improve existing features and functions rather than user-visible.

## DFX

### fix: dfx identity set-wallet no longer requires --force when used with --network ic

This was intended to skip verification of the wallet canister on the IC network,
but ended up only writing to the wallets.json file if --force was passed.

### chore: updating dependencies

* Support for the latest version of the {IC} specification and replica.

* Updating to latest versions of Motoko, Candid, and agent-rs

### feat: Type Inference Update

* Changes to `dfx new` project template and JavaScript codegen to support type inference in IDE's

* Adding webpack dev server to project template

* Migration path documented at https://sdk.dfinity.org/docs/release-notes/0.8.0-rn.html

# 0.7.7

Breaking changes to frontend code generation, documented in 0.8.0

## DFX

### feat: deploy and canister install will now only upgrade a canister if the wasm actually changed

dfx deploy and dfx canister install now compare the hash of the already-installed module
with the hash of the built canister's wasm output.  If they are the same, they leave the canister
in place rather than upgrade it.  They will still synchronize assets to an asset canister regardless
of the result of this comparison.


# 0.7.6

## icx-proxy

The streaming callback mechanism now requires the following record structure for the token:
```
type StreamingCallbackToken = record {
    key: text;
    content_encoding: text;
    index: nat;
    sha256: opt blob;
};
```

Previously, the token could be a record with any set of fields.

# 0.7.2

## DFX

### fix: set default cycle balance to 3T

Change the default cycle balance of a canister from 10T cycles to 3T cycles.

## Cycles Wallet

- Module hash: 1404b28b1c66491689b59e184a9de3c2be0dbdd75d952f29113b516742b7f898
- https://github.com/dfinity/cycles-wallet/commit/e902708853ab621e52cb68342866d36e437a694b

### fix: It is no longer possible to remove the last controller.

Fixed an issue where the controller can remove itself from the list of controllers even if it's the only one,
leaving the wallet uncontrolled.
Added defensive checks to the wallet's remove_controller and deauthorize methods.

# 0.7.1

## DFX

### feat: sign request_status for update call

When using `dfx canister sign` to generate a update message, a corresponding
request_status message is also signed and append to the json as `signed_request_status`.
Then after sending the update message, the user can check the request_status using
`dfx canister send message.json --status`.

### fix: wallet will not proxy dfx canister call by default

Previously, `dfx canister call` would proxy queries and update calls via the wallet canister by default.
(There was the `--no-wallet` flag to bypass the proxy and perform the calls as the selected identity.)
However, this behavior had drawbacks, namely each `dfx canister call` was an inter-canister call
by default and calls would take a while to resolve. This fix makes it so that `dfx canister call` no longer
proxies via the wallet by default. To proxy calls via the wallet, you can do
`dfx canister --wallet=<wallet-id> call`.

### feat: add --no-artificial-delay to dfx replica and start

This change adds the `--no-artificial-delay` flag to `dfx start` and `dfx replica`.
The replica shipped with dfx has always had an artificial consensus delay (introduced to simulate
a delay users might see in a networked environment.) With this new flag, that delay can
be lessened. However, you might see increased CPU utilization by the replica process.

### feat: add deposit cycles and uninstall code

This change introduces the `deposit_cycles` and `uninstall_code` management canister
methods as dedicated `dfx canister` subcommands.

### fix: allow consistent use of canisters ids in canister command

This change updates the dfx commands so that they will accept either a canister name
(sourced from your local project) or a valid canister id.

# 0.7.0

## DFX

### feat: add output type to request-status

This change allows you to specify the format the return result for `dfx canister request-status`.

### fix: deleting a canister on a network removes entries for other networks

This change fixes a bug where deleting a canister on a network removed all other entries for
the canister in the canister_ids.json file.

### feat: point built-in `ic` network provider at mainnet

`--network ic` now points to the mainnet IC (as Sodium has been deprecated.)

### feat: add candid UI canister

The dedicated candid UI canister is installed on a local network when doing a `dfx canister install`
or `dfx deploy`.

### fix: Address already in use (os error 48) when issuing dfx start

This fixes an error which occurred when starting a replica right after stopping it.

### feat: ledger subcommands

dfx now supports a dedicated `dfx ledger` subcommand. This allows you to interact with the ledger
canister installed on the Internet Computer. Example commands include `dfx ledger account-id` which
prints the Account Identifier associated with your selected identity, `dfx ledger transfer` which
allows you to transfer ICP from your ledger account to another, and `dfx ledger create-canister` which
allows you to create a canister from ICP.

### feat: update to 0.17.0 of the Interface Spec

This is a breaking change to support 0.17.0 of the Interface Spec. Compute & memory allocation values
are set when creating a canister. An optional controller can also be specified when creating a canister.
Furthermore, `dfx canister set-controller` is removed, in favor of `dfx canister update-settings` which
allows the controller to update the controller, the compute allocation, and the memory allocation of the
canister. The freezing threshold value isn't exposed via dfx cli yet, but it may still be modified by
calling the management canister via `dfx canister call aaaaa-aa update-settings`

### feat: add wallet subcommands

dfx now supports a dedicated `dfx wallet` subcommand. This allows you to interact with the cycles wallet
associated with your selected identity. For example, `dfx wallet balance` to get the cycle balance,
`dfx wallet list-addresses` to display the associated controllers & custodians, and `dfx wallet send <destination> <amount>`
to send cycles to another wallet.

## Cycles Wallet

- Module Hash: a609400f2576d1d6df72ce868b359fd08e1d68e58454ef17db2361d2f1c242a1
- https://github.com/dfinity/cycles-wallet/commit/06bb256ca0738640be51cf84caaced7ea02ca29d

### feat: Use Internet Identity Service.

# 0.7.0-beta.5

## Cycles Wallet

- Module Hash: 3d5b221387875574a9fd75b3165403cf1b301650a602310e9e4229d2f6766dcc
- https://github.com/dfinity/cycles-wallet/commit/c3cbfc501564da89e669a2d9de810d32240baf5f

### feat: Updated to Public Interface 0.17.0

### feat: The wallet_create_canister method now takes a single record argument, which includes canister settings.

### fix: Return correct content type and encoding for non-gz files.

### fix: Updated frontend for changes to canister creation interface.

# 0.7.0-beta.3

## DFX

### fix: assets with an unrecognized file extension will use content-type "application/octet-stream"

# 0.7.0-beta.2

## DFX

### feat: synchronize assets rather than uploading even assets that did not change

DFX will now also delete assets from the container that do not exist in the project.
This means if you stored assets in the container, and they are not in the project,
dfx deploy or dfx install will delete them.

## Asset Canister

### Breaking change: change to store() method signature

- now takes arguments as a single record parameter
- must now specify content type and content encoding, and may specify the sha256

# 0.7.0-beta.1

## DFX

### fix: now deletes from the asset canister assets that no longer exist in the project

### feat: get certified canister info from read state #1514

Added `dfx canister info` command to get certified canister information. Currently this information is limited to the controller of the canister and the SHA256 hash of its WASM module. If there is no WASM module installed, the hash will be None.

## Asset Canister

### Breaking change: change to list() method signature

- now takes a parameter, which is an empty record
- now returns an array of records

### Breaking change: removed the keys() method

- use list() instead

# 0.7.0-beta.0

## DFX

### feat: webserver can now serve large assets

# 0.6.26

## DFX

### feat: add --no-wallet flag and --wallet option to allow Users to bypass Wallet or specify a Wallet to use for calls (#1476)

Added `--no-wallet` flag to `dfx canister` and `dfx deploy`. This allows users to call canister management functionality with their Identity as the Sender (bypassing their Wallet canister.)
Added `--wallet` option to `dfx canister` and `dfx deploy`. This allows users to specify a wallet canister id to use as the Sender for calls.
`--wallet` and `--no-wallet` conflict with each other. Omitting both will invoke the selected Identity's wallet canister to perform calls.

### feat: add canister subcommands `sign` and `send`

Users can use `dfx canister sign ...` to generated a signed canister call in a json file. Then `dfx canister send [message.json]` to the network.

Users can sign the message on an air-gapped computer which is secure to host private keys.

#### Note

* `sign` and `send` currently don't proxy through wallet canister. Users should use the subcommands with `dfx canister --no-wallet sign ...`.

* The `sign` option `--expire-after` will set the `ingress_expiry` to a future timestamp which is current plus the duration.
Then users can send the message during a 5 minutes time window ending in that `ingress_expiry` timestamp. Sending the message earlier or later than the time window will both result in a replica error.

### feat: implement the HTTP Request proposal in dfx' bootstrap webserver. +
And add support for http requests in the base storage canister (with a default to `/index.html`).

This does not support other encodings than `identity` for now (and doesn't even return any headers). This support will be added to the upgraded asset storage canister built in #1482.

Added a test that uses `curl localhost` to test that the asset storage AND the webserver properly support the http requests.

This commit also upgrades tokio and reqwest in order to work correctly. There are also _some_ performance issues noted (this is slower than the `icx-http-server` for some reason), but those are not considered criticals and could be improved later on.

Renamed the `project_name` in our own generated assets to `canister_name`, for things that are generated during canister build (and not project generation).

### feat: add support for ECDSA on secp256k1

You can now a generate private key via OpenSSL or a simlar tool, import it into dfx, and use it to sign an ingress message.

``` bash
openssl ecparam -name secp256k1 -genkey -out identity.pem
dfx identity import <name> identity.pem
dfx identity use <name>
dfx canister call ...
```

## Asset Canister

### feat: The asset canister can now store assets that exceed the message ingress limit (2 MB)

* Please note that neither the JS agent nor the HTTP server have been updated yet to server such large assets.
* The existing interface is left in place for backwards-compatibility, but deprecated:
** retrieve(): use get() and get_chunk() instead
** store(): use create_batch(), create_chunk(), and commit_batch() instead
** list(): use keys() instead

# 0.6.25

## DFX

- feat: dfx now provides CANISTER_ID_<canister_name> environment variables for all canisters to "npm build" when building the frontend.

## Agents

### Rust Agent

- feat: AgentError due to request::Error will now include the reqwest error message
in addition to "Could not reach the server"
- feat: Add secp256k1 support (dfx support to follow)

# 0.6.24

## DFX

- feat: add option to specify initial cycles for newly created canisters (#1433)

Added option to `dfx canister create` and `dfx deploy` commands: `--with-cycles <with-cycles>`.
This allows the user to specify the initial cycle balance of a canister created by their wallet.
This option is a no-op for the Sodium network.

``` bash
dfx canister create --with-cycles 8000000000 some_canister
dfx deploy --with-cycles 8000000000
```

Help string:
```
Specifies the initial cycle balance to deposit into the newly
created canister. The specified amount needs to take the
canister create fee into account. This amount is deducted
from the wallet's cycle balance
```

- feat: install `dfx` by version or tag (#1426)

This feature adds a new dfx command `toolchain` which have intuitive subcommands.
The toolchain specifiers can be a complete version number, major minor version, or a tag name.

``` bash
dfx toolchain install 0.6.24 # complete version
dfx toolchain install 0.6    # major minor
dfx toolchain install latest # tag name
dfx toolchain default latest
dfx toolchain list
dfx toolchain uninstall latest
```

- fix: onboarding related fixups (#1420)

Now that the Mercury Alpha application subnetwork is up and we are getting ready to onboard devs, the dfx error message for wallet creation has changed:
For example,
``` bash
dfx canister --network=alpha create hello
Creating canister "hello"...
Creating the canister using the wallet canister...
Creating a wallet canister on the alpha network.
Unable to create a wallet canister on alpha:
The Replica returned an error: code 3, message: "Sender not authorized to use method."
Wallet canisters on alpha may only be created by an administrator.
Please submit your Principal ("dfx identity get-principal") in the intake form to have one created for you.
```

- feat: add deploy wallet subcommand to identity (#1414)

This feature adds the deploy-wallet subcommand to the dfx identity.
The User provides the ID of the canister onto which the wallet WASM is deployed.

``` bash
dfx identity deploy-wallet --help
dfx-identity-deploy-wallet
Installs the wallet WASM to the provided canister id

USAGE:
    dfx identity deploy-wallet <canister-id>

ARGS:
    <canister-id>    The ID of the canister where the wallet WASM will be deployed

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
```

# 0.6.22

## DFX

- feat: dfx call random value when argument is not provided (#1376)

- fix: canister call can take canister ids for local canisters even if … (#1368)
- fix: address panic in dfx replica command (#1338)
- fix: dfx new webpack.config.js does not encourage running 'js' through ts-… (#1341)

## Sample apps

- There have been updates, improvements, and new sample apps added to the [examples](https://github.com/dfinity/examples/tree/master/motoko) repository.

    All of Motoko sample apps in the [examples](https://github.com/dfinity/examples/tree/master/motoko) repository have been updated to work with the latest release of the SDK.

    There are new sample apps to illustrate using arrays ([Quicksort](https://github.com/dfinity/examples/tree/master/motoko/quicksort)) and building create/read/update/delete (CRUD) operations for a web application [Superheroes](https://github.com/dfinity/examples/tree/master/motoko/superheroes).

- The [LinkedUp](https://github.com/dfinity/linkedup) sample application has been updated to work with the latest release of Motoko and the SDK.

## Motoko

## Agents

## Canister Development Kit (CDK)
