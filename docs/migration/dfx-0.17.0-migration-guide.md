# dfx 0.17.0 Migration Guide

## Use dfxvm rather than `dfx toolchain`

We've removed the `dfx toolchain` command.

Please use the [dfx version manager][dfxvm] to manage dfx versions instead.

## Deprecation of `dfx upgrade`

The `dfx upgrade` command is deprecated.  After the release of the
[dfx version manager][dfxvm], the `dfx upgrade` command will no longer be
available for use. Attempting to execute it after this release will
result in an error message, and the command will not be executed.

## Release tarball format

If you've been manually downloading release tarballs, please note that
we have begun to release tarballs in a new format and will stop releasing
the old format in the future.

The new format contains a top-level directory with the same name as the
basename of the tarball. The filenames no longer contain the version number.

Example links to tarballs in the new format:

- [dfx-x86_64-apple-darwin.tar.gz](https://github.com/dfinity/sdk/releases/download/0.16.1/dfx-x86_64-apple-darwin.tar.gz_)
- [dfx-x86_64-unknown-linux-gnu.tar.gz](https://github.com/dfinity/sdk/releases/download/0.16.1/dfx-x86_64-unknown-linux-gnu.tar.gz)

[dfxvm]: https://github.com/dfinity/dfxvm
