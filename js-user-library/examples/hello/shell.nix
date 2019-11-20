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
    export HOME=$(mktemp -d)

    # Ideally we would depend on pkgs.dfinity-sdk.js-user-library, and changes
    # there would trigger a rebuild.
    pushd ../..
    npm install
    npm run bundle
    popd

    npm install

    # Hack to make sure that binaries are installed
    pushd $(mktemp -d)
    dfx new temp &> /dev/null
    popd

    dfx start --background
    dfx build hello
    dfx canister install $(jq --raw-output '.canisters.hello.deployment_id' dfx.json) canisters/hello/main.wasm

    npm run bundle

    open $(jq --raw-output '"http://\(.defaults.start.address):\(.defaults.start.port)"' dfx.json)

    set +e

    # Clean up before we exit the shell
    trap "{ \
      killall dfx nodemanager client
      exit 255; \
    }" EXIT
  '';
}
