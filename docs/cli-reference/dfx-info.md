# dfx info

The `dfx info` command prints the schema for `dfx.json`.

## Basic usage

``` bash
dfx info [type] [flag]
```

## Flags

You can use the following optional flags with the `dfx schema` command.

| Flag              | Description |
|-------------------|-------------|
| `-h`, `--help`    | Displays usage information. |

## Information Types

You can use the following option with the `dfx schema` command.

| Option               | Description                                                                                                       |
|----------------------|-------------------------------------------------------------------------------------------------------------------|
| webserver-port       | Display schema for either dfx.json or networks.json. (default: dfx) |

## Examples

You can print the schema for `dfx.json` by running the following command:

``` bash
$ dfx info webserver-port
4943
```
