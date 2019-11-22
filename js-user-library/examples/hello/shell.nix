# The goal of this nix-shell is to provide a somewhat clean environment for the
# state of the SDK as it exists on the current branch. We do this by not
# relying on, or modifying, any global paths where the SDK may have previously
# been installed.

let pkgs = (import ../../.. {}).pkgs; in
let sdk = pkgs.dfinity-sdk.packages; in

pkgs.mkShell {
  buildInputs = [
    sdk.rust-workspace # for dfx
    pkgs.jq # for reading config
    pkgs.mktemp
    pkgs.nodejs-10_x
  ];
  shellHook = ''
    set -e
    temp_home=$(mktemp -d)
    export HOME="$temp_home"

    # Temporarily remove the "dfx" field in dfx.json so that we can use the
    # version of dfx in the rust workspace. Otherwise, dfx can complain that a
    # version matching the project can't be found. Preferably we would set this
    # to the version reported by `dfx --version` but can't due to SDK-613.
    version=$(dfx config dfx)
    dfx config dfx null

    # Ideally we would depend on pkgs.dfinity-sdk.js-user-library, and changes
    # there would trigger a rebuild when entering this shell.
    pushd ../..
    npm install
    npm run bundle
    popd

    npm install

    # Hack to make sure that binaries are installed
    temp_project=$(mktemp -d)
    pushd "$temp_project"
    dfx new temp &> /dev/null
    popd
    rm -rf "$temp_project"

    dfx start --background
    dfx build hello
    dfx canister install hello

    npm run bundle

    open $(jq --raw-output '"http://\(.defaults.start.address):\(.defaults.start.port)"' dfx.json)

    set +e

    # Clean up before we exit the shell
    trap "{ \
      dfx stop; \
      dfx config dfx "''${version}"; \
      rm -rf "$temp_home"; \
      exit 255; \
    }" EXIT
  '';
}
