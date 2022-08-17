# dfx bootstrap {#_dfx_bootstrap}

Use the `dfx bootstrap` command to start the bootstrap web server defined in the `dfx.json` configuration file or specified using command-line options.

The bootstrap web server you specify is used to serve the front-end static assets for your project.

## Basic usage {#_basic_usage}

``` bash
dfx bootstrap [option]
```

## Flags {#_flags}

You can use the following optional flags with the `dfx bootstrap` command.

| Flag                 | Description                                   |
-----------------------|-----------------------------------------------|
| `-h`, `--help`       | Displays usage information.                   |
| `-V`, `--version`    | Displays version information.                 |

## Options {#_options}

You can specify the following options for the `dfx bootstrap` command.

| Option               | Description     |
-----------------------|-----------------|
| `ip` <ip_address\>    | Specifies the IP address that the bootstrap server listens on. If you don't specify an IP address, the `address` setting you have configured in the `dfx.json` configuration file is used. By default, the server address is 127.0.0.1. |
| \--network <network\> | Specifies the network to connect to if you want to override the default local network endpoint (`http://127.0.0.1:8080/api`).|
| \--port <port\>       | Specifies the port number that the bootstrap server listens on. By default, port number 8081 is used.                                                                                                                                   |
| \--timeout <timeout\> | Specifies the maximum amount of time, in seconds, the bootstrap server will wait for upstream requests to complete. By default, the bootstrap server waits for a maximum of 30 seconds.                                                 |

## Examples {#_examples}

You can use the `dfx bootstrap` command to start a web server for your application using custom settings, including a specific server address, port number, and static asset location.

For example, to start the bootstrap server using a specific IP address and port number, you would run a command similar to the following:

``` bash
dfx bootstrap --ip 192.168.47.1 --port 5353
```

The command displays output similar to the following:

``` bash
binding to: V4(192.168.47.1:5353)
replica(s): \http://127.0.0.1:8080/api
Webserver started...
```

To use the default server address and port number but specify a custom location for static assets and longer timeout period, you might run a command similar to the following:

``` bash
dfx bootstrap --root $HOME/ic-projects/assets --timeout 60
```

You can use CTRL-C to stop the bootstrap server.
