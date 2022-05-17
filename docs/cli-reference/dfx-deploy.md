# dfx deploy

Use the `dfx deploy` command to register, build, and deploy a dapp on the local canister execution environment, on the IC or on a specified testnet. By default, all canisters defined in the project `dfx.json` configuration file are deployed.

This command simplifies the developer workflow by enabling you to run one command instead of running the following commands as separate steps:

    dfx canister create --all
    dfx build
    dfx canister install --all

Note that you can only run this command from within the project directory structure. For example, if your project name is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its subdirectories.

## Basic usage

``` bash
dfx deploy [flag] [option] [canister_name]
```

## Flags

You can use the following optional flags with the `dfx deploy` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

## Options

You can use the following options with the `dfx deploy` command.

| Option                             | Description                                                                                                                                                                 |
|------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--network <network>`              | Overrides the environment to connect to. By default, the local canister execution environment is used.                                                                      |
| `--argument <argument>`            | Specifies an argument using Candid syntax to pass to the canister during deployment. Note that this option requires you to define an actor class in the Motoko program. |
| `--with-cycles <number-of-cycles>` | Enables you to specify the initial number of cycles for a canister in a project.                                                                                            |

### Arguments

You can specify the following arguments for the `dfx deploy` command.

| Argument        | Description                                                                                                                                                                                                                                                                                                                                    |
|-----------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `canister_name` | Specifies the name of the canister you want to register, build, and deploy. Note that the canister name you specify must match at least one name in the `canisters` section of the `dfx.json` configuration file for the project. If you don’t specify a canister name, `dfx deploy` will deploy all canisters defined in the `dfx.json` file. |

## Examples

You can use the `dfx deploy` command to deploy all or specific canisters on the local canister execution environment, on the IC or on a specified testnet.

For example, to deploy the `hello` project on the hypothetical `ic-pubs` testnet configured in the `dfx.json` configuration file, you can run the following command:

``` bash
dfx deploy hello --network ic-pubs
```

To deploy a project on the local canister execution environment and pass a single argument to the installation step, you can run a command similar to the following:

``` bash
dfx deploy hello_actor_class --argument '("from DFINITY")'
```

Note that currently you must use an actor class in your Motoko dapp. In this example, the `dfx deploy` command specifies an argument to pass to the `hello_actor_class` canister. The main program for the `hello_actor_class` canister looks like this:

    actor class Greet(name: Text) {
        public query func greet() : async Text {
            return "Hello, " # name # "!";
        };
    };

You can use the `dfx deploy` command with the `--with-cycles` option to specify the initial balance of a canister created by your wallet. If you don’t specify a canister, the number of cycles you specify will be added to all canisters by default. To avoid this, specify a specific canister by name. For example, to add an initial balance of 8000000000000 cycles to a canister called "hello-assets", run the following command:

``` bash
dfx deploy --with-cycles 8000000000000 hello-assets
```
