# dfx generate

Use the `dfx generate` command to generate canister type declarations for supported programming languages. Currently, `dfx generate` supports four languages: Motoko, Candid, JavaScript, and TypeScript.

You can use this command to generate type declarations for all canisters that are defined for a project in the project’s `dfx.json` configuration file or a specific canister.

Note that you can only run this command from within the project directory structure. For example, if your project name is `hello_world`, your current working directory must be the `hello_world` top-level project directory or one of its subdirectories.

The `dfx generate` command looks for the configuration under the `declarations` section of a canister in the `dfx.json` configuration file.

## Basic usage

``` bash
dfx generate [canister_name]
```

## Flags

You can use the following optional flags with the `dfx generate` command.

| Flag              | Description                   |
|-------------------|-------------------------------|
| `-h`, `--help`    | Displays usage information.   |
| `-V`, `--version` | Displays version information. |

## Arguments

You can specify the following arguments for the `dfx generate` command.

| Argument        | Description                                                                                                                                                                                                                                                                                                                                                        |
|-----------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `canister_name` | Specifies the name of the canister for which to generate type declarations. The canister name must match at least one name that you have configured in the `canisters` section of the `dfx.json` configuration file for your project. If you don’t specify this argument, `dfx generate` will generate type declarations for all canisters declared in `dfx.json`. |

## Configuration

The behavior of `dfx generate` is controlled by the `dfx.json` configuration file. Under `dfx.json` → `canisters` → `<canister_name>`, you can add a `declarations` section. In this section, you can specify the following fields:

| Field          | Description                                                                                                                                  |
|----------------|----------------------------------------------------------------------------------------------------------------------------------------------|
| `output`       | Directory to place declarations for the canister. Default is `src/declarations/<canister_name>`.                                             |
| `bindings`     | List of languages to generate type declarations. Options are `"js", "ts", "did", "mo"`. Default is `["js", "ts", "did"]`.                    |
| `env_override` | String that will replace `process.env.{canister_name_uppercase}_CANISTER_ID` in the `src/dfx/assets/language_bindings/canister.js` template. |

Outputs from `dfx generate`:

| Language         | File                                    |
|------------------|-----------------------------------------|
| `JavaScript(js)` | `index.js` and `<canister_name>.did.js` |
| `TypeScript(ts)` | `<canister_name>.did.ts`                |
| `Candid(did)`    | `<canister_name>.did`                   |
| `Motoko(mo)`     | `<canister_name>.mo`                    |

## Examples

[This](../_attachments/sample-generate-dfx.json) is a sample output of `dfx generate`.

Note that the file name and path to the programs on your file system must match the information specified in the `dfx.json` configuration file.

In this example, the `hello_world` canister itself is written in Motoko. The `declarations` section specifies that type declarations for all four languages will be generated and stored at `src/declarations/`.

``` bash
dfx generate hello_world
```

Since there is only one canister in `dfx.json`, calling `dfx generate` without an argument will have the same effect as the above command. If there were multiple canisters defined in `dfx.json`, this would generate type declarations for all defined canisters.

``` bash
dfx generate
```
