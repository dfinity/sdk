— DRAFT

# Overview

In order to use the SDK, the user has to download a version of `dfx` for
their platform. Normally that means installing the latest version and
using it, but the end user might be working on a project that was using
an older version of `dfx`.

For example, a user could have created a project with `dfx` v0.1, but
the latest is v1.0 and the user is not ready to upgrade their project
just yet.

This document details the design and decisions made to solve the problem
of having multiple `dfx` versions, and how upstream dependencies
integrate with `dfx`.

# User-facing Version

The version of `dfx` is the only version the user should be aware of.
Any upstream version will be hidden behind the `dfx` version.

For example, assuming `v1.1.5` and `v1.2.3` of `dfx` are supported
stable versions, with unstable being `v1.3.0-beta.1`, and assuming a
version of `dfx` could be released with bug fixes that affect both the
previous stable, current stable, and unstable releases, without changes
to `dfx` itself we should release a `v1.1.6`, `v1.2.4` and
`v1.3.0-beta.2` versions of `dfx` containing `dfx` with the fix
included.

This means the user would never actually see the version of `dfx` that
they are running.

# Installing `dfx`

The user should initially download the SDK by using the following
command:

    curl https://sdk.dfinity.org/install.sh | sh

This will install a binary compatible with your operating system, and
add it to `/usr/local/bin`(or equivalent).[1]

This binary contains a number of things:

1.  The global CLI. This is a CLI made to manage the cache, read the
    project file if necessary, and forward any calls to a versioned CLI.

2.  A tar gz release file containing a distribution versioned with the
    latest (available) version of the SDK. This tar file includes the
    versioned CLI, the Motoko compiler, the node manager and client, and
    any other necessary binaries.

# Version Resolution

The `dfx` global CLI then tries to find the version it should use (in
order);

1.  If a `DFX_VERSION` environment variable is set, use that version
    directly. If that version is not available in the cache, report the
    error and do not continue.

2.  If there is a local `dfx` configuration file, it will use the
    version of the SDK specified in it (the `dfx_version` field). If
    that version is not available in the cache, it will try to download
    it from the Internet. If there is no connection, report the error
    and do not continue.

3.  If there is an Internet connection, to a maximum of once a week,
    `dfx` will reach out to `sdk.dfinity.org` to find the latest version
    available.

4.  If there is a user-level cache available, `dfx` will use the latest
    version already downloaded by the user that is not newer than the
    `dfx` global CLI version.

5.  Finally, the global `dfx` comes versioned and will use that version
    number as a last resort. If that version is not part of the cache,
    it will use its internal tar file to bootstrap the user level cache.

The global CLI then defers the call to a local, user-level cached,
versioned CLI.

This whole process implies that:

1.  The CLI will always create a new project using the latest available
    client and SDK.

2.  The CLI will still work without an Internet connection.

3.  The global CLI only needs to be updated on major changes; either the
    `cache` command changes, or a URL needs to be updated. If a new
    version needs to be downloaded we can also tell the user how to
    perform the upgrade.

# Upgrade Subcommand

An `upgrade` command can be made available to ping our servers, download
the latest release, install it in the user-level cache, and update the
project’s `dfx_version` field (if in a project) to the new version. If
no Internet is available at that time it should error out.

A migration script should also be included between two versions to
upgrade the project. This is out of scope for version 1.0.

# Versions

## Operators

Semantic versioning operators should be supported in the `dfx_version`
field. For example, using the following configuration file should allow
the user to use any `dfx` version between 1.0.0 and less than 1.4:

    {
      "dfx_version": ">=1.0.0 <1.4.0"
    }

## Directories

Having a directory (starting with either `.`, `~` or `/`) in the
`dfx_version` field should be allowed to let the user (and most
importantly our own integration tests) use a custom version of the
installed binaries. No verification other than having the versioned CLI
in it should be necessary.

## Tags (*optional*)

We might want to have tracks of software that the user can use,
resolving to versions through the server. The following tags could be
made available from the start:

1.  `latest`. The latest stable version.

2.  `unstable`. The next alpha, beta or RC version.

3.  `lts`. The last Long Term Supported version, if such a version
    exists.

By default, prior to creating a project, the `latest` field will be used
when contacting the servers to gather the latest stable version.

# Remote Commands

Commands relating to wallet, key management, upgrade and deployment of
canisters, and other commands that affect a remote network (either the
main network or a hosted test network) should validate that the version
of the SDK is compatible with the version of the network being used
remotely.

A few ways of doing this can be considered:

1.  The HTTP API has a call to get the version, and we consider any
    delta X to be incompatible. 2 major versions is normally a good
    delta if we implement a 1 major version deprecation policy, but 1
    major version could also work. This implies that the client and SDK
    are loosely versioned together.

2.  The HTTP API has a call that lists all versions of the SDK it is
    backward compatible with.

3.  The HTTP API stays backward compatible forever.

There could be other schemes that work. This is out of scope for this
particular proposal, but should be addressed prior to launching the main
network.

# URL Scheme

`sdk.dfinity.org` should have a well-defined URL scheme that will avoid
regressions:

<table>
<caption>URL Schemes</caption>
<colgroup>
<col style="width: 50%" />
<col style="width: 50%" />
</colgroup>
<tbody>
<tr class="odd">
<td style="text-align: left;"><p>URL</p></td>
<td style="text-align: left;"><p>Description</p></td>
</tr>
<tr class="even">
<td
style="text-align: left;"><p><code>sdk.dfinity.org/install.{sh,bash,fish,bat,...}</code></p></td>
<td style="text-align: left;"><p>should return a shell script that
installs the global <code>dfx</code> CLI according to platform and shell
environment.</p></td>
</tr>
<tr class="odd">
<td
style="text-align: left;"><p><code>sdk.dfinity.org/v/</code></p></td>
<td style="text-align: left;"><p>Root of all the versions. The
<code>index.html</code> should list all available versions.</p></td>
</tr>
<tr class="even">
<td
style="text-align: left;"><p><code>sdk.dfinity.org/v/1.2.3/x86_64-darwin.tgz</code></p></td>
<td style="text-align: left;"><p>The release for version 1.2.3 for
OSX.</p></td>
</tr>
<tr class="odd">
<td
style="text-align: left;"><p><code>sdk.dfinity.org/tags/</code></p></td>
<td style="text-align: left;"><p>Root of all tags released.</p></td>
</tr>
<tr class="even">
<td
style="text-align: left;"><p><code>sdk.dfinity.org/tags/latest/manifest.json</code></p></td>
<td style="text-align: left;"><p>The manifest file containing the
version number and any flags necessary to get the version currently
tagged latest.</p></td>
</tr>
</tbody>
</table>

URL Schemes

# Cache

A cache directory will exist on the user’s home folder. On Linux and
OSX, it will likely be in `$HOME/.cache/dfinity`, while on Windows would
likely be in `C:\Users\$USER\AppData\Local\DFINITY`.

That cache folder should contain `./v/$VERSION/` folders for each
version downloaded.

## Upkeep

A `cache` subcommand should be available to users to manage their cache.
Example of subcommands:

    dfx cache clear  # Delete the cache folder entirely.
    dfx cache list  # List all version installed.
    dfx cache install 1.2.3  # Download and install version 1.2.3 in the cache
    dfx cache delete 1.2.3  # Delete all the cache elements for version 1.2.3

Because of the delegation between the global and versioned CLI, the
`cache` subcommand should be defined in the global CLI.

[1] Other systems, such as `brew`, `dpkg` or simply downloading a binary
directly, should be made available.
