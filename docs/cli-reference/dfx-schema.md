# dfx schema

The `dfx schema` command prints the schema for `dfx.json`.

## Basic usage

``` bash
dfx schema [option] [flag]
```

## Flags

You can use the following optional flags with the `dfx schema` command.

| Flag              | Description |
|-------------------|-------------|
| `-h`, `--help`    | Displays usage information. |
| `-V`, `--version` | Displays version information. |

## Options

You can use the following option with the `dfx schema` command.

| Option                 | Description                                                                                                       |
|------------------------|-------------------------------------------------------------------------------------------------------------------|
| `--for <dfx/networks>` | Display schema for either dfx.json or networks.json. (default: dfx) |
| `--outfile <outfile>`  | Specifies a file to output the schema to instead of printing it to stdout. |

## Examples

You can print the schema for `dfx.json` by running the following command:

``` bash
dfx schema
```

You can print the schema for `networks.json` by running the following command:

``` bash
dfx schema --for networks
```

If you want to write the schema for dfx.json to `path/to/file/schema.json`, you can do so by running the following command:

``` bash
dfx schema --outfile path/to/file/schema.json
```
