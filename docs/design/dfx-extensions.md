# DFX extensions - how it works 

`dfx` extension is a feature in the DFINITY Developer Software Development Kit (SDK) that allows for extending the `dfx` CLI's core functionality. Modeled after the principles of [Git custom commands](https://mfranc.com/tools/git-custom-command/) and [Cargo's custom subcommands](https://doc.rust-lang.org/book/ch14-05-extending-cargo.html#extending-cargo-with-custom-commands), the feature enables the addition of user-defined commands that seamlessly integrate with the existing `dfx` command set. 

## dfx extension install

The dfx utility offers a feature to install new extensions. Here's a high-level overview of how the installation process works:

1. Determine Extension Compatibility:
    - Before any installation can take place, dfx checks if the requested extension is already installed. If it is, the process terminates with an error.
    - The utility identifies the version of the extension compatible with its own version. This ensures that users don't end up installing extensions that may not work properly with their specific version of dfx.
    - Compatibility is determined using an extension compatibility matrix, which presumably maps extensions to compatible versions of dfx.
    - The compatibility matrix is a map that relates dfx versions to a set of extensions and their versions.
    - This matrix is fetched from a predefined URL. This means that dfx can always retrieve the most up-to-date compatibility information by querying this URL.
    - When checking for compatibility, dfx consults this matrix to determine the latest version of an extension that's compatible with the current dfx version.
    - This approach ensures that dfx only runs or interacts with extensions that are guaranteed to work correctly with its current version, preventing potential mismatches or runtime errors.
    - Given a dfx version and an extension name, the system checks the compatibility matrix.
    - If the given dfx version is not in the matrix, an error is raised.
    - If the extension name is not associated with the dfx version, another error is raised.
    - The versions of the extension that are compatible with the dfx version are then sorted, and the latest version is chosen as the most suitable one.
    - This mechanism ensures that users always get the newest version of an extension that works correctly with their dfx version.
    - In essence, while dfx provides tools to manage extensions, the compatibility matrix and extension manifest add an extra layer of sophistication. They ensure that extensions are accurately described, can be transformed into command-line interfaces, and most importantly, are always in sync with the version of dfx the user is operating. This comprehensive management system guarantees smooth operations and enhances user experience.

2. Download and Extraction:
    - Once the compatible version of the extension is determined, dfx constructs a download URL. This URL points to a GitHub releases page where the extensions are hosted. Currently, extensions support downloading only from `dfinity/dfx-extensions` repository. Here is how the URL template looks like:
    ```
    "https://github.com/dfinity/dfx-extensions/releases/download/{github_release_tag}/{extension_archive_name}.tar.gz";
    ```
    - The extension is then downloaded from this URL. If the download fails for any reason, an error is returned.
    - After successful download, the extension, which is in a compressed archive format (.tar.gz), is unpacked to a temporary directory.

3. Final Installation:
    - Post extraction, the extension is renamed and moved to its permanent location (the directory where extensions are meant to reside).
    - If the user specifies a custom name for the extension at the time of installation, the binary is renamed to reflect this custom name.
    - On UNIX systems, appropriate permissions are set for the installed extension to ensure it's executable.

4. Error Handling:
    - Throughout this process, any potential errors are diligently checked for, ensuring a robust installation process. Examples include errors related to incompatible versions, download failures, decompression errors, etc.
    - This process ensures that the extensions are not only installed from a trusted source but are also compatible with the user's dfx version, enhancing the utility's reliability and user experience.

## Running Extensions

Installed extensions are stored in `dfx`'s cache. Let's say you have `sns` extension installed; here is how the cache directory structure will look like:
```console
❯ tree $(dfx cache show)
~/.cache/dfinity/versions/0.15.0
└── extensions
   └── sns
      ├── CHANGELOG.md
      ├── extension.json
      ├── LICENSE
      ├── README.md
      ├── sns
      └── sns-cli
```

When requested to run an extension, dfx fetches the location of the executable associated with the extension, by determining the cache location associated with its current version, and searching for directory with extension name inside: `$(dfx cache show)/extensions/EXT_NAME/EXT_NAME`. Fox example , when you're running `dfx sns ARGS` or `dfx extension run sns ARGS`, `dfx` will try to find and extension under this path: `$(dfx cache show)/extensions/sns/sns`. 

When executing the binary, the path to the cache is appended as an argument when invoking the extension's executable, in practice that means the extension will always be executed with `--dfx-cache-path $(dfx cache show)` parameter, like so: `exec $(dfx cache show)/extensions/sns/sns --dfx-cache-path $(dfx cache show)`. This allows the extension to depend on other extensions, and binaries stored in dfx's cache.

The extension's binary is launched as a child process. The system waits for the child process to finish executing. Once the process completes, dfx checks the exit code. If it's non-zero (indicating an error), an error is returned.


### Extensions as Commands
Extensions are converted to command-line commands via an `extension.json` manifest, which encodes metadata and command data. Once installed, extensions produce CLI commands with defined subcommands and arguments through this manifest. The manifest file contains metadata like the extension's name, version, authors, dependencies, and subcommands. Each extension can possess subcommands with specific arguments. By translating the manifest into command-line commands, dfx enables the user to interact with extension commands, including their subcommands and arguments, which in practice means that any command and subcommand will be displayed when `dfx EXT_NAME --help`, or `dfx EXT_NAME SUBCMD` is called, and it will display all available args, options and subcommands. 
