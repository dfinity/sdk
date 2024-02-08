# dfx upgrade

> :warning: `dfx upgrade` will be disabled once the [dfx version manager][dfxvm] is released.

Use the `dfx upgrade` command to upgrade the SDK components running on your local computer. This command checks the version of the SDK that you have currently installed against the latest publicly-available version specified in the `manifest.json` file. If an older version of the SDK is detected locally, the `dfx upgrade` command automatically fetches the latest version from the CDN.

## Basic usage

``` bash
dfx upgrade [flag] [option]
```

## Options

You can use the following option with the `dfx upgrade` command.

| Option                        | Description                                                                                                                                                                                                                  |
|-------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--current-version <version>` | Specifies the version you want to identify as the current version. This option enables you to override the version of the software currently identified as the latest version with the version you pass on the command-line. |

## Examples

You can upgrade the version of the SDK that you have currently installed by running the following command:

``` bash
dfx upgrade
```

This command checks the version of `dfx` you have currently installed and the latest version available published on the SDK website in a manifest file. If a newer version of `dfx` is available, the command automatically downloads and installs the latest version.

``` bash
Current version: 0.6.8
Fetching manifest \https://sdk.dfinity.org/manifest.json
Already up to date
```

[dfxvm]: https://github.com/dfinity/dfxvm
