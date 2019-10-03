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
    export HOME=$TMP

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

    dfx build

    # Until https://github.com/dfinity-lab/actorscript/pull/693 is merged
    echo "export default ({ IDL }) => {" > build/canisters/hello/main.js
    echo "  const Text = IDL.Text;" >> build/canisters/hello/main.js
    echo "  return new IDL.ActorInterface({" >> build/canisters/hello/main.js
    echo "    'greet': IDL.Func(IDL.Obj({'0': Text}), IDL.Obj({'0': Text}))" >> build/canisters/hello/main.js
    # echo "    'greet': IDL.Func([Text], [Text])" >> build/canisters/hello/main.js
    echo "  });" >> build/canisters/hello/main.js
    echo "};" >> build/canisters/hello/main.js

    npm run bundle

    dfx start --background
    open $(jq --raw-output '"http://\(.defaults.start.address):\(.defaults.start.port)"' dfinity.json)

    set +e

    # Clean up before we exit the shell
    trap "{ \
      killall dfx nodemanager client
      exit 255; \
    }" EXIT
  '';
}
