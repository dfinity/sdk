# dfx changelog

# UNRELEASED

### feat: More PocketIC flags supported

`dfx start --pocketic` is now compatible with `--artificial-delay` and the `subnet_type`  configuration option, and enables `--enable-canister-http` by default.

### feat: Support canister log allowed viewer list

Added support for the canister log allowed viewer list, enabling specified users to access a canister's logs without needing to be set as the canister's controller.
Valid settings are:
- `--add-log-viewer`, `--remove-log-viewer` and `--set-log-viewer` flags with `dfx canister update-settings` 
- `--log-viewer` flag with `dfx canister create`
- `canisters[].initialization_values.log_visibility.allowed_viewers` in `dfx.json`

## Dependencies

### Motoko

Updated Motoko to [0.13.0](https://github.com/dfinity/motoko/releases/tag/0.13.0)

### Replica

Updated replica to elected commit c43a4880199c00135c8415957851e823b3fb769e.
This incorporates the following executed proposals:

- [133144](https://dashboard.internetcomputer.org/proposal/133144)
- [133143](https://dashboard.internetcomputer.org/proposal/133143)
- [133142](https://dashboard.internetcomputer.org/proposal/133142)
- [133063](https://dashboard.internetcomputer.org/proposal/133063)
- [133062](https://dashboard.internetcomputer.org/proposal/133062)
- [133061](https://dashboard.internetcomputer.org/proposal/133061)
- [132548](https://dashboard.internetcomputer.org/proposal/132548)
- [132547](https://dashboard.internetcomputer.org/proposal/132547)
- [132507](https://dashboard.internetcomputer.org/proposal/132507)
- [132482](https://dashboard.internetcomputer.org/proposal/132482)
- [132481](https://dashboard.internetcomputer.org/proposal/132481)
- [132500](https://dashboard.internetcomputer.org/proposal/132500)
- [132416](https://dashboard.internetcomputer.org/proposal/132416)
- [132413](https://dashboard.internetcomputer.org/proposal/132413)
- [132414](https://dashboard.internetcomputer.org/proposal/132414)
- [132412](https://dashboard.internetcomputer.org/proposal/132412)
- [132376](https://dashboard.internetcomputer.org/proposal/132376)
- [132375](https://dashboard.internetcomputer.org/proposal/132375)
- [132223](https://dashboard.internetcomputer.org/proposal/132223)
- [132222](https://dashboard.internetcomputer.org/proposal/132222)
- [132149](https://dashboard.internetcomputer.org/proposal/132149)
- [132148](https://dashboard.internetcomputer.org/proposal/132148)
- [131787](https://dashboard.internetcomputer.org/proposal/131787)
- [131757](https://dashboard.internetcomputer.org/proposal/131757)
- [131697](https://dashboard.internetcomputer.org/proposal/131697)

### Candid UI

Module hash 15da2adc4426b8037c9e716b81cb6a8cf1a835ac37589be2cef8cb3f4a04adaa

# 0.24.0

### fix: bumps sveltekit starter dependency versions to prevent typescript config error

### feat: expose canister upgrade options in CLI

`dfx canister install` and `dfx deploy` takes options `--skip-pre-upgrade` and `--wasm-memory-persistence`.

`dfx deploy --mode` now takes the same possible values as `dfx canister install --mode`: "install", "reinstall", "upgrade" and "auto".

In "auto" mode, the upgrade options are hints which only take effects when the actual install mode is "upgrade". 

To maintain backward compatibility, a minor difference between the two commands remains.
If the `--mode` is not set, `dfx deploy` defaults to "auto", while `dfx canister install` defaults to "install".

### feat: Also report Motoko stable compatibility warnings

Report upgrade compatibility warnings for Motoko, such as deleted stable variables, in addition to compatibility errors.

### feat: Support for Motoko's enhanced orthogonal persistence.

Support Motoko's enhanced orthogonal persistence by automatically setting the canister upgrade option `wasm_memory_persistence` based on the Wasm metadata.

### feat: PocketIC state

`dfx start --pocketic` no longer requires `--clean`, and can persist replica state between runs.

### fix: Scripts always run with current directory set to the project root

Build scripts and other scripts now always run with the working directory
set to the project root (directory containing dfx.json).

This applies to the following:
 - build scripts
 - extension run
 - tech stack value computation
 - packtool (vessel, mops etc)

### feat: `dfx extension list` supports listing available extensions

`dfx extension list` now support `--available` flag to list available extensions from the
[extension catalog](https://github.com/dfinity/dfx-extensions/blob/main/catalog.json).
The extension catalog can be overridden with the `--catalog-url` parameter.

## Dependencies

### Frontend canister

Added `create_chunks`. It has the same behavior as `create_chunk`, except that it takes a `vec blob` and returns a `vec BatchId` instead of non-`vec` variants.

Module hash: 3a533f511b3960b4186e76cf9abfbd8222a2c507456a66ec55671204ee70cae3

### Motoko

Updated Motoko to [0.12.1](https://github.com/dfinity/motoko/releases/tag/0.12.1)

# 0.23.0

### fix: relax content security policy for sveltekit starter

We had to roll back part of the increased default security policy for the sveltekit starter due to the framework's use of inline scripts

### feat: Add canister snapshots

The new `dfx canister snapshot` command can be used to create, apply, and delete snapshots of stopped canisters.

### feat: PocketIC HTTP gateway

icx-proxy's HTTP gateway has been replaced with PocketIC's. (This does not impact the meaning of `--pocketic` in `dfx start`.)

### feat: Enable threshold schnorr signatures for Ed25519

Schnorr signature signing for `Ed25519` is now enabled.
A test key id `Ed25519:dfx_test_key` is ready to be used by locally created canisters.

### feat: Added settings_digest field to the network-id file

### feat: install extensions using the catalog

`dfx extension install` now locates extensions using the
[extension catalog](https://github.com/dfinity/dfx-extensions/blob/main/catalog.json).
This can be overridden with the `--catalog-url` parameter.

## Dependencies

### Replica

Updated replica to elected commit 3d0b3f10417fc6708e8b5d844a0bac5e86f3e17d.
This incorporates the following executed proposals:

- [131473](https://dashboard.internetcomputer.org/proposal/131473)


## Dependencies

### Replica

Updated replica to elected commit 2c0b76cfc7e596d5c4304cff5222a2619294c8c1.
This incorporates the following executed proposals:

- [131390](https://dashboard.internetcomputer.org/proposal/131390)
- [131055](https://dashboard.internetcomputer.org/proposal/131055)
- [131054](https://dashboard.internetcomputer.org/proposal/131054)
- [131032](https://dashboard.internetcomputer.org/proposal/131032)
- [131028](https://dashboard.internetcomputer.org/proposal/131028)

### feat: generate .env files for Motoko canisters

### feat: support `"security_policy"` and `"disable_security_policy_warning"` in `.ic-assets.json5`

*This change has an accompanying migration guide. Please see the 0.23.0 migration guide for instructions on how to adapt your project to this feature.*

It is now possible to specify a `"security_policy"` field in `.ic-assets.json5` for asset configurations.
Valid options are `"disabled"`, `"standard"`, and `"hardened"`.
The security policy provides a set of standard headers to make frontends more secure.
Headers manually specified in the `"headers"` field take precedence over the security policy headers.

If `"security_policy"` is not specified or `"disabled"` is set, then no headers are added. If `"security_policy"` is not set at all, a warning is displayed that there is no security policy set.

If `"standard"` is specified, a set of security headers is added to the asset. The headers can be displayed with `dfx info security-policy`.
It is a set of security headers that will work for most dapps. A warning is displayed that the headers could be hardened.

If `"hardened"` is set, the same headers as with `"standard"` are added.
The asset sync expects that improved headers are set that would improve security where appropriate.
If no custom headers are present the asset sync will fail with an error.

All warnings regarding security policies can be disabled with ``"disable_security_policy_warning": true`. It needs to be set per asset.

The standard/hardened security policy headers can be seen with `dfx info security-policy`.
It also contains a lot of suggestions on how to harden the policy.

Updated the starter projects to use `"security_policy"` instead of including the whole security policy by defining individual headers.

### feat: `dfx info security-policy`

Shows the headers that get applied to assets that are configured to `"security_policy": "standard"` or `"security_policy": "hardened"` in `.ic-assets.json5`.
Produces output that can be directly pasted into a `.json5` document.

### feat: `dfx extension install <url to extension.json>`

It's now possible for `dfx extension install` to install an extension from
somewhere other than https://github.com/dfinity/dfx-extensions, by passing
a URL to an extension.json file rather than an extension name.

For example, these are equivalent:
```bash
dfx extension install nns
dfx extension install https://raw.githubusercontent.com/dfinity/dfx-extensions/main/extensions/nns/extension.json
```

This update also adds the optional field `download_url_template` to extension.json,
which dfx will use to locate an extension release archive.

### fix: `dfx extension install` no longer reports an error if the extension is already installed

However, if a version is specified with `--version`, and the installed version is different,
then `dfx extension install` will still report an error.

### fix: `dfx ledger create-canister` sets controller properly

A recent [hotfix](https://forum.dfinity.org/t/nns-update-2024-05-15-cycles-minting-canister-hotfix-proposal-129728/30807) to the CMC changed how the arguments to `notify_create_canister` need to be passed.
`dfx` now again properly calls that function.

### feat: display replica port in `dfx start`

This replaces the dashboard link, which is now shown only in verbose mode. This should hopefully be less confusing for new users.

### feat!: add `crate` field to dfx.json

It is now possible to specify a particular crate within a Rust package to use for a canister module, using the `crate` field.
This enables specifying crates with different names than the package. In a few cases these were previously auto-detected
by dfx, you will need to add this field if you were using such a setup.

### feat: the `--wallet` parameter now accepts an identity name

The `--wallet` parameter can now be either a principal or the name of an identity.

If the name of an identity, dfx looks up the associated wallet's principal.

This means `--wallet <name>` is the equivalent of `--wallet $(dfx identity get-wallet --identity <name>)`.

### fix: display error cause of some http-related errors

Some commands that download http resources, for example `dfx extension install`, will
once again display any error cause.

### chore: remove the deprecated --use-old-metering flag

# 0.22.0

### asset uploads: retry some HTTP errors returned by the replica

Now retries the following, with exponential backoff as is already done for connect and transport errors:
- 500 internal server error
- 502 bad gateway
- 503 service unavailable
- 504 gateway timeout
- 429 many requests

### fix: Allow canisters to be deployed even if unrelated canisters in dfx.json are malformed

### feat!: enable cycles ledger support unconditionally

### chore!: removed `unsafe-eval` CSP from default starter template

To do this, the `@dfinity/agent` version was updated as well.

### fix: `dfx build` no longer requires a password for password-protected identities

### chore!: enforce `--wallet` requirement for `dfx canister call --with-cycles` earlier

### feat: add `dfx schema` support for .json files related to extensions

- `dfx schema --for extension-manifest` corresponds to extension.json
- `dfx schema --for extension-dependencies` corresponds to dependencies.json

### chore!: enforce minimum password length of 9 characters

The [NIST guidelines](https://pages.nist.gov/800-63-3/sp800-63b.html) require passwords to be longer than 8 characters.
This is now enforced when creating new identities.
Identities protected by a shorter password can still be decrypted.

### feat: `dfx extension install` now uses the extension's dependencies.json file to pick the highest compatible version

### feat: Enable threshold schnorr signatures for Bip340Secp256k1

Schnorr signature signing for `Bip340Secp256k1` is now enabled.
A test key id `Bip340Secp256k1:dfx_test_key` is ready to be used by locally created canisters.

## Dependencies

### Replica

Updated replica to elected commit 5849c6daf2037349bd36dcb6e26ce61c2c6570d0.
This incorporates the following executed proposals:

- [130985](https://dashboard.internetcomputer.org/proposal/130985)
- [130984](https://dashboard.internetcomputer.org/proposal/130984)
- [130819](https://dashboard.internetcomputer.org/proposal/130819)
- [130818](https://dashboard.internetcomputer.org/proposal/130818)
- [130748](https://dashboard.internetcomputer.org/proposal/130748)
- [130749](https://dashboard.internetcomputer.org/proposal/130749)
- [130728](https://dashboard.internetcomputer.org/proposal/130728)
- [130727](https://dashboard.internetcomputer.org/proposal/130727)
- [130409](https://dashboard.internetcomputer.org/proposal/130409)
- [130408](https://dashboard.internetcomputer.org/proposal/130408)

### Motoko

Updated Motoko to [0.11.2](https://github.com/dfinity/motoko/releases/tag/0.11.2)

# 0.21.0

### feat: dfx killall

Introduced `dfx killall`, a command for killing DFX-started processes.

### feat!: remove support for bitcoin query API

`dfx call --query aaaaa-aa bitcoin_get_balance_query/bitcoin_get_utxos_query` will result in an error.

### fix: simplified log message when using the default shared network configuration

Now displays `Using the default configuration for the local shared network.`
instead of `Using the default definition for the 'local' shared network because ~/.config/dfx/networks.json does not define it.`

### chore!: Improved error message about canister ranges when directly connecting to a node on a non-root subnet

### feat: `dfx start` for the shared local network stores replica state files in unique directories by options

The state files for different replica versions are often incompatible,
so `dfx start` requires the `--clean` argument in order to reset data when
using different replica versions or different replica options.

For the local shared network, dfx now stores replica state files in different
directories, split up by replica version and options.

As an example, you'll be able to do things like this going forward:
```bash
dfx +0.21.0 start
(cd project1 && dfx deploy && dfx canister call ...)
dfx stop

dfx +0.22.0 start
# notice --clean is not required.
# even if --clean were passed, the canisters for project1 would be unaffected.
(cd project2 && dfx deploy)
# project1 won't be affected unless you call dfx in its directory
dfx stop

dfx +0.21.0 start
# the canisters are still deployed
(cd project1 && dfx canister call ...)
```

Prior to this change, the second `dfx start` would have had to include `--clean`,
which would have reset the state of the shared local network, affecting all projects.

This also means `dfx start` for the shared local network won't ever require you to pass `--clean`.

`dfx start` will delete old replica state directories.  At present, it retains the 10 most recently used.

This doesn't apply to project-specific networks, and it doesn't apply with `--pocketic`.

It doesn't apply to project-specific networks because the project's canister ids would
reset anyway on first access. If you run `dfx start` in a project directory where dfx.json
defines the local network, you'll still be prompted to run with `--clean` if using a
different replica version or different replica options.

It doesn't apply to `--pocketic` because PocketIC does not yet persist any data.

### feat: allow specifying encodings in `.ic-assets.json`

When uploading assets to an asset canister, `dfx` by default uploads `.txt`, `.html` and `.js` files in `identity` encoding but also in `gzip` encoding to the frontend canister if encoding saves bytes.
It is now possible to specify in `.ic-assets.json` which encodings are used besides `identity`.
Note that encodings are only used if the encoding saves bytes compared to `identity` or if `identity` is not a specified encoding.

Example: To turn off `gzip` for `.js` files and to turn on `gzip` for `.jpg` files, use this in `.ic-assets.json`:
``` json
{
  "match": "**/*.js",
  "encodings": ["identity"]
},
{
  "match": "**/*.jpg",
  "encodings": ["identity", "gzip"]
}
```

### feat: `dfx canister url`

Add `dfx canister url` subcommand to display the url of a given canister. Basic usage as below:

``` bash
dfx canister url <canister>
```

The `<canister>` argument specifies the name or id of the canister for which you want to display the url.

### feat: `log_visibility` canister setting

Adds support for the `log_visibility` canister setting, which configures which users are allowed to read a canister's logs.
Valid options are `controllers` and `public`. The setting can be used with the `--log-visibility` flag in `dfx canister create`
and `dfx canister update-settings`, or in `dfx.json` under `canisters[].initialization_values.log_visibility`.

## Asset canister synchronization

### feat: support `brotli` encoding

Asset synchronization now not only supports `identity` and `gzip`, but also `brotli` encoding.
The default encodings are still
- `identity` and `gzip` for MIME types `.txt`, `.html` and `.js`
- `identity` for anything else

## Dependencies

### Frontend canister

**fix!: URL decoding follows the whatwg standard**

Previously, the frontend canister used custom logic to decode URLs.
The logic was replaced with a dependency that follows https://url.spec.whatwg.org/#percent-decode, which is what JavaScript's `new Request("https://example.com/% $").url` also uses.
This also drops support for decoding `%%` to `%`. `%` does no longer need to be encoded.

URLs that contain invalid encodings now return `400 Bad Request` instead of `500 Internal Server Error`

- Module hash: 2cc4ec4381dee231379270a08403c984986c9fc0c2eaadb64488b704a3104cc0
- https://github.com/dfinity/sdk/pull/3767

### Replica

Updated replica to elected commit 246d0ce0784d9990c06904809722ce5c2c816269.
This incorporates the following executed proposals:

- [130392](https://dashboard.internetcomputer.org/proposal/130392)
- [130400](https://dashboard.internetcomputer.org/proposal/130400)
- [130315](https://dashboard.internetcomputer.org/proposal/130315)
- [130134](https://dashboard.internetcomputer.org/proposal/130134)

# 0.20.2

### fix: `dfx canister delete` fails

`dfx canister delete` occasionally fails because it attempts to withdraw too many cycles from the canister before it is deleted.
Usually, `dfx` tries again with a larger margin of cycles, but sometimes this gets stuck.
It is now possible to use `--initial-margin` to manually supply a margin in case the automatic margin does not work.

### perf: improve sync command performance

Improves `sync` (eg. `dfx deploy`, `icx-asset sync`) performance by parallelization:
- Make asset properties query faster by parallelization, significant improvement for canisters that have many assets
- Make chunk creation process faster, by increasing parallelization 4=>25, significant improvement when deploying lots of small assets

`icx-asset`: add support for log levels, defaulting to `info`

### PocketIC support

Passing `--pocketic` to `dfx start` now starts a PocketIC server instead of the replica. PocketIC is lighter-weight than the replica and execution environment internals can be manipulated by REST commands. For more information, see the [PocketIC readme](https://github.com/dfinity/pocketic).

### feat: subaccount can be derived from principal in `dfx ledger account-id`

### feat: `dfx info candid-ui-url`

`dfx info candid-ui-url` displays the URL to the Candid UI canister for an explicitly specified `--network <network name>` (or `local` by default).

### chore: Improve help text of `dfx identity new` to include which characters are valid in identity names

### fix: Capitalization of "Wasm" in docs and messages

The output of `dfx canister status` has been also changed to use consistent capitalization of words.

### fix!(frontend-canister): include `.well-known` directory by default for asset upload

When uploading assets to an asset canister, `dfx` by default excludes directories and files with names that start with `.`.
`dfx` will start including folders with the name `.well-known` by default.
It is possible to override this in `.ic-assets.json` like this:

``` json
{
  "match": ".well-known",
  "ignore": true
}
```

### fix: Transferring funds too early in `dfx ledger create-canister` with --next-to

When creating a canister with `dfx ledger create-canister --next-to` on a canister that does not exist (e.g., 2vxsx-fae), then the funds are first transferred away from the users account, but the call then fails to create the new canister, and the funds are not returned to the user's account.

## Dependencies

### Updated to [agent-rs 0.35.0](https://github.com/dfinity/agent-rs/blob/main/CHANGELOG.md#0350---2024-05-10)

### Replica

Updated replica to elected commit ec35ebd252d4ffb151d2cfceba3a86c4fb87c6d6.
This incorporates the following executed proposals:

- [130083](https://dashboard.internetcomputer.org/proposal/130083)
- [129747](https://dashboard.internetcomputer.org/proposal/129747)
- [129746](https://dashboard.internetcomputer.org/proposal/129746)
- [129706](https://dashboard.internetcomputer.org/proposal/129706)
- [129697](https://dashboard.internetcomputer.org/proposal/129697)
- [129696](https://dashboard.internetcomputer.org/proposal/129696)
- [129628](https://dashboard.internetcomputer.org/proposal/129628)
- [129627](https://dashboard.internetcomputer.org/proposal/129627)

# 0.20.1

### feat: reformatted error output

Rather than increasing indentation, dfx now aligns the error causes with a "Caused by: " prefix.

Also changed error types to report error causes as causes, rather than embedding their error cause in the error text.

Before:
```bash
Error: Failed while trying to deploy canisters.
Caused by: Failed while trying to deploy canisters.
  Failed to build all canisters.
    Failed while trying to build all canisters.
      The build step failed for canister 'bw4dl-smaaa-aaaaa-qaacq-cai' (wasminst_backend) with an embedded error: Failed to build Motoko canister 'wasminst_backend'.: Failed to compile Motoko.: Failed to run 'moc'.: The command '"/Users/ericswanson/.cache/dfinity/versions/0.19.0/moc" ... params ...  failed with exit status 'exit status: 1'.
Stdout:

Stderr:
/Users/ericswanson/w/wasminst/src/wasminst_backend/main2.mo: No such file or directory
```

After:
```bash
Error: Failed while trying to deploy canisters.
Caused by: Failed to build all canisters.
Caused by: Failed while trying to build all canisters.
Caused by: The build step failed for canister 'bw4dl-smaaa-aaaaa-qaacq-cai' (wasminst_backend)
Caused by: Failed to build Motoko canister 'wasminst_backend'.
Caused by: Failed to compile Motoko.
Caused by: Failed to run 'moc'.
Caused by: The command '"/Users/ericswanson/.cache/dfinity/versions/0.20.0/moc" ... params ... failed with exit status 'exit status: 1'.
Stdout:

Stderr:
/Users/ericswanson/w/wasminst/src/wasminst_backend/main2.mo: No such file or directory
```

### fix: "Failed to decrypt PEM file" errors messages will now include the cause

### feat: Wasm memory soft-limit

Adds support for the `wasm_memory_limit` canister setting, which limits the canister's heap during most calls but does not affect queries. As with other canister settings, it can be set in `dfx canister create` or `dfx canister update-settings` via the `--wasm-memory-limit` flag, as well as in `dfx.json` under `canisters[].initialization_values.wasm_memory_limit`.

### feat: extensions can define a canister type

Please see [extension-defined-canister-types](docs/concepts/extension-defined-canister-types.md) for details.

### feat: init_arg_file in dfx.json

Introduces support for the `init_arg_file` field in `dfx.json`, providing an alternative method to specify initialization arguments.

This field accepts a relative path, from the directory containing the `dfx.json` file.

**Note**

- Only one of `init_arg` and `init_arg_file` can be defined at a time.
- If `--argument` or `--argument-file` are set, the argument from the command line takes precedence over the one in dfx.json.

### fix: dfx new failure when node is available but npm is not

`dfx new` could fail with "Failed to scaffold frontend code" if node was installed but npm was not installed.

## Dependencies

### Cycles wallet

Updated cycles wallet to a gzipped version of `20240410` release:
- Module hash: `7745d3114e3e5fbafe8a7150a0a8c15a5b8dc9257f294d5ced67d41be76065bc`, in gzipped form: `664df1045e093084f4ebafedd3a793cc3b3be0a7ef1b245d8d3defe20b33057c`
- https://github.com/dfinity/cycles-wallet/commit/b013764dd827560d8538ee2b7be9ecf66bed6be7

### Replica

Updated replica to elected commit 5e285dcaf77db014ac85d6f96ff392fe461945f5.
This incorporates the following executed proposals:

- [129494](https://dashboard.internetcomputer.org/proposal/129494)
- [129493](https://dashboard.internetcomputer.org/proposal/129493)
- [129428](https://dashboard.internetcomputer.org/proposal/129428)
- [129427](https://dashboard.internetcomputer.org/proposal/129427)
- [129423](https://dashboard.internetcomputer.org/proposal/129423)
- [129408](https://dashboard.internetcomputer.org/proposal/129408)
- [129379](https://dashboard.internetcomputer.org/proposal/129379)
- [129378](https://dashboard.internetcomputer.org/proposal/129378)

# 0.20.0

### fix: set `CANISTER_CANDID_PATH_<canister name>` properly for remote canisters

In the remote canister declaration it is possible to set a candid file to use when the canister is remote on a specific network.
`dfx` now correctly sets the `CANISTER_CANDID_PATH_<canister name>` environment variable during the build process on remote networks if the file exists.

### feat: display schema for dfx metadata json

`dfx schema --for dfx-metadata` to display JSON schema of the "dfx" metadata.

### feat: add tech_stack to the Canister Metadata Standard

The standardized `dfx` metadata is extended with another object: `tech_stack`.

Please check [tech-stack](docs/concepts/tech-stack.md) for more details.

### chore: updated management canister .did file

### feat: added `dfx completion` command

This command generates shell completion scripts for `bash`, `elvish`, `fish`, `zsh`, or PowerShell.

Describing how to install shell completion scripts is beyond the scope of this document.
Here are two commands that would enable command completion in the current shell:

In zsh:

```bash
source <(dfx completion zsh)
```

In bash:

```bash
source <(dfx completion)
```

### fix: dfx no longer always creates .dfx directory if dfx.json is present

Previously, `dfx` would always create a `.dfx` directory in the project root if `dfx.json` was present.
Now, it only does so if the command accesses the .dfx directory in some way.

### fix: dfx only loads dfx.json for commands that need it

For example, this will work now:
```bash
echo garbage >dfx.json && dfx identity get-principal
```

## Dependencies

### Replica

Updated replica to elected commit 02dcaf3ccdfe46bd959d683d43c5513d37a1420d.
This incorporates the following executed proposals:

- [129084](https://dashboard.internetcomputer.org/proposal/129084)
- [129081](https://dashboard.internetcomputer.org/proposal/129081)
- [129035](https://dashboard.internetcomputer.org/proposal/129035)
- [128876](https://dashboard.internetcomputer.org/proposal/128876)
- [128904](https://dashboard.internetcomputer.org/proposal/128904)
- [128864](https://dashboard.internetcomputer.org/proposal/128864)
- [128816](https://dashboard.internetcomputer.org/proposal/128816)
- [128846](https://dashboard.internetcomputer.org/proposal/128846)

# 0.19.0

### fix: call management canister Bitcoin query API without replica-signed query

`dfx canister call --query` defaults to use "Replica-signed query" feature.

It doesn't work with bitcoin query calls to the management canister because the Boundary Nodes cannot route the `read_state` call.

Only for these particular queries, `dfx` will make the query calls without checking the replica signatures.

If the response reliability is a concern, you can make update calls to the secure alternatives.

### feat(beta): enable cycles ledger support

If the environment variable `DFX_CYCLES_LEDGER_SUPPORT_ENABLE` is set and no cycles wallet is configured, then dfx will try to use the cycles ledger to perform any operation that the cycles wallet usually is used for.

The following commands/options have been unhidden:
- `dfx cycles`
- `--from-subaccount` for `dfx deploy`, `dfx canister create`, `dfx canister deposit-cycles` to determine which cycles ledger subaccount the used cycles should be used from
- `--created-at-time` for `dfx deploy`, `dfx create canister`, `dfx canister deposit-cycles` to control transaction deduplication on the cycles ledger
- `--to-subaccount` for `dfx canister delete` to control into which subaccount cycles are withdrawn before the canister is deleted

The cycles ledger will not be supported by default until the cycles ledger canister is under NNS control.

### feat: dfx canister call ... --output json

This is the same as `dfx canister call ... | idl2json`, for convenience.

See also: https://github.com/dfinity/idl2json

### fix: Output of dfx ping is now valid JSON

Added commas in between fields, and newlines to improve formatting.

### fix: canister status output to be grep compatible

`dfx canister status` now outputs to `stdout`, rather than `stderr`, so that its output is `grep` compatible.

### fix: fetching canister logs to be grep & tail compatible

`dfx canister logs` now outputs to stdout, rather than stderr, so that its output is `grep` and `tail` compatible.

### fix: fetching canister logs

The management canister method `fetch_canister_logs` can be called only as a query, not as an update call. Therefore, `dfx canister logs <canister_id>` now uses a query call for this purpose.

### `dfx wallet set-name` now actually sets the name of the wallet

### feat: hyphenated project names

DFX no longer forbids hyphens in project names. Anywhere they appear as the name of a variable, e.g. environment variables or generated JS variables, they will be replaced with underscores.

### fix: .ic-assets.json configuration entries no longer overwrite the default for `allow_raw_access`

Previously, any configuration element in .ic-assets.json functioned as if a setting of
`"allow_raw_access": true` were present in the json object.

For example, given the following configuration, all files would be configured
with `allow_raw_access` set to `true`, as if the second entry specified
`"allow_raw_access": true` (which is the default), even though it does not.

```json
[
  {
    "match": "**/*",
    "allow_raw_access": false
  },
  {
    "match": "**/*",
    "headers": {
      "X-Anything": "Something"
    }
  }
]
```

Now, given the same configuration, all files would be configured with `allow_raw_access` set to false, as expected.

Note that the default value of `allow_raw_access` is still `true`.

### fix: removed version switching logic

Removed the logic for calling a different version of dfx based on DFX_VERSION or the `dfx` field in
dfx.json.  This is now performed by dfxvm.

### feat: --always-assist flag for `dfx canister call/install/sign and dfx deploy`

When all the arguments are optional, dfx automatically provides a `null` value when no arguments are provided.
`--always-assist` flag enables the candid assist feature for optional arguments, instead of providing a default `null` value.

### fix(deps): the second pull forget to set wasm_hash_download in pulled.json

When the dependency has been in the cache, `dfx deps pull` forgot to set correct `wasm_hash_download` in `pulled.json`.

It caused the following `init/deploy` commands to fail.

## Dependencies

### Replica

Updated replica to elected commit 425a0012aeb40008e2e72d913318bc9dbdf3b4f4.
This incorporates the following executed proposals:

- [128806](https://dashboard.internetcomputer.org/proposal/128806)
- [128805](https://dashboard.internetcomputer.org/proposal/128805)
- [128296](https://dashboard.internetcomputer.org/proposal/128296)
- [128295](https://dashboard.internetcomputer.org/proposal/128295)
- [128171](https://dashboard.internetcomputer.org/proposal/128171)

### Bitcoin canister

Downgraded Bitcoin canister to [release/2023-10-13](https://github.com/dfinity/bitcoin-canister/releases/tag/release%2F2023-10-13)

### Motoko

Updated Motoko to [0.11.1](https://github.com/dfinity/motoko/releases/tag/0.11.1)

# 0.18.0

### fix!: removed the `dfx upgrade` command

The `dfx upgrade` command now prints a message directing the user to install dfxvm.

### fix!: Remove fallback .env formats

In dfx 0.14.0, we standardized on `CANISTER_ID_<CANISTER_NAME_UPPERCASE>` and
`CANISTER_CANDID_PATH_<CANISTER_NAME_UPPERCASE>` for
environment variables for canister IDs and candid paths respectively,
and deprecated the old formats.  This version removes the old formats.

The only variable names now provided are the following,
all uppercase, with any '-' replaced by '_':
- `CANISTER_CANDID_PATH_<CANISTER_NAME>`
- `CANISTER_ID_<CANISTER_NAME>`

For reference, these formats were removed (any '-' characters were replaced by '_'):
- `CANISTER_CANDID_PATH_<canister_name_case_from_dfx_json>`
- `<CANISTER_NAME_UPPERCASE>_CANISTER_ID`

### feat: add `dfx canister logs <canister_id>` for fetching canister's logs (preview)

There is a new subcommand `logs` to fetch canister's logs.
When printing the log entries it tries to guess if the content can be converted to UTF-8 text and prints an array of hex bytes if it fails.

**Note**

This feature is still in development. Changes may occur in following releases.

### feat: display local asset canister URLs in subdomain format

Locally, canisters can either be accessed via `<canister_id>.localhost:<port>` or `localhost:<port>?canisterId=<canister_id>`.
The query parameter format is annoying to handle in SPAs, therefore the subdomain format is now displayed alongside the subdomain version after deployments.

The query parameter format is not removed because Safari does not support localhost subdomains.

### fix: .env files sometimes missing some canister ids

Made it so `dfx deploy` and `dfx canister install` will always write
environment variables for all canisters in the project that have canister ids
to the .env file, even if they aren't being deployed/installed
or a dependency of a canister being deployed/installed.

### feat: unify CLI options to specify arguments

There are a few subcommands that take `--argument`/`--argument-file` options to set canister call/init arguments.

We unify the related logic to provide consistent user experience.

The notable changes are:

- `dfx deploy` now accepts `--argument-file`.
- `dfx deps init` now accepts `--argument-file`.

### feat: candid assist feature

Ask for user input when Candid argument is not provided in `dfx canister call`, `dfx canister install` and `dfx deploy`.
Previously, we cannot call `dfx deploy --all` when multiple canisters require init args, unless the init args are specified in `dfx.json`. With the Candid assist feature, dfx now asks for init args in terminal when a canister requires init args.

### fix: restored access to URLs like http://localhost:8080/api/v2/status through icx-proxy

Pinned icx-proxy at 69e1408347723dbaa7a6cd2faa9b65c42abbe861, shipped with dfx 0.15.2

This means commands like the following will work again:
```
curl -v --http2-prior-knowledge "http://localhost:$(dfx info webserver-port)/api/v2/status" --output -
```

### feat: `dfx cycles approve` and `transfer --from`

It is now possible to approve other principals to spend cycles on your behalf using `dfx cycles approve <spender> <amount>`.
`dfx cycles transfer` now also supports `--from`, `--from-subaccount`, and `--spender-subaccount`.
For detailed explanations on how these fields work please refer to the [ICRC-2 specification](https://github.com/dfinity/ICRC-1/blob/main/standards/ICRC-2/README.md).

### feat: cut over to dfxvm

The script at https://internetcomputer.org/install.sh now installs
the [dfxvm version manager](https://github.com/dfinity/dfxvm) instead of the dfx binary.

### fix(deps): init/deploy still requires hash check

`dfx deps pull` was recently changed to allow hash mismatch wasm. But `init` and `deploy` weren't change accordingly.

Also the warning of hash mismatch is removed since it scares users and users can't fix it locally.

### fix(generate): Rust canister source candid wrongly deleted

Fixed a bug where `dfx generate` would delete a canister's source candid file if the `declarations.bindings` in `dfx.json` did not include "did".

### fix: failed to install when specify id without dfx.json

Fixed a bug where `dfx canister install` would fail when specify a canister id and there is no dfx.json.

### fix: failed to call a canister removed from dfx.json

Fixed a bug where `dfx canister call` would fail when the deployed canister was removed from dfx.json.

### chore: bump candid to 0.10.4

Fix the Typescript binding for init args.

## Dependencies

### Replica

Updated replica to elected commit d966b2737ca75f1bfaa84f21e7f3f7c54b5d7f33.
This incorporates the following executed proposals:

- [128155](https://dashboard.internetcomputer.org/proposal/128155)
- [128154](https://dashboard.internetcomputer.org/proposal/128154)
- [128099](https://dashboard.internetcomputer.org/proposal/128099)
- [128088](https://dashboard.internetcomputer.org/proposal/128088)
- [127707](https://dashboard.internetcomputer.org/proposal/127707)
- [127706](https://dashboard.internetcomputer.org/proposal/127706)

### Motoko

Updated Motoko to [0.11.0](https://github.com/dfinity/motoko/releases/tag/0.11.0)

### Asset canister

Module hash: 32e92f1190d8321e97f8d8f3e793019e4fd2812bfc595345d46d2c23f74c1ab5

bump ic-cdk to 0.13.1

### Candid UI

Module hash: 1208093dcc5b31286a073f00f748ac6612dbae17b66c22332762705960a8aaad

bump ic-cdk to 0.13.1

### Bitcoin canister

Updated Bitcoin canister to [release/2024-01-22](https://github.com/dfinity/bitcoin-canister/releases/tag/release%2F2024-01-22)

# 0.17.0

### feat: new starter templates

`dfx new` now has a new set of customizable project templates and an interactive menu for selecting them. Supports the Svelte, Vue, and React frameworks, and Azle and Kybra backends.

### fix: --no-frontend no longer creates a frontend

Previously `dfx new --no-frontend` still created a frontend canister. This behavior is now accessed via `--frontend simple-assets`.

### feat: `dfx cycles redeem-faucet-coupon`

It is now possible to redeem faucet coupons to cycles ledger accounts.

### feat: `dfx cycles convert`

It is now possible to turn ICP into cycles that are stored on the cycles ledger using `dfx cycles convert --amount <amount of ICP>`

### feat: specified_id in dfx.json

In addition to passing `--specified-id` in `dfx deploy` and `dfx canister create`, `specified_id` can be set in `dfx.json`.

If it is set in both places, the specified ID from the command line takes precedence over the one in dfx.json.

### feat: create canister on same subnet as other canisters

`dfx deploy`, `dfx canister create`, and `dfx ledger create-canister` now support the option `--next-to <canister principal>` to create canisters on the same subnet as other canisters.
The [registry canister](https://dashboard.internetcomputer.org/canister/rwlgt-iiaaa-aaaaa-aaaaa-cai#get_subnet_for_canister) is used as the source of truth to figure out the subnet id.

### feat: init_arg in dfx.json

In addition to passing `--argument` or `--argument-file` in `dfx deploy` and `dfx canister install`, `init_arg` can be set in `dfx.json`.

If it is set in both places, the argument from the command line takes precedence over the one in dfx.json.

### feat(deps): init_arg in pullable metadata

Providers can set an optional `init_arg` field in `pullable` metadata.

When consumers run `dfx deps init` without `--argument`, the value in `init_arg` will be used automatically.

Consumers won't have to figure out the init argument by themselves. It can be overwritten by `dfx deps init --argument`.

### fix(deps): dfx deps init will try to set "(null)" init argument

For pulled canisters which have no `init_arg` in `pullable` metadata, `dfx deps init` without `--argument` will try to set `"(null)"` automatically.

This works for canisters with top-level `opt` in init argument. This behavior is consistent with `dfx deploy` and `dfx canister install`.

The init argument can be overwritten by `dfx deps init --argument`.

### fix(deps): content of wasm_hash_url can have extra fields than the hash

It is natural to point `wasm_hash_url` to the `<FILE>.sha256` file generated by `shasum` or `sha256sum` which consists of the hash and the file name.

Now when `dfx deps pull`, such content will be accept properly.

### feat: dfx upgrade will direct the user to install dfxvm if it has been released.

If the latest release of https://github.com/dfinity/dfxvm is \>\= 1.0, `dfx upgrade` will
direct the user to install dfxvm and then exit.

### feat: fetch did file from canister metadata when making canister calls

`dfx canister call` will always fetch the `.did` file from the canister metadata. If the canister doesn't have the `candid:service` metadata, dfx will fallback to the current behavior of reading the `.did` file from the local build artifact. This fallback behavior is deprecated and we will remove it in a future release. This should not affect Motoko and Rust canisters built from dfx, as `dfx build` automatically writes the Candid metadata into the canister.

If you build with custom canister type, add the following into `dfx.json`:

```
"metadata": [
  {
    "name": "candid:service"
  }
]
```

If you build the canister without using `dfx`, you can use [ic-wasm](https://github.com/dfinity/ic-wasm/releases) to store the metadata:

```
ic-wasm canister.wasm -o canister.wasm metadata candid:service -f service.did -v public
```

### fix: removed the `dfx toolchain` command

Please use the [dfx version manager](https://github.com/dfinity/dfxvm) instead.

### feat: allow dfxvm install script to bypass confirmation

The dfxvm install script now accepts `DFXVM_INIT_YES=<non empty string>` to skip confirmation.

### chore: bump `ic-agent`, `ic-utils` and `ic-identity-hsm` to 0.32.0

# 0.16.1

### feat: query stats support

When using `dfx canister status`, the output now includes the new query statistics. Those might initially be 0, if the feature is not yet enabled on the subnet the canister is installed in.

### fix: Candid parser when parsing `vec \{number\}` with `blob` type

Fix the bug that when parsing `vec \{1;2;3\}` with `blob` type, dfx silently ignores the numbers.

### fix: support `import` for local did file

If the local did file contains `import` or init args, dfx will rewrite the did file when storing in canister metadata.
Due to current limitations of the Candid parser, comments will be dropped during rewriting.
If the local did file doesn't contain `import` or init args, we will not perform the rewriting, thus preserving the comments.

### fix: subtyping check reports the special opt rule as error

### fix: can now run several dfx canister commands outside of a project

The following commands now work outside of a project:
- `dfx canister start <specific canister id>`
- `dfx canister stop <specific canister id>`
- `dfx canister deposit-cycles <amount> <specific canister id>`
- `dfx canister uninstall-code <specific canister id>`

## Dependencies

### Replica

Updated replica to elected commit 044cfd5147fc97d7e5a214966941b6580c325d72.
This incorporates the following executed proposals:

- [127463](https://dashboard.internetcomputer.org/proposal/127463)
- [127461](https://dashboard.internetcomputer.org/proposal/127461)
- [127104](https://dashboard.internetcomputer.org/proposal/127104)

### Candid UI

Module hash: e5f049a97041217554c1849791c093c4103a6844625be3d6453df2e91abeed35

Fix the HTTP header for deploying in remote environments

# 0.16.0

### feat: large canister modules now supported

When using `dfx deploy` or `dfx canister install`, previously Wasm modules larger than 2MiB would be rejected.
They are now automatically submitted via the chunking API if they are large enough.
From a user perspective the limitation will simply have been lifted.

### feat: dfx deps: wasm_hash_url and loose the hash check

Providers can provide the hash through `wasm_hash_url` instead of hard coding the hash directly.

If the hash of downloaded wasm doesn’t match the provided hash (`wasm_hash`, `wasm_hash_url` or read from mainnet state tree), dfx deps won’t abort. Instead, it will print a warning message.

### feat: create canister on specific subnets or subnet types

`dfx deploy`, `dfx canister create`, and `dfx ledger create-canister` now support the option `--subnet <subnet principal>` to create canisters on specific subnets.

`dfx canister create` and `dfx deploy` now support the option `--subnet-type <subnet type>` to create canisters on a random subnet of a certain type.
Use `dfx ledger show-subnet-types` to list the available subnet types

### feat!: update `dfx cycles` commands with mainnet `cycles-ledger` canister ID

The `dfx cycles` command no longer needs nor accepts the `--cycles-ledger-canister-id <canister id>` parameter.

### chore: removed the dfx start --emulator mode

This was deprecated in dfx 0.15.1.

### chore: removed ic-ref from the binary cache

### chore: updated dependencies for new rust projects

Updated to candid 0.10, ic-cdk 0.12, and ic-cdk-timers 0.6

### fix: store playground canister acquisition timestamps with nanosecond precision on all platforms

They've always been stored with nanosecond precisions on Linux and Macos.
Now they are stored with nanosecond precision on Windows too.

### fix: dfx canister delete, when using an HSM identity, no longer fails by trying to open two sessions to the HSM

Previously, this would fail with a PKCS#11: CKR_CRYPTOKI_ALREADY_INITIALIZED error.

## Dependencies

### Motoko

Updated Motoko to [0.10.4](https://github.com/dfinity/motoko/releases/tag/0.10.4)

### Frontend canister

Module hash: 3c86d912ead6de7133b9f787df4ca9feee07bea8835d3ed594b47ee89e6cb730

### Candid UI

Module hash: b91e3dd381aedb002633352f8ebad03b6eee330b7e30c3d15a5657e6f428d815

Fix the routing error when deploying to gitpod/github workspace.
Fix that Candid UI cannot be opened using localhost URL.

### Replica

Updated replica to elected commit 324eb99eb7531369a5ef75560f1a1a652d123714.
This incorporates the following executed proposals:

- [127096](https://dashboard.internetcomputer.org/proposal/127096)
- [127094](https://dashboard.internetcomputer.org/proposal/127094)
- [127034](https://dashboard.internetcomputer.org/proposal/127034)
- [127031](https://dashboard.internetcomputer.org/proposal/127031)
- [126879](https://dashboard.internetcomputer.org/proposal/126879)
- [126878](https://dashboard.internetcomputer.org/proposal/126878)
- [126730](https://dashboard.internetcomputer.org/proposal/126730)
- [126729](https://dashboard.internetcomputer.org/proposal/126729)
- [126727](https://dashboard.internetcomputer.org/proposal/126727)
- [126366](https://dashboard.internetcomputer.org/proposal/126366)
- [126365](https://dashboard.internetcomputer.org/proposal/126365)
- [126293](https://dashboard.internetcomputer.org/proposal/126293)

# 0.15.3

### fix: allow `http://localhost:*` as `connect-src` in the asset canister's CSP

This will enable browsing the asset canister at `http://<canister-id>.localhost:<port>` in most browsers.

### fix: frontend code crashing when there is no canister ID

### feat: `dfx ledger top-up` also accepts canister names

Previously, `dfx ledger top-up` only accepted canister principals. Now it accepts both principals and canister names.

### fix: installer once again detects if curl supports tlsv1.2

A change to `curl --help` output made it so the install script did not detect
that the `--proto` and `--tlsv1.2` options are available.

### chore: skip reserving 8GB of memory when deleting a canister

When dfx deletes a canister, it first withdraws as many cycles as possible from the canister.
While doing so, dfx previously set the memory allocation of the canister to 8GB in order to not run into any memory problems while withdrawing.
This, however, lead to problems with dynamic memory pricing in subnets with a lot of data because then it becomes very expensive to reserve that much data.
dfx now no longer sets a memory allocation. We anticipate fewer problems this way.

### feat: Added support for icx-proxy `--domain` parameter

In order to access a local replica through a domain name or domain names,
it's necessary to pass the `--domain` parameter to icx-proxy.  dfx now supports
this in configuration and as a parameter to dfx start.  You can specify a single
domain or a list of domains in any of the following ways:

- in networks.json, in `.<network>.proxy.domain`
- in dfx.json, in `.networks.<network>.proxy.domain`
- in dfx.json, in `.defaults.proxy.domain`
- to dfx start, as `dfx start --domain <domain1> --domain <domain2> ...`

## Dependencies

### Candid UI

- Module hash: d172df265a14397a460b752ff07598380bc7ebd9c43ece1e82495ae478a88719c
- Internet identity integration in Candid UI. Thanks to @Web3NL!
  + You can customize the II url and derivationOrigin via URL parameter `ii` and `origin` respectively.
- Update with the new profiling API

### Motoko

Updated Motoko to [0.10.3](https://github.com/dfinity/motoko/releases/tag/0.10.3)

# 0.15.2

### fix: `dfx canister delete <canister id>` removes the related entry from the canister id store

Previously, deleting a canister in the project by id rather than by name
would leave the canister id in the canister id store. This would cause
`dfx deploy` to fail.

### fix: dfx extension install can no longer create a corrupt cache directory

Running `dfx cache delete && dfx extension install nns` would previously
create a cache directory containing only an `extensions` subdirectory.
dfx only looks for the existence of a cache version subdirectory to
determine whether it has been installed. The end result was that later
commands would fail when the cache did not contain expected files.

### fix: output_env_file is now considered relative to project root

The .env file location, whether specified as `output_env_file` in dfx.json
or `--output-env-file <file>` on the commandline, is now considered relative
to the project root, rather than relative to the current working directory.

### feat: Read dfx canister install argument from a file

Enables passing large arguments that cannot be passed directly in the command line using the `--argument-file` flag. For example `dfx canister install --argument-file ./my/argument/file.txt my_canister_name`.


### feat: change `list_permitted` and `list_authorized` to an update call.

This requires the `list_authorized` and `list_permitted` methods to be called as an update and disables the ability to
call it as a query call. This resolves a potential security risk.

### fix: `dfx ledger transfer` now logs to stderr messages about duplicates rather than printing them to stdout

The message "transaction is a duplicate of another transaction in block ...", previously printed to stdout, is now logged to stderr. This means that the output of `dfx ledger transfer` to stdout will contain only `Transfer sent at block height <block height>`.

### feat: accept more ways to specify cycle and e8s amounts

Underscores (`_`) can now be used to make large numbers more readable. For example: `dfx canister deposit-cycles 1_234_567 mycanister`

Certain suffixes that replace a number of zeros are now supported. The (case-insensitive) suffixes are:
- `k` for `000`, e.g. `500k`
- `m` for `000_000`, e.g. `5m`
- `b` for `000_000_000`, e.g. `50B`
- `t` for `000_000_000_000`, e.g. `0.3T`

For cycles an additional `c` or `C` is also acceptable. For example: `dfx canister deposit-cycles 3TC mycanister`

### feat: added `dfx cycles` command

This won't work on mainnet yet, but can work locally after installing the cycles ledger.

Added the following subcommands:
 - `dfx cycles balance`
 - `dfx cycles transfer <to> <amount>` (transfer cycles from one account to another account)
 - `dfx cycles top-up <to> <amount>` (send cycles from an account to a canister)

## Dependencies

### Motoko

Updated Motoko to [0.10.2](https://github.com/dfinity/motoko/releases/tag/0.10.2)

### Frontend canister

Defining a custom `etag` header no longer breaks certification.

Fixed a certification issue where under certain conditions the fallback file (`/index.html`) was served with an incomplete certificate tree, not proving sufficiently that the fallback file may be used as a replacement.

Add the option to (re)set all permissions using upgrade arguments. This is especially useful for SNSes that cannot make calls as the canister's controller.

- Module hash: 657938477f1dee46db70b5a9f0bd167ec5ffcd2f930a1d96593c17dcddef61b3
- https://github.com/dfinity/sdk/pull/3443
- https://github.com/dfinity/sdk/pull/3451
- https://github.com/dfinity/sdk/pull/3429
- https://github.com/dfinity/sdk/pull/3428
- https://github.com/dfinity/sdk/pull/3421

### Replica

Updated replica to elected commit 69e1408347723dbaa7a6cd2faa9b65c42abbe861.
This incorporates the following executed proposals:

- [126095](https://dashboard.internetcomputer.org/proposal/126095)
- [126000](https://dashboard.internetcomputer.org/proposal/126000)
- [125592](https://dashboard.internetcomputer.org/proposal/125592)
- [125591](https://dashboard.internetcomputer.org/proposal/125591)
- [125504](https://dashboard.internetcomputer.org/proposal/125504)
- [125503](https://dashboard.internetcomputer.org/proposal/125503)
- [125343](https://dashboard.internetcomputer.org/proposal/125343)
- [125342](https://dashboard.internetcomputer.org/proposal/125342)
- [125321](https://dashboard.internetcomputer.org/proposal/125321)
- [125320](https://dashboard.internetcomputer.org/proposal/125320)
- [125002](https://dashboard.internetcomputer.org/proposal/125002)
- [125001](https://dashboard.internetcomputer.org/proposal/125001)
- [124858](https://dashboard.internetcomputer.org/proposal/124858)
- [124857](https://dashboard.internetcomputer.org/proposal/124857)

### Bitcoin canister

Updated Bitcoin canister to [release/2023-10-13](https://github.com/dfinity/bitcoin-canister/releases/tag/release%2F2023-10-13)

# 0.15.1

### feat: Added support for reserved_cycles and reserved_cycles_limit

`dfx canister status` will now display the reserved cycles balance and reserved cycles limit for a canister.

Added command-line options:
  - `dfx canister create --reserved-cycles-limit <limit>`
  - `dfx canister update-settings --reserved-cycles-limit <limit>`

In addition, `dfx deploy` will set `reserved_cycles_limit` when creating canisters if specified in `canisters.<canister>.initialization_values.reserved_cycles_limit` in dfx.json.

### feat: emit management canister idl when imported by Motoko canister

`import management "ic:aaaaa-aa;`

This will automatically produce the idl in the `.dfx` folder.

### fix: Include remote canisters in canisters_to_generate

Generate frontend declarations for remote canisters too because frontend JS code may want to call them.

### feat: `dfx extension install <extension> --version <specific version>`

Install a specific version of an extension, bypassing version checks.

### feat: Updated handling of missing values in state tree certificates

The `Unknown` lookup of a path in a certificate results in an `AgentError` (the IC returns `Absent` for non-existing paths).

### fix: dfx deploy urls printed for asset canisters

### chore: --emulator parameter is deprecated and will be discontinued soon

Added warning that the `--emulator` is deprecated and will be discontinued soon.

### fix: node engines in starter

Updates node engines to reflect the same engines supported in agent-js.
```
"node": "^12 || ^14 || ^16 || >=17",
"npm": "^7.17 || >=8"
```

### feat: deploy to playground

Introduced a new network type called `playground`. Canisters on such networks are not created through standard means, but are instead borrowed from a canister pool.
The canisters time out after a while and new canisters need to be borrowed for further deployments.
To define custom playground networks, use a network definition that includes the `playground` key:
```json
"<network name>": {
  "playground": {
    "playground_canister": "<canister pool id>",
    "timeout_seconds": <amount of seconds after which a canister is returned to the pool>
  }
}
```

Introduced a new network that is available by default called `playground`. Additionally, `--playground` is an alias for `--network playground`.
By default, this network targets the Motoko Playground backend to borrow canisters. The borrowed canisters will be available for 20 minutes, and the timer restarts on new deployments.
When the timer runs out the canister(s) will be uninstalled and are returned to the pool.
Any commands that allow choosing a target network (e.g. `dfx canister call`) require `--playground` or `--network playground` in order to target the borrowed canister(s).
Use `dfx deploy --playground` to deploy simple projects to a canister borrowed from the Motoko Playground.

### feat: `--ic` is shorthand for `--network ic`

For example, `dfx deploy --ic` rather than `dfx deploy --network ic`.

### fix: Motoko base library files in cache are no longer executable

### feat: `dfx start` for shared network warns if ignoring 'defaults' in dfx.json

Background: In order to determine whether to start a project-specific network or the shared network, `dfx start` looks for the `local` network in dfx.json.
   - If found, `dfx start` starts the project-specific local network, applying any `defaults` from dfx.json.
   - If there is no dfx.json, or if dfx.json does not define a `local` network, `dfx start` starts the shared network.  Because the shared network is not specific to any project, `dfx start` ignores any other settings from dfx.json, including `defaults`.

If `dfx start` is starting the shared network from within a dfx project, and that dfx.json contains settings in the `defaults` key for `bitcoin`, `replica`, or `canister_http`, then `dfx start` will warn that it is ignoring those settings.  It will also describe how to define equivalent settings in networks.json.

### fix: dfx canister call --wallet no longer passes the parameter twice

The parameter was erroneously passed twice.  Now it is passed only once.

### fix: Removed deprecation warning about project-specific networks

Removed this warning: "Project-specific networks are deprecated and will be removed after February 2023." While we may remove project-specific networks in the future, it is not imminent.  One key requirement is the ability to run more than one subnet type at one time.

## Dependencies

### icx-proxy

Updated to a version of the icx-proxy that is released with the replica and other related binaries.

Changes in behavior:
- "%%" is no longer accepted when url-decoding filenames for the asset canister.  Though curl supports this, it's not part of the standard. Please replace with %25.
- The icx-proxy now performs response verification.  This has exposed some bugs in the asset canister.  However, since this new icx-proxy matches what the boundary nodes use, this will better match the behavior seen on the mainnet.
- Bugs that this has exposed in the asset canister:
  - after disabling aliasing for an asset, the asset canister will return an incorrect certification in the 404 response.
  - after setting a custom "etag" header in .ic-assets.json, the asset canister will return an incorrect certification in the 200 response.
  - assets with certain characters in the filename (example: "æ") will no longer be served correctly.  The definition of "certain characters" is not yet known.

### Candid UI

- Module hash: 934756863c010898a24345ce4842d173b3ea7639a8eb394a0d027a9423c70b5c
- Add `merge_init_args` method in Candid UI.
- Draw flamegraph for canister upgrade.

### Frontend canister

For certification v1, if none of the requested encoding are certified but another encoding is certified, then the frontend canister once again returns the certificatie even though the response hash won't match.
This allows the verifying side to try to transform the response such that it matches the response hash.
For example, if only the encoding `gzip` is requested but the `identity` encoding is certified, the `gzip` encoding is returned with the certificate for the `identity` encoding.
The verifying side can then unzip the response and will have a valid certificate for the `identity` response.

- Module hash: baf9bcab2ebc2883f850b965af658e66725087933df012ebd35c03929c39efe3
- https://github.com/dfinity/sdk/pull/3369
- https://github.com/dfinity/sdk/pull/3298
- https://github.com/dfinity/sdk/pull/3281

### Replica

Updated replica to elected commit 91bf38ff3cb927cb94027d9da513cd15f91a5b04.
This incorporates the following executed proposals:

- [124795](https://dashboard.internetcomputer.org/proposal/124795)
- [124790](https://dashboard.internetcomputer.org/proposal/124790)
- [124538](https://dashboard.internetcomputer.org/proposal/124538)
- [124537](https://dashboard.internetcomputer.org/proposal/124537)
- [124488](https://dashboard.internetcomputer.org/proposal/124488)
- [124487](https://dashboard.internetcomputer.org/proposal/124487)

# 0.15.0

## DFX

### chore: add `--use-old-metering` flag

The `use-old-metering` flag enables old metering in replica. The new metering is enabled in the `starter` by default, so this flag is to compare the default new metering with the old one.

The flag is temporary and will be removed in a few months.

### fix: added https://icp-api.io to the default Content-Security-Policy header

Existing projects will need to change this value in .ic-assets.json or .ic-assets.json5 to include https://icp-api.io

All projects will need to redeploy.

### fix: access to raw assets is now enabled by default

The default value for `allow_raw_access` is now `true`.  This means that by default, the frontend canister will no longer restrict the access of traffic to the `<canister-id>.raw.icp0.io` domain, and will no longer automatically redirect all requests to the certified domain (`<canister-id>.icp0.io`), unless configured explicitly.

Note that existing projects that specify `"allow_raw_access": false` in .ic-assets.json5 will need to change or remove this value manually in order to allow raw access.

### feat!: Removed dfx nns and dfx sns commands

Both have now been turned into the dfx extensions. In order to obtain them, please run `dfx extension install nns` and `dfx extension install sns` respectively. After the installation, you can use them as you did before: `dfx nns ...`, and `dfx sns ...`.

### feat!: Removed dfx replica and dfx bootstrap commands

Use `dfx start` instead.  If you have a good reason why we should keep these commands, please contribute to the discussion at https://github.com/dfinity/sdk/discussions/3163

### fix: Wait for new module hash when installing wallet

A previous change made dfx wait after installing a canister until the replica updated its reported module hash, but this change did not affect wallets. Now dfx waits for wallets too, to eliminate a class of wallet installation errors.

### fix: Ctrl-C right after dfx start will hang for minutes and panics

Early break out from actors starting procedure.

### feat: can disable the warnings about using an unencrypted identity on mainnet

It's now possible to suppress warnings of this form:

```
WARN: The <identity> identity is not stored securely. Do not use it to control a lot of cycles/ICP. Create a new identity with `dfx identity new` and use it in mainnet-facing commands with the `--identity` flag
```

To do so, export the environment variable `DFX_WARNING` with the value `-mainnet_plaintext_identity`.
```bash
export DFX_WARNING="-mainnet_plaintext_identity"
```

Note that this can be combined to also disable the dfx version check warning:
```bash
export DFX_WARNING="-version_check,-mainnet_plaintext_identity"
```

### fix!: restrict `dfx identity new` to safe characters

New identities like `dfx identity new my/identity` or `dfx identity new 'my identity'` can easily lead to problems, either for dfx internals or for usability.
New identities are now restricted to the characters `ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz.-_@0123456789`.
Existing identities are not affected by this change.

## Frontend canister

> **NOTE**: We've re-enabled response verification v2 in the asset canister.

### fix: Certification for aliasing updates on asset deletion

Best explained by an example: Two assets exist with aliasing enabled: `/content` and `/content.html`. Usually, when requesting `/content`, `/content.html` is served because it has aliasing enabled.
But in this scenario, because `/content` exists, it overwrites the alias and `/content` is served when requesting the path `/content`.
When the file `/content` is deleted, `/content` is once again a valid alias of `/content.html`.
Previously, the alias of `/content.html` was not properly updated in the certification tree, making `/content` inaccessible.

### fix: 404 response is now certified for certification v2

Certification v2 allows certifying arbitrary responses. If the requested file does not exist, and the fallback file (`/index.html`) does not exist either,
the frontend canister serves a HTTP 404 response. This response was previously not certified.

### fix!: The CreateAsset batch operation now fails if the asset already exists

Previously, the operation was a no-op if the content type matched, but ignored other, possibly different, asset properties. Now, it fails with an error.

### fix!: http_request_streaming_callback and get_chunk now require the sha256 parameter to be set

The `http_request_streaming_callback()` and `get_chunk()` methods use the `sha256` parameter to ensure that the chunks they return are part of the same asset contents returned by the initial call.  This parameter is now required to be Some(hash).

For `http_request()` and `http_request_streaming_callback()`, there should be no change: all callers of `http_request_streaming_callback()` are expected to pass the entire token returned by `http_request()`, which includes the sha256 parameter.

Any callers of `get_chunk()` should make sure to always pass the `sha256` value returned by the `get()` method.  It will always be present.

## Dependencies

### Motoko

Updated Motoko to [0.9.7](https://github.com/dfinity/motoko/releases/tag/0.9.7)

### Updated candid to 0.9.0

### Candid UI

- Module hash: b9173bb25dabe5e2b736a8f2816e68fba14ca72132f5485ce7b8f16a85737a17
- https://github.com/dfinity/sdk/pull/3260
- https://github.com/dfinity/sdk/pull/3252
- https://github.com/dfinity/candid/pull/449
- https://github.com/dfinity/candid/pull/453

### Frontend canister

- Module hash: e20be8df2c392937a6ae0f70d20ff23b75e8c71d9085a8b8bb438b8c2d4eafe5
- https://github.com/dfinity/sdk/pull/3337
- https://github.com/dfinity/sdk/pull/3298
- https://github.com/dfinity/sdk/pull/3256
- https://github.com/dfinity/sdk/pull/3252
- https://github.com/dfinity/sdk/pull/3249
- https://github.com/dfinity/sdk/pull/3212
- https://github.com/dfinity/sdk/pull/3227

### Replica

Updated replica to elected commit cabe2ae3ca115b1a3f24d75814d4f8e317b2964d.
This incorporates the following executed proposals:

- [124331](https://dashboard.internetcomputer.org/proposal/124331)
- [124330](https://dashboard.internetcomputer.org/proposal/124330)
- [124272](https://dashboard.internetcomputer.org/proposal/124272)
- [124021](https://dashboard.internetcomputer.org/proposal/124021)
- [123977](https://dashboard.internetcomputer.org/proposal/123977)
- [123976](https://dashboard.internetcomputer.org/proposal/123976)
- [123922](https://dashboard.internetcomputer.org/proposal/123922)
- [123784](https://dashboard.internetcomputer.org/proposal/123784)
- [123730](https://dashboard.internetcomputer.org/proposal/123730)
- [123711](https://dashboard.internetcomputer.org/proposal/123711)
- [123474](https://dashboard.internetcomputer.org/proposal/123474)
- [123410](https://dashboard.internetcomputer.org/proposal/123410)
- [123311](https://dashboard.internetcomputer.org/proposal/123311)

# 0.14.2

## DFX

### feat: deprecate `dfx bootstrap` and `dfx replica` commands

Please use `dfx start` instead, which is a combination of the two commands.

If you have a good reason why we should keep these commands, please contribute to the discussion at https://github.com/dfinity/sdk/discussions/3163

### feat: add optional custom build command for asset canisters

The custom build command can be set in `dfx.json` the same way it is set for `custom` type canisters. If the command is not provided, DFX will fallback to the default `npm run build` command.

```json
{
  "canisters": {
    "ui": {
      "type": "assets",
      "build": ["<custom build command>"]
    }
  }
}
```

### fix: Diagnose duplicate assets and display upgrade steps

If `dfx deploy` detects duplicate assets in the dist/ and frontend assets/ directories, it will now suggest upgrade steps.

### fix: motoko canisters can import other canisters with service constructor

After specific canister builder output wasm and candid file, `dfx` will do some post processing on the candid file.

The complete IDL will be copied into `.dfx` folder with name `constructor.did`.
It will be used for type checking during canister installation.

Then it is separated into two parts: `service.did` and `init_args.txt`, corresponding to canister metadata `candid:service` and `candid:args`.

`service.did` will be imported during dependent canisters building. And it will also be used by the Motoko LSP to provide IDE support.

### fix: dfx start now respects the network replica port configuration in dfx.json or networks.json

## Frontend canister

> **NOTE**: We've disabled response verification v2 in the asset canister while we improve test coverage.

The redirect from `.raw.ic0.app` now redirects to `.ic0.app` instead of `.icp0.io`

The `validate_commit_proposed_batch()` method no longer requires any permission to call.

The asset canister now enforces limits during upload.  These limits to not apply to assets already uploaded.

Unconditional limits:
- `create_batch()` now fails if `dfx deploy --by-proposal` got as far as calling `propose_commit_batch()`, and the batch has not since been committed or deleted.

Configurable limits:
- `max_batches`: limits number of batches being uploaded.
- `max_chunks`: limits total number of chunks across all batches being uploaded.
- `max_bytes`: limits total size of content bytes across all chunks being uploaded.

Added methods:
- `configure()` to set limits
- `validate_configure()`: companion method for SNS
- `get_configuration()`: to view limits

Suggestions for configured limits:
- dapps controlled by SNS: max_batches=1; max_chunks and max_bytes based on asset composition.
- dapps not controlled by SNS: unlimited (which is the default)

Note that as always, if `dfx deploy` does not completely upload and commit a batch, the asset canister will retain the batch until 5 minutes have passed since the last chunk was uploaded.  If you have configured limits and the combination of an unsuccessful deployment and a subsequent attempt would exceed those limits, you can either wait 5 minutes before running `dfx deploy` again, or delete the incomplete batch with `delete_batch()`.

### fix: return the correct expr_path for index.html fallback routes

Previously, the requested path was used to construct the `expr_path` for the `index.html` fallback route.  This was incorrect, as the `expr_path` should be the path of the `index.html` file itself in this case.

## Frontend canister assets synchronization

### fix: now retries failed `create_chunk()` calls

Previously, it would only retry when waiting for the request to complete.

### fix: now considers fewer error types to be retryable

Previously, errors were assumed to be retryable, except for a few specific error messages and 403/unauthorized responses.  This could cause deployment to appear to hang until timeout.

Now, only transport errors and timeout errors are considered retryable.

## Dependencies

### Frontend canister

- Module hash: 1286960c50eb7a773cfb5fdd77cc238588f39e21f189cc3eb0f35199a99b9c7e
- https://github.com/dfinity/sdk/pull/3205
- https://github.com/dfinity/sdk/pull/3198
- https://github.com/dfinity/sdk/pull/3154
- https://github.com/dfinity/sdk/pull/3158
- https://github.com/dfinity/sdk/pull/3144

### ic-ref

Updated ic-ref to 0.0.1-a9f73dba

### Cycles wallet

Updated cycles wallet to `20230530` release:
- Module hash: c1290ad65e6c9f840928637ed7672b688216a9c1e919eacbacc22af8c904a5e3
- https://github.com/dfinity/cycles-wallet/commit/313fb01d59689df90bd3381659d94164c2a61cf4

### Motoko

Updated Motoko to 0.9.3

### Replica

Updated replica to elected commit ef8ca68771baa20a14af650ab89c9b31b1dc9a5e.
This incorporates the following executed proposals:
- [123248](https://dashboard.internetcomputer.org/proposal/123248)
- [123021](https://dashboard.internetcomputer.org/proposal/123021)
- [123007](https://dashboard.internetcomputer.org/proposal/123007)
- [122923](https://dashboard.internetcomputer.org/proposal/122923)
- [122924](https://dashboard.internetcomputer.org/proposal/122924)
- [122910](https://dashboard.internetcomputer.org/proposal/122910)
- [122911](https://dashboard.internetcomputer.org/proposal/122911)
- [122746](https://dashboard.internetcomputer.org/proposal/122746)
- [122748](https://dashboard.internetcomputer.org/proposal/122748)
- [122617](https://dashboard.internetcomputer.org/proposal/122617)
- [122615](https://dashboard.internetcomputer.org/proposal/122615)

# 0.14.1

## DFX

### fix: `dfx canister delete` without stopping first

When running `dfx canister delete` on a canister that has not been stopped, dfx will now confirm the deletion instead of erroring.

### feat: gzip option in dfx.json

`dfx` can gzip wasm module as the final step in building canisters.

This behavior is disabled by default.

You can enable it in `dfx.json`:

```json
{
  "canisters" : {
    "app" : {
      "gzip" : true
    }
  }
}
```

You can still specify `.wasm.gz` file for custom canisters directly. If any metadata/optimize/shrink options are set in `dfx.json`, the `.wasm.gz` file will be decompressed, applied all the wasm modifications, and compressed as `.wasm.gz` in the end.

### fix: prevented using --argument with --all in canister installation

Removed `dfx deploy`'s behavior of providing the same argument to all canisters, and `dfx canister install`'s behavior of providing an empty argument to all canisters regardless of what was specified. Now installing multiple canisters and providing an installation argument is an error in both commands.

### chore: make `sns` subcommands visible in `dfx help`

### chore: upgraded to clap v4

Updated the command-parsing library to v4. Some colors may be different.

### feat: dfx deps subcommands

This feature was named `dfx pull` before. To make a complete, intuitive user experience, we present a set of subcommands under `dfx deps`:

- `dfx deps pull`: pull the dependencies from mainnet and generate `deps/pulled.json`, the candid files of direct dependencies will also be put into `deps/candid/`;
- `dfx deps init`: set the init arguments for the pulled dependencies and save the data in `deps/init.json`;
- `dfx deps deploy`: deploy the pulled dependencies on local replica with the init arguments recorded in `deps/init.json`;

All generated files in `deps/` are encouraged to be version controlled.

### chore: Add the `nns-dapp` and `internet_identity` to the local canister IDs set by `dfx nns import`
`dfx nns install` installs a set of canisters in a local replica.  `dfx nns import` complements this by setting the canister IDs so that they can be queried by the user.  But `dfx nns import` is incomplete.  Now it will also provide the IDs of the `nns-dapp` and `internet_identity` canisters.

### feat: `.env` file includes all created canister IDs
Previously the `.env` file only included canister IDs for canisters that were listed as explicit dependencies during the build process.
Now all canisters that have a canister ID for the specified network are included in `.env`.

### feat!: Ask for user consent when removing themselves as principal

Removing oneself (or the wallet one uses) can result in the loss of control over a canister.
Therefore `dfx canister update-settings` now asks for extra confirmation when removing the currently used principal/wallet from the list of controllers.
To skip this check in CI, use either the `--yes`/`-y` argument or use `echo "yes" | dfx canister update-settings <...>`.

### fix: dfx start will restart replica if it does not report healthy after launch

If the replica does not report healthy at least once after launch,
dfx will terminate and restart it.

### fix: dfx start now installs the bitcoin canister when bitcoin support is enabled

This is required for future replica versions.

Adds a new field `canister_init_arg` to the bitcoin configuration in dfx.json and networks.json.  Its default is documented in the JSON schema and is appropriate for the canister wasm bundled with dfx.

### fix: no longer enable the bitcoin_regtest feature

### docs: cleanup of documentation

Cleaned up documentation of IC SDK.

## Asset Canister Synchronization

### feat: Added more detailed logging to `ic-asset`.

Now, `dfx deploy -v` (or `-vv`) will print the following information:
- The count for each `BatchOperationKind` in `CommitBatchArgs`
- The number of chunks uploaded and the total bytes
- The API version of both the `ic-asset` and the canister
- (Only for `-vv`) The value of `CommitBatchArgs`

### fix: Commit batches incrementally in order to account for more expensive v2 certification calculation

In order to allow larger changes without exceeding the per-message instruction limit, the sync process now:
- sets properties of assets already in the canister separately from the rest of the batch.
- splits up the rest of the batch into groups of up to 500 operations.

### fix: now retries failed `create_chunk()` calls

Previously, it would only retry when waiting for the request to complete.

### fix: now considers fewer error types to be retryable

Previously, errors were assumed to be retryable, except for a few specific error messages and 403/unauthorized responses.  This could cause deployment to appear to hang until timeout.

Now, only transport errors and timeout errors are considered retryable.

## Dependencies

### Frontend canister

The asset canister now properly removes the v2-certified response when `/index.html` is deleted.

Fix: The fallback file (`/index.html`) will now be served when using certification v2 if the requested path was not found.

The HttpResponse type now explicitly mentions the `upgrade : Option<bool>` field instead of implicitly returning `None` all the time.

The asset canister no longer needs to use `await` for access control checks. This will speed up certain operations.

- Module hash: 651425d92d3796ddae581191452e0e87484eeff4ff6352fe9a59c7e1f97a2310
- https://github.com/dfinity/sdk/pull/3120
- https://github.com/dfinity/sdk/pull/3112

### Motoko

Updated Motoko to 0.8.8

### Replica

Updated replica to elected commit b3b00ba59c366384e3e0cd53a69457e9053ec987.
This incorporates the following executed proposals:
- [122529](https://dashboard.internetcomputer.org/proposal/122529)
- [122284](https://dashboard.internetcomputer.org/proposal/122284)
- [122198](https://dashboard.internetcomputer.org/proposal/122198)
- [120591](https://dashboard.internetcomputer.org/proposal/120591)
- [119318](https://dashboard.internetcomputer.org/proposal/119318)
- [118023](https://dashboard.internetcomputer.org/proposal/118023)
- [116294](https://dashboard.internetcomputer.org/proposal/116294)
- [116135](https://dashboard.internetcomputer.org/proposal/116135)
- [114479](https://dashboard.internetcomputer.org/proposal/114479)
- [113136](https://dashboard.internetcomputer.org/proposal/113136)
- [111932](https://dashboard.internetcomputer.org/proposal/111932)
- [111724](https://dashboard.internetcomputer.org/proposal/111724)
- [110724](https://dashboard.internetcomputer.org/proposal/110724)
- [109500](https://dashboard.internetcomputer.org/proposal/109500)
- [108153](https://dashboard.internetcomputer.org/proposal/108153)
- [107668](https://dashboard.internetcomputer.org/proposal/107668)
- [107667](https://dashboard.internetcomputer.org/proposal/107667)
- [106868](https://dashboard.internetcomputer.org/proposal/106868)
- [106817](https://dashboard.internetcomputer.org/proposal/106817)
- [105666](https://dashboard.internetcomputer.org/proposal/105666)
- [104470](https://dashboard.internetcomputer.org/proposal/104470)
- [103281](https://dashboard.internetcomputer.org/proposal/103281)
- [103231](https://dashboard.internetcomputer.org/proposal/103231)
- [101987](https://dashboard.internetcomputer.org/proposal/101987)

# 0.14.0

## DFX

### fix: stop `dfx deploy` from creating a wallet if all canisters exist

### feat: expose `wasm-opt` optimizer in `ic-wasm` to users

Add option to specify an "optimize" field for canisters to invoke the `wasm-opt` optimizer through `ic-wasm`.

This behavior is disabled by default.

If you want to enable this behavior, you can do so in dfx.json:
```json
"canisters": {
  "app": {
    "optimize" : "cycles"
  }
}
```

The options are "cycles", "size", "O4", "O3", "O2", "O1", "O0", "Oz", and "Os".  The options starting with "O" are the optimization levels that `wasm-opt` provides. The "cycles" and "size" options are recommended defaults for optimizing for cycle usage and binary size respectively.

### feat: updates the dfx new starter project for env vars

- Updates the starter project for env vars to use the new `dfx build` & `dfx deploy` environment variables
- Changes the format of the canister id env vars to be `CANISTER_ID_<canister_name_uppercase>`, for the frontend declaraction file to be consistent with the dfx environment variables. `CANISTER_ID` as both a prefix and suffix are supported for backwards compatibility.

### fix!: --clean required when network configuration changes

If the network configuration has changed since last time `dfx start` was run, `dfx start` will now error if you try to run it without `--clean`, to avoid spurious errors. You can provide the `--force` flag if you are sure you want to start it without cleaning state.

### feat: --artificial-delay flag

The local replica uses a 600ms delay by default when performing update calls. With `dfx start --artificial-delay <ms>`, you can decrease this value (e.g. 100ms) for faster integration tests, or increase it (e.g. 2500ms) to mimick mainnet latency for e.g. UI responsiveness checks.

### fix: make sure assetstorage did file is created as writeable.

### feat: specify id when provisional create canister

When creating a canister on non-mainnet replica, you can now specify the canister ID.

`dfx canister create <CANISTER_NAME> --specified-id <PRINCIPAL>`

`dfx deploy <CANISTER_NAME> --specified-id <PRINCIPAL>`

You can specify the ID in the range of `[0, u64::MAX / 2]`.
If not specify the ID, the canister will be created in the range of `[u64::MAX / 2 + 1, u64::MAX]`.
This canister ID allocation behavior only applies to the replica, not the emulator (ic-ref).

### feat: dfx nns install --ledger-accounts

`dfx nns install` now takes an option `--ledger-accounts` to initialize the ledger canister with these accounts.

### fix: update Rust canister template.

`ic-cdk-timers` is included in the dependencies.

### chore: change the default Internet Computer gateway domain to `icp0.io`

By default, DFX now uses the `icp0.io` domain to connect to Internet Computer as opposed to using `ic0.app`.
Canisters communicating with `ic0.app` will continue to function nominally.

### feat: --no-asset-upgrade

### feat: confirmation dialogues are no longer case sensitive and accept 'y' in addition to 'yes'

### fix: `dfx generate` no longer requires canisters to have a canister ID
Previously, canisters required that the canister was created before `dfx generate` could be called.

As a result, the `--network` parameter does not have an impact on the result of `dfx generate` anymore.
This means that `dfx generate` now also generates type declarations for remote canisters.

### fix: Make `build` field optional in dfx.json

The `build` field in custom canisters was already optional in code, but this fixes it in the schema.

By specifying the `--no-asset-upgrade` flag in `dfx deploy` or `dfx canister install`, you can ensure that the asset canister itself is not upgraded, but instead only the assets themselves are installed.

### feat: Get identity from env var if present

The identity may be specified using the environment variable `DFX_IDENTITY`.

### feat: Add DFX_ASSETS_WASM

Added the ability to configure the Wasm module used for assets canisters through the environment variable `DFX_ASSETS_WASM`.

### fix: dfx deploy and icx-asset no longer retry on permission failure

### feat: --created-at-time for the ledger functions: transfer, create-canister, and top-up

### fix: ledger transfer duplicate transaction prints the duplicate transaction response before returning success to differentiate between a new transaction response and between a duplicate transaction response.

Before it was possible that a user could send 2 ledger transfers with the same arguments at the same timestamp and both would show success but there would have been only 1 ledger transfer. Now dfx prints different messages when the ledger returns a duplicate transaction response and when the ledger returns a new transaction response.

### chore: clarify `dfx identity new` help text

### chore: Add a message that `redeem_faucet_coupon` may take a while to complete

### feat: `dfx deploy <frontend canister name> --by-proposal`

This supports asset updates through SNS proposal.

Uploads asset changes to an asset canister (propose_commit_batch()), but does not commit them.

The SNS will call `commit_proposed_batch()` to commit the changes.  If the proposal fails, the caller of `dfx deploy --by-proposal` should call `delete_batch()`.

### feat: `dfx deploy <frontend canister name> --compute-evidence`

Builds the specified asset canister, determines the batch operations required to synchronize the assets, and computes a hash ("evidence") over those batch operations.  This evidence will match the evidence computed by `dfx deploy --by-proposal`, and which will be specified in the update proposal.

No permissions are required to compute evidence, so this can be called with `--identity anonymous` or any other identity.

## Asset Canister

Added `validate_take_ownership()` method so that an SNS is able to add a custom call to `take_ownership()`.

Added `is_aliased` field to `get_asset_properties` and `set_asset_properties`.

Added partial support for proposal-based asset updates:
- Batch ids are now stable.  With upcoming changes to support asset updates by proposal,
  having the asset canister not reuse batch ids will make it easier to verify that a particular
  batch has been proposed.
- Added methods:
  - `propose_commit_batch()` stores batch arguments for later commit
  - `delete_batch()` deletes a batch, intended for use after compute_evidence if cancellation needed
  - `compute_evidence()` computes a hash ("evidence") over the proposed batch arguments. Once evidence computation is complete, batch will not expire.
  - `commit_proposed_batch()` commits batch previously proposed (must have evidence computed)
  - `validate_commit_proposed_batch()` required validation method for SNS

Added `api_version` endpoint. With upcoming changes we will introduce breaking changes to asset canister's batch upload process. New endpoint will help `ic-asset` with differentiation between API version, and allow it to support all versions of the asset canister.

Added support for v2 asset certification. In comparison to v1, v2 asset certification not only certifies the http response body, but also the headers. The v2 spec is first published in [this PR](https://github.com/dfinity/interface-spec/pull/147)

Added canister metadata field `supported_certificate_versions`, which contains a comma-separated list of all asset certification protocol versions. You can query it e.g. using `dfx canister --network ic metadata <canister name or id> supported_certificate_versions`. In this release, the value of this metadata field value is `1,2` because certification v1 and v2 are supported.

Fixed a bug in `http_request` that served assets with the wrong certificate. If no encoding specified in the `Accept-Encoding` header is available with a certificate, an available encoding is returned without a certificate (instead of a wrong certificate, which was the case previously). Otherwise, nothing changed.
For completeness' sake, the new behavior is as follows:
- If one of the encodings specified in the `Accept-Encoding` header is available with certification, it now is served with the correct certificate.
- If no requested encoding is available with certification, one of the requested encodings is returned without a certificate (instead of a wrong certificate, which was the case previously).
- If no encoding specified in the `Accept-Encoding` header is available, a certified encoding that is available is returned instead.

Added support for API versioning of the asset canister in `ic-asset`.

Added functionality that allows you to set asset properties during `dfx deploy`, even if the asset has already been deployed to a canister in the past. This eliminates the need to delete and re-deploy assets to modify properties - great news! This feature is also available when deploying assets using the `--by-proposal` flag. As a result, the API version of the frontend canister has been incremented from `0` to `1`. The updated `ic-asset` version (which is what is being used during `dfx deploy`) will remain compatible with frontend canisters implementing both API `0` and `1`. However, please note that the new frontend canister version (with API `v1`) will not work with tooling from before the dfx release (0.14.0).

## Dependencies

### Frontend canister

- API version: 1
- Module hash: e7866e1949e3688a78d8d29bd63e1c13cd6bfb8fbe29444fa606a20e0b1e33f0
- https://github.com/dfinity/sdk/pull/3094
- https://github.com/dfinity/sdk/pull/3002
- https://github.com/dfinity/sdk/pull/3065
- https://github.com/dfinity/sdk/pull/3058
- https://github.com/dfinity/sdk/pull/3057
- https://github.com/dfinity/sdk/pull/2960
- https://github.com/dfinity/sdk/pull/3051
- https://github.com/dfinity/sdk/pull/3034
- https://github.com/dfinity/sdk/pull/3023
- https://github.com/dfinity/sdk/pull/3022
- https://github.com/dfinity/sdk/pull/3021
- https://github.com/dfinity/sdk/pull/3019
- https://github.com/dfinity/sdk/pull/3016
- https://github.com/dfinity/sdk/pull/3015
- https://github.com/dfinity/sdk/pull/3001
- https://github.com/dfinity/sdk/pull/2987
- https://github.com/dfinity/sdk/pull/2982

### Motoko

Updated Motoko to 0.8.7

### ic-ref

Updated ic-ref to 0.0.1-ca6aca90

### ic-btc-canister

Started bundling ic-btc-canister, release 2023-03-31

# 0.13.1

## Asset Canister

Added validate_grant_permission() and validate_revoke_permission() methods per SNS requirements.

## Dependencies

### Frontend canister

- Module hash: 98863747bb8b1366ae5e3c5721bfe08ce6b7480fe4c3864d4fec3d9827255480
- https://github.com/dfinity/sdk/pull/2958

# 0.13.0

## DFX

### feat: Add dfx sns download

This allows users to download SNS canister Wasm binaries.

### fix: fixed error text
- `dfx nns install` had the wrong instructions for setting up the local replica type

### fix: creating an identity with `--force` no longer switches to the newly created identity

### feat(frontend-canister)!: reworked to use permissions-based access control

The permissions are as follows:
- ManagePermissions: Can grant and revoke permissions to any principal.  Controllers implicitly have this permission.
- Prepare: Can call create_batch and create_chunk
- Commit: Can call commit_batch and methods that manipulate assets directly, as well as any method permitted by Prepare.

For upgraded frontend canisters, all authorized principals will be granted the Commit permission.
For newly deployed frontend canisters, the initializer (first deployer of the canister) will be granted the Commit permission.

Added three new methods:
- list_permitted: lists principals with a given permission.
  - Callable by anyone.
- grant_permission: grants a single permission to a principal
  - Callable by Controllers and principals with the ManagePermissions permission.
- revoke_permission: removes a single permission from a principal
  - Any principal can revoke its own permissions.
  - Only Controllers and principals with the ManagePermissions permission can revoke the permissions of other principals.

Altered the behavior of the existing authorization-related methods to operate only on the "Commit" permission.  In this way, they are backwards-compatible.
- authorize(principal): same as grant_permission(principal, Commit)
- deauthorize(principal): same as revoke_permission(permission, Commit)
- list_authorized(): same as list_permitted(Commit)

### fix(frontend-canister)!: removed ability of some types of authorized principals to manage the ACL

It used to be the case that any authorized principal could authorize and deauthorize any other principal.
This is no longer the case.  See rules above for grant_permission and revoke_permission.

### feat(frontend-canister)!: default secure configuration for assets in frontend project template

- Secure HTTP headers, preventing several typical security vulnerabilities (e.g. XSS, clickjacking, and many more). For more details, see comments in `headers` section in [default `.ic-assets.json5`](https://raw.githubusercontent.com/dfinity/sdk/master/src/dfx/assets/new_project_node_files/src/__project_name___frontend/src/.ic-assets.json5).
- Configures `allow_raw_access` option in starter `.ic-assets.json5` config files, with the value set to its default value (which is `false`). We are showing that configuration in the default starter projects for the sake of easier discoverability, even though its value is set to the default.

### feat(frontend-canister)!: add `allow_raw_access` config option

By default, the frontend canister will now restrict the access of traffic to the `<canister-id>.raw.ic0.app` domain, and will automatically redirect all requests to the certified domain (`<canister-id>.ic0.app`), unless configured explicitly. Below is an example configuration to allow access to the `robots.txt` file from the "raw" domain:
```json
[
  {
    "match": "robots.txt",
    "allow_raw_access": true
  }
]
```

**Important**: Note that any assets already uploaded to an asset canister will be protected by this redirection, because at present the asset synchronization process does not update the `allow_raw_access` property, or any other properties, after creating an asset.  This also applies to assets that are deployed without any configuration, and later configured to allow raw access.
At the present time, there are two ways to reconfigure an existing asset:
1. re-create the asset
    1. delete the asset in your project's directory
    1. execute `dfx deploy`
    1. re-create the asset in your project's directory
    1. modify `.ic-assets.json` acordingly
    1. execute `dfx deploy`
2. via manual candid call
    ```
    dfx canister call PROJECT_NAME_frontend set_asset_properties '( record { key="/robots.txt"; allow_raw_access=opt(opt(true)) })'
    ```

### feat(frontend-canister): pretty print asset properties when deploying assets to the canister

### feat(frontend-canister): add take_ownership() method

Callable only by a controller.  Clears list of authorized principals and adds the caller (controller) as the only authorized principal.

### feat(ic-ref):
- `effective_canister_id` used for `provisional_create_canister_with_cycles` is passed as an command-line argument (defaults to `rwlgt-iiaaa-aaaaa-aaaaa-cai` if not provided or upon parse failure)

### feat(frontend-canister): add `get_asset_properties` and `set_asset_properties` to frontend canister

As part of creating the support for future work, it's now possible to get and set AssetProperties for assets in frontend canister.

### feat: add `--argument-file` argument to the `dfx canister sign` command

Similar to how this argument works in `dfx canister call`, this argument allows providing arguments for the request from a file.

### feat: Add support for a default network key

A remote canister ID can now be specified for the `__default` network.  If specified, `dfx` will assume that the canister is remote at the specified canister ID for all networks that don't have a dedicated entry.

### feat: use OS-native keyring for pem file storage

If keyring integration is available, PEM files (except for the default identity) are now by default stored in the OS-provided keyring.
If it is not available, it will fall back on the already existing password-encrypted PEM files.
Plaintext PEM files are still available (e.g. for use in non-interactive situations like CI), but not recommended for use since they put the keys at risk.

To force the use of one specific storage mode, use the `--storage-mode` flag with either `--storage-mode password-protected` or `--storage-mode plaintext`.
This works for both `dfx identity new` and `dfx identity import`.

The flag `--disable-encryption` is deprecated in favour of `--storage-mode plaintext`. It has the same behavior.

### feat(frontend-canister): better control and overview for asset canister authorized principals

The asset canister now has two new functions:
- Query function `list_authorized` displays a list of all principals that are currently authorized to change assets and the list of authorized principals.
- Update function `deauthorize` that removes a principal from the list of authorized principals. It can be called by authorized principals and cotrollers of the canister.

In addition, the update function `authorize` has new behavior:
Now, controllers of the asset canister are always allowed to authorize new principals (including themselves).

### fix: add retry logic to `dfx canister delete`

`dfx canister delete` tries to withdraw as many cycles as possible from a canister before deleting it.
To do so, dfx has to manually send all cycles in the canister, minus some margin.
The margin was previously hard-coded, meaning that withdrawals can fail if the margin is not generous enough.
Now, upon failure with some margin, dfx will retry withdrawing cycles with a continuously larger margin until withdrawing succeeds or the margin becomes larger than the cycles balance.

### fix: dfx deploy --mode reinstall for a single Motoko canister fails to compile

The Motoko compiler expects all imported canisters' .did files to be in one folder when it compiles a canister.
`dfx` failed to organize the .did files correctly when running `dfx deploy <single Motoko canister>` in combintaion with the `--mode reinstall` flag.

### fix: give more cycles margin when deleting canisters

There have been a few reports of people not being able to delete canisters.
The error happens if the temporary wallet tries to transfer out too many cycles.
The number of cycles left in the canister is bumped a little bit so that people can again reliably delete their canisters.

## Dependencies

Updated candid to 0.8.4
- Bug fix in TS bindings
- Pretty print numbers

### Frontend canister

- Module hash: d12e4493878911c21364c550ca90b81be900ebde43e7956ae1873c51504a8757
- https://github.com/dfinity/sdk/pull/2942

### ic-ref

Updated ic-ref to master commit `3cc51be5`

### Motoko

Updated Motoko to 0.7.6

### Replica

Updated replica to elected commit b5a1a8c0e005216f2d945f538fc27163bafc3bf7.
This incorporates the following executed proposals:

- [100821](https://dashboard.internetcomputer.org/proposal/100821)
- [97472](https://dashboard.internetcomputer.org/proposal/97472)
- [96114](https://dashboard.internetcomputer.org/proposal/96114)
- [94953](https://dashboard.internetcomputer.org/proposal/94953)
- [94852](https://dashboard.internetcomputer.org/proposal/94852)
- [93761](https://dashboard.internetcomputer.org/proposal/93761)
- [93507](https://dashboard.internetcomputer.org/proposal/93507)
- [92573](https://dashboard.internetcomputer.org/proposal/92573)
- [92338](https://dashboard.internetcomputer.org/proposal/92338)
- [91732](https://dashboard.internetcomputer.org/proposal/91732)
- [91257](https://dashboard.internetcomputer.org/proposal/91257)

# 0.12.1

## DFX

### fix: default not shrink for custom canisters

## Dependencies

### Replica

Updated replica to elected commit dcbf401f27d9b48354e68389c6d8293c4233b055.
This incorporates the following executed proposals:

- [90485](https://dashboard.internetcomputer.org/proposal/90485)
- [90008](https://dashboard.internetcomputer.org/proposal/90008)

### Frontend canister

- Module hash: db07e7e24f6f8ddf53c33a610713259a7c1eb71c270b819ebd311e2d223267f0
- https://github.com/dfinity/sdk/pull/2753

# 0.12.0

## DFX

### feat(frontend-canister): add warning if config is provided in `.ic-assets.json` but not used

### fix(frontend-canister): Allow overwriting default HTTP Headers for assets in frontend canister

Allows to overwrite `Content-Type`, `Content-Encoding`, and `Cache-Control` HTTP headers with custom values via `.ic-assets.json5` config file. Example `.ic-assets.json5` file:
```json5
[
    {
        "match": "web-gz.data.gz",
        "headers": {
            "Content-Type": "application/octet-stream",
            "Content-Encoding": "gzip"
        }
    }
]
```
This change will trigger the update process for frontend canister (new module hash: `2ff0513123f11c57716d889ca487083fac7d94a4c9434d5879f8d0342ad9d759`).

### feat: warn if an unencrypted identity is used on mainnet

### fix: Save SNS canister IDs

SNS canister IDs were not being parsed reliably.  Now the candid file is being specified explicitly, which resolves the issue in at least some cases.

### feat: NNS usability improvements

The command line interface for nns commands has been updated to:

- Give better help when the subnet type is incorrect
- Not offer --network as a flag given that it is unused
- List nns subcommands

### feat: -y flag for canister installation

`dfx canister install` and `dfx deploy` now have a `-y` flag that will automatically confirm any y/n checks made during canister installation.

### fix: Compute Motoko dependencies in linear (not exponential) time by detecting visited imports.

### fix(generate): add missing typescript types and fix issues with bindings array in dfx.json

### chore: update Candid UI canister with commit 79d55e7f568aec00e16dd0329926cc7ea8e3a28b

### refactor: Factor out code for calling arbitrary bundled binaries

The function for calling sns can now call any bundled binary.

### docs: Document dfx nns subcommands

`dfx nns` commands are used to deploy and manage local NNS canisters, such as:

- Governance for integration with the Internet Computer voting system
- Ledger for financial integration testing
- Internet Identity for user registration and authenttication

### feat(frontend-canister): Add simple aliases from `<asset>` to `<asset>.html` and `<asset>/index.html`

The asset canister now by default aliases any request to `<asset>` to `<asset>.html` or `<asset>/index.html`.
This can be disabled by setting the field `"enable_aliasing"` to `false` in a rule for that asset in .ic-assets.json.
This change will trigger frontend canister upgrades upon redeploying any asset canister.

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
```json
"canisters" : {
  "app" : {
    "shrink" : false,
  }
}
```

### feat: configurable custom wasm sections

It's now possible to define custom wasm metadata sections and their visibility in dfx.json.

At present, dfx can only add wasm metadata sections to canisters that are in wasm format.  It cannot add metadata sections to compressed canisters.  Since the frontend canister is now compressed, this means that at present it is not possible to add custom metadata sections to the frontend canister.

dfx no longer adds `candid:service` metadata to canisters of type `"custom"` by default.  If you want dfx to add your canister's candid definition to your custom canister, you can do so like this:

```
    "my_canister_name": {
      "type": "custom",
      "candid": "main.did",
      "wasm": "main.wasm",
      "metadata": [
        {
          "name": "candid:service"
        }
      ]
    },
```

This changelog entry doesn't go into all of the details of the possible configuration.  For that, please see [concepts/canister-metadata](docs/concepts/canister-metadata.md) and the docs in the JSON schema.

### fix: Valid canister-based env vars

Hyphens are not valid in shell environment variables, but do occur in canister names such as `smiley-dapp`. This poses a problem for vars with names such as `CANISTER_ID_$\{CANISTER_NAME\}`.  With this change, hyphens are replaced with underscores in environment variables.  The canister id of `smiley-dapp` will be available as `CANISTER_ID_smiley_dapp`.  Other environment variables are unaffected.

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

- note that dfx will report an error if a custom canister's `wasm` field is a URL and the canister also has `build` steps.

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
```
"http": {
  "enabled": true,
  "log_level": "info"
}
```

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

### fix: do not build or generate remote canisters

Canisters that specify a remote id for the network that's getting built falsely had their build steps run, blocking some normal use patterns of `dfx deploy`.
Canisters with a remote id specified no longer get built.
The same applies to `dfx generate`.

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

### fix: `dfx start` and `dfx stop` will take into account dfx/replica processes from dfx \<\= 0.11.x

### feat: added command `dfx info`

#### feat: `dfx info webserver-port`

This displays the port that the icx-proxy process listens on, meaning the port to connect to with curl or from a web browser.

#### feat: `dfx info replica-port`

This displays the listening port of the replica.

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

When installing a new Wasm module to a canister, DFX will now wait for the updated state (i.e. the new module hash) to be visible in the replica's certified state tree before proceeding with post-installation tasks or producing a success status.

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

### fix: broken link in new .mo project README

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

Updated replica to release candidate 93dcf2a2026c34330c76149dd713d89e37daa533.

This also incorporates the following executed proposals:

- [88831](https://dashboard.internetcomputer.org/proposal/88831)
- [88629](https://dashboard.internetcomputer.org/proposal/88629)
- [88109](https://dashboard.internetcomputer.org/proposal/88109)
- [87631](https://dashboard.internetcomputer.org/proposal/87631)
- [86738](https://dashboard.internetcomputer.org/proposal/86738)
- [86279](https://dashboard.internetcomputer.org/proposal/86279)
* [85007](https://dashboard.internetcomputer.org/proposal/85007)
* [84391](https://dashboard.internetcomputer.org/proposal/84391)
* [83786](https://dashboard.internetcomputer.org/proposal/83786)
* [82425](https://dashboard.internetcomputer.org/proposal/82425)
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

### Updated Motoko from 0.6.29 to 0.7.3

- See https://github.com/dfinity/motoko/blob/master/Changelog.md#073-2022-11-01


### Cycles wallet

- Module hash: b944b1e5533064d12e951621d5045d5291bcfd8cf9d60c28fef02c8fdb68e783
- https://github.com/dfinity/cycles-wallet/commit/fa86dd3a65b2509ca1e0c2bb9d7d4c5be95de378

### Frontend canister:
- Module hash: 6c8f7a094060b096c35e4c4499551e7a8a29ee0f86c456e521c09480ebbaa8ab
- https://github.com/dfinity/sdk/pull/2720

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

### feat: renamed canisters in new projects to `<project>_frontend` and `<project>_backend`

The names of canisters created for new projects have changed.
After `dfx new <project>`, the canister names are:

- `<project>_backend` (previously `<project>`)
- `<project>_frontend` (previously `<project>_assets`)

### feat: Enable threshold ecdsa signature

### feat: new command: `dfx canister metadata <canister> <name>`

For example, to query a canister's candid service definition: `dfx canister metadata hello_backend candid:service`

### refactor: deprecate /_/candid internal webserver

The dfx internal webserver now only services the /_/candid endpoint.  This
is now deprecated.  If you were using this to query candid definitions, you
can instead use `dfx canister metadata`.

### refactor: optimize from ic-wasm

Optimize Rust canister Wasm module via ic-wasm library instead of ic-cdk-optimizer. A separate installation of ic-cdk-optimizer is no longer needed.

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

```
"bitcoin": {
  "enabled": true,
  "nodes": ["127.0.0.1:18444"],
  "log_level": "info" <------- users can now configure the log level
}
```

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

### fix: `dfx schema` does not require valid dfx.json

There is no real reason for `dfx schema` to not work when a broken dfx.json is in the current folder - this is actually a very common scenario when `dfx schema` gets used.

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

### feat: `dfx canister call --candid <path to candid file> ...`

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

### fix: `dfx canister update-settings <canister id>` works even if the canister id is not known to the project.

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

See [Upgrade compatibility](https://internetcomputer.org/docs/language-guide/compatibility) for more details.

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

### feat: `dfx deploy --mode=reinstall <canister>`

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

### feat!: `dfx canister create --controller <controller>` named parameter

Breaking change: The controller parameter for `dfx canister create` is now passed as a named parameter,
rather than optionally following the canister name.

Old: `dfx canister create [canister name] [controller]`
New: `dfx canister create --controller <controller> [canister name]`

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

Under dfx.json → `canisters` → `<canister_name>`, developers can add a "declarations" config. Options are:

* "output" → directory to place declarations for that canister | default is `src/declarations/<canister_name>`

* "bindings" → [] list of options, ("js", "ts", "did", "mo") | default is "js", "ts", "did"

* "env_override" → a string that will replace process.env.\{canister_name_uppercase\}_CANISTER_ID in the "src/dfx/assets/language_bindings/canister.js" template.

js declarations output

* index.js (generated from "src/dfx/assets/language_bindings/canister.js" template)

* `<canister_name>.did.js` - candid js binding output

ts declarations output

  * `<canister_name>.did.d.ts` - candid ts binding output

did declarations output

  * `<canister_name>.did` - candid did binding output

mo declarations output

  * `<canister_name>.mo` - candid mo binding output

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

* Support for the latest version of the \{IC\} specification and replica.

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

Added `dfx canister info` command to get certified canister information. Currently this information is limited to the controller of the canister and the SHA256 hash of its Wasm module. If there is no Wasm module installed, the hash will be None.

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

- feat: dfx now provides `CANISTER_ID_<canister_name>` environment variables for all canisters to "npm build" when building the frontend.

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
The User provides the ID of the canister onto which the wallet Wasm is deployed.

``` bash
dfx identity deploy-wallet --help
dfx-identity-deploy-wallet
Installs the wallet Wasm to the provided canister id

USAGE:
    dfx identity deploy-wallet <canister-id>

ARGS:
    <canister-id>    The ID of the canister where the wallet Wasm will be deployed

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
