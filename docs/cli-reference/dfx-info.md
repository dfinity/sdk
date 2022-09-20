# dfx info

## Basic usage

``` bash
dfx info [type] [flag]
```

## Information Types

These are the types of information that the `dfx info` command can display.

| Option              | Description                                    |
|---------------------|------------------------------------------------|
| networks-json-path  | Path to network definition file networks.json. |
| replica-rev         | The revision of the bundled replica.           |
| webserver-port      | The local webserver (icx-proxy) port.          |

## Flags

You can use the following optional flags with the `dfx info` command.

| Flag              | Description |
|-------------------|-------------|
| `-h`, `--help`    | Displays usage information. |

## Examples

You can display the webserver port by running the following command:

``` bash
$ dfx info webserver-port
4943
```
