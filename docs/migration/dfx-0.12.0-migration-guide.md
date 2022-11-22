# dfx 0.12.0 Migration Guide

This guide assumes you have installed dfx 0.12.0.

```bash
$ dfx --version
dfx 0.12.0
```

## Start the system-wide dfx server

Run this command from a directory that is not within any dfx project.  Run this once (not per-project).

```bash
[~] $ dfx start --background
Running dfx start for version 0.12.0
Using the default definition for the 'local' shared network because ~/.config/dfx/networks.json does not exist.
Dashboard: http://localhost:61605/_/dashboard
```

## Migrate each individual project

This section assumes you have a dfx project functioning with 0.11.2.  Follow these directions for each such project.

1. Change into the project's directory
```bash
$ cd ~/w/migration/0.11.2-to-0.12.0/basic
```

2. Remove the `dfx` version specification from dfx.json
If your project's dfx.json contains a `dfx` key, remove it.  It looks like this:
```json
{
  "dfx": "0.11.2"
}
```
Note that dfx will still respect the dfx version if you leave it in (or change it to `"0.12.0"`, but we recommend to remove it.

3. Stop any running dfx server
```bash
$ dfx stop
Using project-specific network 'local' defined in ~/w/migration/0.11.2-to-0.12.0/basic/dfx.json
WARN: Project-specific networks are deprecated and will be removed after February 2023.
```

4. Remove the `local` network, or the entire `networks` field, from dfx.json

If your project's dfx.json `networks` field contains only the `local` network, remove the `networks` key altogether.

```json
{
  "networks": {
    "local": {
      "bind": "127.0.0.1:8000",
      "type": "ephemeral"
    }
  }
}
```

5. Update the webserver port in `webpack.config.js`

Change port 8000 to port 4943 in this section of `webpack.config.js`.  When you're done, it should look like this:

```
  // proxy /api to port 4943 during development
  devServer: {
    proxy: {
      "/api": {
        target: "http://127.0.0.1:4943",
        changeOrigin: true,
        pathRewrite: {
          "^/api": "/api",
        },
      },
    },
```

dfx uses port `4943` by default for the shared local network in order to ensure that it does not accidentally connect to a project local network.
```bash
$ dfx info webserver-port
4943
```

6. Deploy your project

Notice that the Frontend canister URL will have changed:
```bash
$ dfx deploy
...
  Frontend canister via browser
    basic_frontend: http://127.0.0.1:4943/?canisterId=qoctq-giaaa-aaaaa-aaaea-cai
```

## Usage Changes

### Replace any usage of `dfx config`

dfx 0.12.0 removes the `dfx config` command. Please update Bash scripts to use `jq`, PowerShell scripts to use `ConvertTo-Json`, nushell scripts to use `to json`, etc.

### Use `dfx canister update-settings --set-controller` to set a canister's controller

When using `dfx canister update-settings`, it is easy to mistake `--controller` for `--add-controller`. For this reason `--controller` has been renamed to `--set-controller`.

### Use `dfx canister metadata` to retrieve the candid service definition

dfx used to provide the `/_/candid` endpoint to retrieve a canister's candid service definition. We've removed this endpoint. Please use `dfx canister metadata <canister> candid:service` instead.
