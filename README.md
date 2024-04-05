# IC SDK

Testing PR's from a fork

This repo contains the `IC SDK`: a Software Development Kit for creating and managing [canister smart contracts on the Internet Computer (ICP blockchain)](https://wiki.internetcomputer.org/wiki/Canister_smart_contract).

For further reading:
* [Introduction to the ICP blockchain](https://wiki.internetcomputer.org/wiki/Introduction_to_ICP)
* [Internet Computer dashboard](https://dashboard.internetcomputer.org/)
* [Developer docs for ICP smart contracts](https://internetcomputer.org/docs/current/home)
* [Sample code of ICP smart contracts](https://internetcomputer.org/samples)
* [IC wiki](https://wiki.internetcomputer.org/wiki/Main_Page)

## What gets installed

The `IC SDK` installation script installs several components in default locations on your local computer. The following table describes the development environment components that the installation script installs:

| Component    | Description                                                                                        | Default location                              |
|--------------|----------------------------------------------------------------------------------------------------|-----------------------------------------------|
| dfx          | Command-line interface (CLI)                                                     | `/usr/local/bin/dfx`                          |
| moc          | Motoko runtime compiler                                                                            | `~/.cache/dfinity/versions/<VERSION>/moc`     |
| replica      | Internet Computer local network binary                                                             | `~/.cache/dfinity/versions/<VERSION>/replica` |
| uninstall.sh | Script to remove the SDK and all of its components                                    | `~/.cache/dfinity/uninstall.sh`               |
| versions     | Cache directory that contains a subdirectory for each version of the SDK you install. | `~/.cache/dfinity/versions`                   |

## SDK vs CDK vs `dfx`

There are a few components above worth expanding on:

1. **dfx** - `dfx` is the command-line interface for the `IC SDK`. This is why many commands for the IC SDK start with the command "`dfx ..`" such as `dfx new` or `dfx stop`.

2. **Canister Development Kit (CDK)** - A CDK is an adapter used by the IC SDK so a programming language has the features needed to create and manage canisters. 
The IC SDK comes with a few CDKs already installed for you so you can use them in the language of yoru choice. That is why there is a [Rust CDK](https://github.com/dfinity/cdk-rs), [Python CDK](https://demergent-labs.github.io/kybra/), 
[TypeScript CDK](https://demergent-labs.github.io/azle/), etc... Since CDKs are components used the SDK, some developer choose to use the CDK directly (without the `IC SDK`), 
but typically are used as part of the whole `IC SDK`.


## Getting Started

### Installing

You can install the `IC SDK` a few different ways.

#### via `curl` (recommended)

``` bash
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"
```

This command will install a binary compatible with your operating system, and add it to `/usr/local/bin`.

#### via GitHub Releases

Find a release for your architecture [here](https://github.com/dfinity/sdk/releases).

#### in GitHub Action, using [`dfinity/setup-dfx`](https://github.com/dfinity/setup-dfx)

```yml
    steps:
    - name: Install dfx
      uses: dfinity/setup-dfx@main
```

### Getting Help

Once the `IC SDK` is installed, get acquainted with its capabilities by entering.

``` bash
dfx help
```

## Contributing to the DFINITY SDK

See our contributing guidelines [here](./CONTRIBUTING.md).

### Building the IC SDK

Building the `IC SDK` is very simple:

``` bash
cargo build
```

## Release Process

`IC SDK` is released in two steps:

1. Publishing a new `IC SDK` release.

2. Publishing a new `manifest.json` and `install.sh` to instruct the installer
   to actually download and install the new `IC SDK` release.

### Publishing the IC SDK

1. The release manager makes sure the `dfx` `stable` branch points to the revision
   that should be released and that the revision is tagged with a version (like
   `0.5.6`).

2. The
   [`sdk-release`](https://hydra.dfinity.systems/jobset/dfinity-ci-build/sdk-release#tabs-configuration)
   jobset on Hydra tracks the `stable` branch and starts evaluating shortly
   after `stable` advances.

3. As you can see it only has the single job `publish.dfx` which is
   defined [here](https://github.com/dfinity-lab/sdk/blob/stable/ci/release.nix)
   in terms of the
   [`dfx`](https://github.com/dfinity-lab/sdk/blob/stable/publish.nix) job. Note
   that the `publish.dfx` job only exists when the revision has a
   proper version tag. This prevents publishing of untagged revisions.

4. Our CD system running at `deployer.dfinity.systems` is configured with the
   [`publish-sdk-dfx-release`](https://github.com/dfinity-lab/infra/blob/1fe63e06135be206d064a74461f739c4fafec3c7/services/nix/publish-sdk-release.nix#L39:L47)
   job. It will monitor the aforementioned `publish.dfx` job for
   new builds, whenever there's a new build it will download the output (the CD
   script) and execute it.

5. As you can see the script also sends a message to the `#build-notifications`
   Slack channel so you can see when and if the SDK has been published.

### Publishing `manifest.json` and `install.sh`

After the `IC SDK` has been released it's available for download but the install
script at https://sdk.dfinity.org/install.sh won't immediately install it. To
make sure the installer actually downloads and installs the new `IC SDK` release the
`manifest.json` file at https://sdk.dfinity.org/manifest.json has to set its
`tags.latest` field to the new version. The following explains how to do that.

1. Edit the `public/manifest.json` file such that it points to the new `IC SDK`
   version and make sure this is merged in `master`.

2. Similarly to releasing the `IC SDK` there's a
   [`install-sh`](https://github.com/dfinity-lab/sdk/blob/stable/publish.nix) job
   that builds a CD script for publishing the `manifest.json` and `install.sh`
   to our CDN.

3. This
   [job](https://hydra.dfinity.systems/job/dfinity-ci-build/sdk/publish.install-sh.x86_64-linux)
   is built on the `sdk` jobset which tracks the `master` branch.

4. `deployer.dfinity.systems` is configured with the
   [`publish-sdk-install-sh`](https://github.com/dfinity-lab/infra/blob/1fe63e06135be206d064a74461f739c4fafec3c7/services/nix/publish-sdk-release.nix#L48:L56)
   job which will monitor the aforementioned `publish.install-sh.x86_64-linux`
   job for new builds, whenever there's a new build it will download the output
   (the CD script) and execute it.


## Troubleshooting
This section provides solutions to problems you might encounter when using the `IC SDK` via `dfx` command line

### Project Reset

This command will remove the build directory and restart your replica:

``` bash
dfx stop && dfx start --clean --background
```

### Using Internet Identity Locally
You can deploy the Internet Identity canister into your replica alongside your project by cloning https://github.com/dfinity/internet-identity. From the `internet-identity` directory, run the following command:

``` bash
II_ENV=development dfx deploy --no-wallet --argument '(null)'
```

There are more notes at https://github.com/dfinity/internet-identity#running-locally that may be helpful.
