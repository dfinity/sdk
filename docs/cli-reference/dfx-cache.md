# dfx cache

Use the `dfx cache` command with flags and subcommands to manage the `dfx` version cache.

The basic syntax for running `dfx cache` commands is:

``` bash
dfx cache [subcommand] [flag]
```

Depending on the `dfx cache` subcommand you specify, additional arguments, options, and flags might apply. For reference information and examples that illustrate using `dfx cache` commands, select an appropriate command.

| Command                    | Description                                                                   |
|----------------------------|-------------------------------------------------------------------------------|
| [`delete`](#delete)        | Deletes the specified version of `dfx` from the local cache.                  |
| `help`                     | Displays usage information message for a specified subcommand.                |
| [`install`](#install)      | Installs the specified version of `dfx` from the local cache.                 |
| [`list`](#_dfx_cache_list) | Lists the versions of `dfx` currently installed and used in current projects. |
| [`show`](#_dfx_cache_show) | Show the path of the cache used by this version of the `dfx` executable.      |

To view usage information for a specific subcommand, specify the subcommand and the `--help` flag. For example, to see usage information for `dfx cache delete`, you can run the following command:

``` bash
dfx cache delete --help
```

## dfx cache delete

Use the `dfx cache delete` command to delete a specified version of `dfx` from the version cache on the local computer.

### Basic usage

``` bash
dfx cache delete [version] [flag]
```

### Flags

You can use the following optional flags with the `dfx cache delete` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Arguments

You can specify the following argument for the `dfx cache delete` command.

| Command   | Description                                                        |
|-----------|--------------------------------------------------------------------|
| `version` | Specifies the version of `dfx` you to delete from the local cache. |

### Examples

You can use the `dfx cache delete` command to permanently delete versions of `dfx` that you no longer want to use. For example, you can run the following command to delete `dfx` version `0.6.2`:

``` bash
dfx cache delete 0.6.2
```

## dfx cache install

Use the `dfx cache install` command to install `dfx` using the version currently found in the `dfx` cache.

### Basic usage

``` bash
dfx cache install [flag]
```

### Flags

You can use the following optional flags with the `dfx cache install` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Examples

You can use the `dfx cache install` command to force the installation of `dfx` from the version in the cache. For example, you can run the following command to install `dfx`:

``` bash
dfx cache install
```

## dfx cache list

Use the `dfx cache list` command to list the `dfx` versions you have currently installed and used in projects.

If you have multiple versions of `dfx` installed, the cache list displays an asterisk (\*) to indicate the currently active version.

### Basic usage

``` bash
dfx cache list [flag]
```

### Flags

You can use the following optional flags with the `dfx cache list` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Examples

You can use the `dfx cache list` command to list the `dfx` versions you have currently installed and used in projects. For example, you can run the following command to list versions of `dfx` found in the cache:

``` bash
dfx cache list
```

This command displays the list of `dfx` versions found similar to the following:

``` bash
0.6.4 *
0.6.3
0.6.0
```

## dfx cache show

Use the `dfx cache show` command to display the full path to the cache used by the `dfx` version you are currently using.

### Basic usage

``` bash
dfx cache show [flag]
```

### Flags

You can use the following optional flags with the `dfx cache show` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

### Examples

You can use the `dfx cache show` command to display the path to the cache used by the `dfx` version you are currently using:

``` bash
dfx cache show
```

This command displays the path to the cache used by the `dfx` version you are currently using:

``` bash
/Users/pubs/.cache/dfinity/versions/0.6.4
```
