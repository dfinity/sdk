# Contributing to DFX

## Developing `dfx`

Use `cargo` with the relevant [Rust toolchain](../rust-toolchain.toml) like any other Rust project.

``` bash
sdk $ cargo build
sdk $ cargo test
```

Then use `cargo run` or the method of your choice to run `dfx`. Dfx will display the version `dfx 0.9.1-63-gd7020bbb`,
even if you are compiling from the latest master.

``` bash
sdk $ alias dfx=$(pwd)/target/debug/dfx
sdk $ dfx --version
dfx 0.9.1-63-gd7020bbb
```

### End-to-End Tests

#### Setup

1. Install bats and bats-support 0.3.0. See the CI provisioning scripts for examples:
    - [Linux](../scripts/workflows/provision-linux.sh)
    - [Darwin](../scripts/workflows/provision-darwin.sh)
2. Export `BATSLIB` to your bats-support directory, typically:
    ``` bash
    $ export BATSLIB=/usr/local/lib/bats-support
    ```
3. Build dfx and add its target directory to your path:
    ``` bash
    sdk $ cargo build
    sdk $ export PATH="$(pwd)/target/debug:$PATH"
    ```
4. Install `jq`.

#### Running End-to-End Tests

``` bash
sdk $ bats e2e/tests-dfx/*.bash
sdk $ bats e2e/tests-replica/*.bash
```

#### Running End-to-End Tests Against Reference IC

This runs the end-to-end tests against the
[reference implementation of the Internet Computer](https://github.com/dfinity/ic-hs).

``` bash
sdk $ USE_IC_REF=1 bats e2e/tests-dfx/*.bash
sdk $ USE_IC_REF=1 bats e2e/tests-replica/*.bash
```

## Conventional Commits

We use a squash & merge PR strategy, which means that each PR will result in exactly
one commit in `master`. When releasing, we are using the history to know which commits
and what messages make into the release notes (and what needs to be documented).

That means we enforce conventional commits to help us distinguish those commits. When
creating a PR, we have a special check that validate the PR title and will fail if it
doesn't follow the conventional commit standard (see
https://www.conventionalcommits.org/).

What that means is your PR title should start with one of the following prefix:

- `feat:`. Your PR implement a new feature and should be documented. If version numbers
  were following semver, this would mean that we need to put the PR in the next minor.
- `fix:`. Your PR fixes an issue. There should be a link to the issue being fixed.
  In SemVer, this would be merged in both minor and patch branches.
- `refactor:`, `chore:`, `build:`, `docs:`, `test:` does not affect the release notes
  and will be ignored.
- `release:`. Your PR is for tagging a release and should be ignored, but will be
  a break point for the log history when doing release notes.

## Dependencies

### Updating the Replica

#### Using GitHub Action
. Head over to [the GitHub Action](https://github.com/dfinity/sdk/actions/workflows/update-replica-version.yml).
. Click "Run workflow" button, choose appropriate options (you're probably fine with using defaults), and click "Run workflow" (the green one). 
. Depending on the selected options, the workflow will run anything between 3 to 35 minutes. After that time, a new PR will be created.
. The PR contains the content that needs to be pasted into CHANGELOG.adoc, as well as the link for editing the CHANGELOG.adoc directly on the branch of that PR.
. After making changes to the CHANGELOG.adoc file, PR is ready for review.

#### Locally
To update the replica to a given $SHA from the dfinity repo, execute the following:
``` bash
# Requires niv to run. To install niv, run nix-env -iA nixpkgs.niv
./scripts/update-replica.sh $SHA
```

### Updating Motoko

To update Motoko to a given $VERSION from the motoko and motoko-base repos, execute the following:
``` bash
# Requires niv to run. To install niv, run nix-env -iA nixpkgs.niv
./scripts/update-motoko.sh $VERSION
# if you want to also update ledger canisters for e2e tests, use:
./scripts/update-motoko.sh $VERSION --update-nns
```

### Licenses

[Latest licenses of all dependencies of dfx (build for x86_64-linux)](https://hydra.oregon.dfinity.build/latest/dfinity-ci-build/sdk/licenses.dfx.x86_64-linux/licenses.dfinity-sdk-dfx.html).
