# IC SDK Release Process

## Overview

1. Stage 1: Preparation (day 1)
  1. Update the replica version
  1. Update the changelog on master
  1. Create the release branch
1. Stage 2: Beta releases (day 1~2)
1. Stage 3: Final release (day 3)
1. Stage 4: Create draft PRs to relevant repos (day 3)
   1. Portal
   1. Motoko Playground
   1. dfx-extensions
   1. 
1. Stage 5: Promotion (day 4)


1. Create beta releases
1. Open a PR to update the Portal
1. Create the final release
1. Open a PR to promote the release
1. Open a PR to update the Motoko Playground allow-list
1. Open a PR to update dfx-extension compatibility
1. Promote the release
1. Post to the forum
1. Open a PR to update the examples repo

## Details

### Update the Replica Version

Before making a new release, try to update the replica to the latest version
by running the [update-replica] workflow.

### Update the changelog

Open a PR to master.  Roll the changelog by adding a new header for the
new dfx version underneath the "# Unreleased" header.  Further changes to dfx
should be added under the "#Unreleased" header, unless they are ported to
the release branch.

### Create the Release Branch

Create a release branch from `master`, for example `release-0.15.3`. If you create a new patch version make sure there will be no breaking changes included.

This branch will be used to create beta releases as well as the final release.

### Create Beta Releases

1. Check out the release branch.
1. Run the release script, for example `./scripts/release.sh 0.15.3-beta.1`
1. Open a PR from the branch pushed by the previous to the release branch,
obtain CR approval, and merge the PR.
    - The release script will wait for you to do this
    - It will then create and push a tag
    - This triggers the [publish][publish-workflow] workflow
1. Wait for the [publish][publish-workflow] workflow to create the GitHub release
from the last commit on the release branch.
1. Update the GitHub release
    - Copy/paste the changelog section for the new version into the release notes
    - Make sure that the "Pre-release" flag **is** set and the "Latest" flag is **NOT** set.
1. Announce the release to #eng-sdk
    - Post a message like this, linking to the GitHub release notes:
        > dfx 0.15.3-beta.1 is available for manual installation and testing.
        >
        > ```bash
        > dfxvm install 0.15.3-beta.1
        > ```
        >
        > See also release notes.
1. Repeat the above steps until ready to promote the latest beta.

### Open a Draft PR to update the Portal

You can do this step while the beta releases are being tested.

- Add a link to the [release-notes-table]
- Update the sdk submodule

Obtain approval, but do not merge the PR yet.

This PR is a draft in order to help remind the reviewer not to merge it.

### Create the Final Release

Once the beta releases are ready to be promoted:

1. Check out the release branch
1. Run the release script, for example `./scripts/release.sh 0.15.3`
1. Follow the same steps as for the beta releases

### Open a PR to promote the release

1. Create a new branch from the release branch, for example `release-0.15.3-promote`.
1. Update the [version manifest][public-manifest]:
    - Set `.tags.latest` to the new dfx version
    - Remove the beta releases from the `versions` array
1. Open a PR from this branch to master

Obtain approval, but do not merge this PR yet.

### Open a PR to Update the Motoko Playground allow-list

You can do it either by using GitHub UI ([broadcast-frontend-hash-workflow])
or by running the following command:

```bash
gh workflow run "broadcast-frontend-hash.yml" -f dfx_version=<n.n.n>
```

Obtain approval, but do not merge this PR yet.

### Open a PR to update the dfx-extensions compatibility list

Change [this file](https://github.com/dfinity/dfx-extensions/blob/main/compatibility.json) so that the new version of dfx is compatible with the latest release of dfx extensions.

Obtain approval, but do not merge this PR yet.

### Promote the release

You should now have four open, approved PRs:

- Update the portal
- Promote the release
- Update the Motoko Playground allow-list
- Update the dfx-extensions compatibility list

Merge all four PRs.

### Post to the forum

Post a message to the forum, linking to the GitHub release notes.

### Open a PR to update the examples repo

Open a PR in the examples repo to update the dfx version used by default in the examples.
The PR should update DFX_VERSION in these two files:

- [provision-darwin.sh]
- [provision-linux.sh]

[broadcast-frontend-hash-workflow]: https://github.com/dfinity/sdk/actions/workflows/broadcast-frontend-hash.yml
[provision-darwin.sh]: https://github.com/dfinity/examples/blob/master/.github/workflows/provision-darwin.sh
[provision-linux.sh]: https://github.com/dfinity/examples/blob/master/.github/workflows/provision-linux.sh
[public-manifest]: https://github.com/dfinity/sdk/blob/master/public/manifest.json
[publish-workflow]: https://github.com/dfinity/sdk/blob/master/.github/workflows/publish.yml
[release-notes-table]: https://github.com/dfinity/portal/blob/master/docs/other/updates/release-notes/release-notes.md
[update-replica]: https://github.com/dfinity/sdk/actions/workflows/update-replica-version.yml
