# `icx-asset`
A command line tool to manage an asset storage canister.

## icx-asset sync

Synchronize one or more directories to an asset canister.

Usage: `icx-asset sync <canister id> <source directory>...`

Example:
```
# same asset synchronization as dfx deploy for a default project, if you've already run dfx build
$ icx-asset --pem ~/.config/dfx/identity/default/identity.pem sync <canister id> src/prj_assets/assets dist/prj_assets  
```

## icx-asset ls

List assets in the asset canister.

## icx-asset upload

Usage: `icx-asset upload [<key>=]<file> [[<key>=]<file> ...]`

Examples:

```
# upload a single file as /a.txt
$ icx-asset upload a.txt

# upload a single file, a.txt, under another name
$ icx-asset upload /b.txt=a.txt

# upload a directory and its contents as /some-dir/*
$ icx-asset upload some-dir

# Similar to synchronization with dfx deploy, but without deleting anything:
$ icx-asset upload /=src/<project>/assets


```