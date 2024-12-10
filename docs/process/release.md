# IC SDK Release Process

## Stage 1: Preparation - day 1

### Update the Replica Version

Click the "Run workflow" button on the [Update Replica page](https://github.com/dfinity/sdk/actions/workflows/update-replica-version.yml) workflow.

Obtain approval and merge the PR.

### Update the changelog

Open a PR to `master`. Roll the changelog by adding a new header for the
new dfx version underneath the "# Unreleased" header.  Further changes to dfx
should be added under the "#Unreleased" header, unless they are ported to
the release branch.

[Sample PR](https://github.com/dfinity/sdk/pull/3486)

### Create the Release Branch

Create a release branch from `master`, for example `release-0.15.3`.

If you create a new patch version make sure there will be no breaking changes included.

This branch will be used to create beta releases as well as the final release.

## Stage 2: Beta releases - day 1 ~ 2

1. Check out the release branch;
1. Run the release script, for example `./scripts/release.sh 0.15.3-beta.0`.
   It will:
    - Build `dfx` from clean;
    - Validate the default project interactively;
    - Create a beta release branch which update the version number in manifest files and push to GitHub;
    - Wait for you to:
        - Create a PR from the new branch to the release branch,
      e.g. into `release-0.15.3` from `adam/release-0.15.3-beta.0`;
        - Obtain approval and merge the PR;
    - Push a tag which triggers the [publish][publish-workflow] workflow;
1. Wait for the [publish][publish-workflow] workflow to create the GitHub release;
1. Update the GitHub release:
    - Copy/paste the changelog section for the new version into the release notes;
    - Make sure that the "Pre-release" flag **is** set and the "Latest" flag is **NOT** set;
1. Announce the release to #eng-sdk with a message like this:
    > dfx 0.15.3-beta.1 is available for manual installation and testing.
    >
    > ```bash
    > dfxvm install 0.15.3-beta.1
    > ```
    >
    > See also release notes.
1. Post a message to the forum about availability of the not-yet-promoted beta, linking to the GitHub release notes.

[Sample PR](https://github.com/dfinity/sdk/pull/3477)

[publish-workflow]: https://github.com/dfinity/sdk/blob/master/.github/workflows/publish.yml

## Stage 3: Final Release - day 3

Once the beta releases are ready to be promoted:

1. Check out the release branch;
1. Run the release script, for example `./scripts/release.sh 0.15.3`;
1. Follow the same steps as for the beta releases;

[Sample PR](https://github.com/dfinity/sdk/pull/3490)

## Stage 4: Draft PRs to prepare for promotion - day 3

All following PRs should be created as "draft".

Obtain approval, but do not merge them yet.

### Promote the release in [sdk](https://github.com/dfinity/sdk)

1. Create a new branch from the release branch, for example `release-0.15.3-promote`;
1. Update the [version manifest](https://github.com/dfinity/sdk/blob/master/public/manifest.json):
    - Set `.tags.latest` to the new dfx version;
    - Remove the beta releases from the `versions` array;
1. Open a PR from this branch to `master`;

[Sample PR](https://github.com/dfinity/sdk/pull/3491)

### Update the [portal](https://github.com/dfinity/portal) release notes and sdk submodule

- Add a link to the [release-notes-table](https://github.com/dfinity/portal/blob/master/docs/other/updates/release-notes/release-notes.md);
    - Also include the link of the migration guide if it is available;
- Update the sdk submodule:
    1. Change to the sdk submodule: `cd submodules/sdk`
    1. Checkout the release branch, e.g. `git checkout release-0.18.0`
    1. Go back to project root and commit the submodule change.

[Sample PR](https://github.com/dfinity/portal/pull/2330)

### Update the [motoko-playground][motoko-playground] frontend canister hash whitelist

- Click the "Run workflow" button on the [Broadcast Frontend Hash page](https://github.com/dfinity/sdk/actions/workflows/broadcast-frontend-hash.yml).
- Fill "Release version of dfx" with the version of this release.
- The workflow will create a PR in the [motoko-playground][motoko-playground] repo.

[Sample PR](https://github.com/dfinity/motoko-playground/pull/217)

[motoko-playground]: https://github.com/dfinity/motoko-playground

### Update the [examples](https://github.com/dfinity/examples) default dfx

Modify `DFX_VERSION` in these two files:

- [provision-darwin.sh](https://github.com/dfinity/examples/blob/master/.github/workflows/provision-darwin.sh)
- [provision-linux.sh](https://github.com/dfinity/examples/blob/master/.github/workflows/provision-linux.sh)

[Sample PR](https://github.com/dfinity/examples/pull/704)

## Stage 5: Promote the release - day 4

* Precondition: Make sure `dfx deploy --playground` works with a project created by `dfx new`. This makes sure that the asset canister wasm is properly allowlisted in the playground backend.

### Update the GitHub release

- Unset the "Pre-release" flag
- Set the "Latest" flag

### Merge PRs

Merge all 4 PRs created in the previous stage:

- Promote the release
- Update the portal
- Update the motoko-playground
- Update the examples

### Post to the forum

Post a message to the forum, linking to the GitHub release notes.

[Sample Post](https://forum.dfinity.org/t/dfx-0-17-0-is-promoted)
