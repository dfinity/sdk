# Overview

`dfx` is the SDK’s official CLI tool, which allows user to create,
deploy and manage projects with the Internet Computer. It is currently
compiled into a single static binary (per platform) which includes all
assets necessary for it to run (including, for example, other binaries
that it writes to disk).

This document covers how the `dfx` binary should be distributed and
versioned on the web. It covers how a user is expected to install the
`dfx` binary, bootstrap and use it for the first time.

Managing binaries and `dfx` versions locally on the user machine is
covered by &lt;&lt;./version-management.adoc&gt;&gt;.

# Installation

Multiple distributions channels can be used to get the `dfx` binary
installed on a user’s machine. The user could go to
<https://sdk.dfinity.org> to have a list of options available for them.

## Shell

The user launches their terminal to download and install the binary
locally. The user runs the following line (or similar) in the shell
(multiple variants could be made available for fish, powershell or
others):

    curl https://sdk.dfinity.org/install.sh | sh -

A sha256 of the `install.sh` script should be provided to allow the user
to verify its veracity.

This does not require root access, but might explain to the user how to
add the binary to their path variable.

## Direct Download

The user downloads the binary directly, with an URL scheme looking like
this:

<table>
<caption>URL Variables</caption>
<colgroup>
<col style="width: 50%" />
<col style="width: 50%" />
</colgroup>
<tbody>
<tr class="odd">
<td style="text-align: left;"><p>Variable</p></td>
<td style="text-align: left;"><p>Description</p></td>
</tr>
<tr class="even">
<td style="text-align: left;"><p><code>VERSION</code></p></td>
<td style="text-align: left;"><p>The version of DFX (e.g.
<code>1.2.3</code>. Can also be a tag such as <code>latest</code> or
<code>next</code>.</p></td>
</tr>
<tr class="odd">
<td style="text-align: left;"><p><code>V_VERSION</code></p></td>
<td style="text-align: left;"><p>The version of DFX. Cannot be
tagged.</p></td>
</tr>
<tr class="even">
<td style="text-align: left;"><p><code>PLATFORM</code></p></td>
<td style="text-align: left;"><p><code>linux</code> or
<code>macos</code></p></td>
</tr>
<tr class="odd">
<td style="text-align: left;"><p><code>ARCH</code></p></td>
<td style="text-align: left;"><p><code>x86_64</code> currently.</p></td>
</tr>
<tr class="even">
<td style="text-align: left;"><p><code>EXT</code></p></td>
<td style="text-align: left;"><p><code>gz</code> or
<code>zip</code>.</p></td>
</tr>
</tbody>
</table>

URL Variables

Example: <https://sdk.dfinity.org/dfx/1.2.0/linux-x86_64/dfx-1.2.0.gz>

The user can then install that binary wherever they please.

## Installer Download

**This section is out of scope for developer network.**

Various installers should be made available for different platforms. For
example, on macOS, a user could download a `pkg` file that installs
`dfx` directly for them.

This could be distributed at the same location as other versions,
replacing the `EXT` variable above with the extension of the installer
the user should be looking for.

## Package Manager

**This section is out of scope for developer network.**

It would be useful for the user to use the package manager they already
have.

For example, the following package managers should be supported:

1.  `brew`. On Mac, this is the most common package manager. No local
    compilation is necessary to use it, and it can be a binary
    distribution.

2.  `npm`. Most users working with frontends will have npm available,
    and it is available on almost every platform.

3.  `dpkg` and `apt-get`. Plus other linux package managers.

The actual names of the package managers officially supported is
currently to be determined (out of scope of this document).

## Github releases

The user could go to GitHub and download a release directly. This would
be very similar to a direct download but would be part of the github
releases.

## Using the Internet Computer

**This section is out of scope for developer network.**

Once the IC launches, the `dfx` binary could also be available through a
canister’s static asset. The user could then use a special command to
download and install the binary on their machine, from the IC directly.

# Distribution

This section details how to build and deploy the various distributions.

## Release Tagging

Tagging a release (on `git`) using the exact tag format
`v{MAJOR}.{MINOR}.{PATCH}[-{PRERELEASE}]` would lead CI to build a
release build of `dfx`, and run the e2e tests on it. After doing that,
the CI process would upload it to some cloud storage to be made
available directly through direct download.

Once the binaries are available for users to download, nothing needs to
be done. All package managers should always verify and download the
latest version.

## Package Version Tagging

**This section is out of scope for developer network.**

In addition to direct version numbers, the user can use the following
tags to download a version of `dfx`:

1.  `latest`. The latest stable version of `dfx`.

2.  `next`. An unstable beta of the next version of `dfx`.

These tags should be available on package managers that support them
(e.g. `npm install dfx@next`).

## Package Version Listing

**This section is out of scope for developer network.**

Using the URL `https://sdk.dfinity.org/dfx/index.{html,json}` should
list all available packages, in either an HTML human pleasant format, or
a json machine readable one.

The JSON schema could look like this:

    {
      "tags": {
        "latest": "1.2.3",
        "next": "2.0.0-beta.1"
      },
      "versions": [
        "1.0.0",
        "1.0.1",
        "1.0.2",
        "1.1.0",
        "1.2.0",
        "1.2.1",
        "1.2.2",
        "1.2.3",
        "2.0.0-beta.0",
        "2.0.0-beta.1"
      ]
    }

# TO BE DETERMINED

What remains to be done prior to the final 1.0.0 release:

1.  Find a proper name for the package managers namespace. Hopefully
    something unique to all managers so users can use the same name
    (e.g. `npm install @dfinity/dfx` and
    `brew install @internet-computer/dfx`).

2.  Figuring out which package managers on linux we want to support and
    how to support them.

3.  Lay out the plan for using a canister for distributing `dfx`.

4.  Figure out if we want to do LTS for some versions.

5.  Finish out of scope sections in this document.
