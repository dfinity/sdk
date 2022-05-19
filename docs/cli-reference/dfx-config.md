# dfx config

Use the `dfx config` command to view or configure settings in the configuration file for a current project. Note that you can only run this command from within the project directory structure. For example, if your project name is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its subdirectories.

## Basic usage

``` bash
dfx config [config_path] [value] [flag]
```

## Flags

You can use the following optional flags with the `dfx config` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

## Options

You can use the following option with the `dfx config` command.

| Option     | Description                                                                                                                                         |
|------------|-----------------------------------------------------------------------------------------------------------------------------------------------------|
| `--format` | Specifies the format of the configuration file output. By default, the file is displayed using JSON format. The valid values are `json` and `text`. |

## Arguments

You can specify the following arguments for the `dfx config` command.

| Argument      | Description                                                                                                                                                                                                                                                                                                                                     |
|---------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `config_path` | Specifies the name of the configuration option that you want to set or read. You must specify the configuration file option using its period-delineated path to set or read the specific option you want to change or view. If you don’t specify the path to a specific configuration option, the command displays the full configuration file. |
| `value`       | Specifies the new value for the option you are changing. If you don’t specify a value, the command returns the current value for the option from the configuration file.                                                                                                                                                                        |

## Examples

You can use the `dfx config` command to change configuration settings such as the location of the default output directory or the name of your main program file.

For example, to change the default build output directory from `canisters` to `staging`, you can run the following command:

``` bash
dfx config defaults.build.output "staging/"
```

To view the current value for a configuration setting, you can specify the path to the setting in the configuration file without specifying a value. For example:

``` bash
dfx config defaults.build.output
```

The command returns the current value for the configuration option:

``` bash
"staging/"
```

Similarly, you can change the name of the main source file or the port number for the local canister execution environment by running commands similar to the following:

``` bash
dfx config canisters.hello.main "src/hello_world/hello-main.mo"
dfx config networks.local.bind 127.0.0.1:5050
```

You can also verify your configuration changes by viewing the `dfx.json` configuration file after running the `dfx config` command.
