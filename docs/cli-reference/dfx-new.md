# dfx new

Use the `dfx new` command to create a new project for the IC. This command creates a default project structure with template files that you can modify to suit your dapp. You must specify the name of the project to you want to create.

You can use the `--dry-run` option to preview the directories and files to be created without adding them to the file system.

## Basic usage

``` bash
dfx new _project_name_ [flag]
```

## Flags

You can use the following optional flags with the `dfx new` command:

| Flag              | Description                                                                                                                                                                                                                                                                                                                                                                                                                         |
|-------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--dry-run`       | Generates a preview the directories and files to be created for a new project without adding them to the file system.                                                                                                                                                                                                                                                                                                               |
| `--frontend`      | Installs the template frontend code for the default project canister. The default value for the flag is `true` if `node.js` is currently installed on your local computer. If `node.js` is not currently installed, you can set this flag to `true` to attempt to install `node.js` and the template file when creating the project or you can set the flag to `false` to skip the installation of template frontend code entirely. |
| `-h`, `--help`    | Displays usage information.                                                                                                                                                                                                                                                                                                                                                                                                         |
| `-V`, `--version` | Displays version information.                                                                                                                                                                                                                                                                                                                                                                                                       |

## Arguments

You must specify the following argument for the `dfx new` command.

| Argument       | Description                                                             |
|----------------|-------------------------------------------------------------------------|
| `project_name` | Specifies the name of the project to create. This argument is required. |

## Examples

You can use `dfx new` to create a new project named `my_social_network` by running the following command:

``` bash
dfx new my_social_network
```

The command creates a new project, including a default project directory structure under the new project name and a Git repository for your project.

If you want to preview the directories and files to be created without adding them to the file system, you can run the following command:

``` bash
dfx new my_social_network --dry-run
```
