# dfx stop

Use the `dfx stop` command to stop the local canister execution environment processes that you currently have running on your computer. In most cases, you run the canister execution environment locally so that you can deploy canisters and test your dapps during development. To simulate the connection to the IC, these processes run continuously either in a terminal shell where you started them or the in the background until you stop or kill them.

Note that you can only run this command from within the project directory structure. For example, if your project name is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its subdirectories.

## Basic usage

``` bash
dfx stop [flag]
```

## Flags

You can use the following optional flags with the `dfx stop` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

## Examples

You can stop the local canister execution environment processes that are running in the background by changing to a project directory then running the following command:

``` bash
dfx stop
```

If the local canister execution environment is running in a current shell rather than in the background, open a new terminal shell, change to a project directory, then run the `dfx stop` command.

The current process identifier (`pid`) for the canister execution environment process started by `dfx` is recorded in the `.dfx/pid` file. You can view the process identifier before running the `dfx stop` command by running the following command:

``` bash
more .dfx/pid
```

This command displays a process identifier similar to the following:

``` bash
1896
```

If you are still having trouble with a persistent service running after attempting to stop, you can terminate all running jobs with:

``` bash
killall dfx replica
```
