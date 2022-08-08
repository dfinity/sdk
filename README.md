# DFX

## Getting Started

`dfx` is the command-line interface for managing your Internet Computer project and the best place to start.

### Installing

You can install `dfx` a few different ways.

#### via `curl` (recommended)

``` bash
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"
```

This command will install a binary compatible with your operating system, and add it to `/usr/local/bin`.

#### via GitHub Releases

Find a release for your architecture [here](https://github.com/dfinity/sdk/releases).

### Getting Help

Once `dfx` is installed, get acquainted with its capabilities by entering.

``` bash
dfx help
```

## Contributing to the DFINITY SDK

See our contributing guidelines [here](.github/CONTRIBUTING.md).

### Building DFX

Building `dfx` is very simple:

``` bash
cargo build
```

## Release Process

DFX is released in two steps:

1. Publishing a new DFX release.

2. Publishing a new `manifest.json` and `install.sh` to instruct the installer
   to actually download and install the new DFX release.

### Publishing DFX

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

After the DFX has been released it's available for download but the install
script at https://sdk.dfinity.org/install.sh won't immediately install it. To
make sure the installer actually downloads and installs the new DFX release the
`manifest.json` file at https://sdk.dfinity.org/manifest.json has to set its
`tags.latest` field to the new version. The following explains how to do that.

1. Edit the `public/manifest.json` file such that it points to the new DFX
   version and make sure this is merged in `master`.

2. Similarly to releasing the DFX there's a
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
This section provides solutions to problems you might encounter when using `dfx`

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
