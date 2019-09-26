let pkgs = (import ../../.. {}).pkgs; in
let sdk = pkgs.dfinity-sdk.packages; in

pkgs.mkShell {
  buildInputs = [
    sdk.rust-workspace # for dfx
    pkgs.nodejs-10_x
  ];
  shellHook = ''
    set -e

    npm install
    dfx build

    # HACK: work around issues with generated JS bindings:
    # * Nothing is exported
    # * Depends on `require` instead of standards based imports
    # * `require("IDL")` is inflexible
    # * "Message" was renamed to "Func"
    # * IDL.Obj vs arrays for arguments and return values?
    echo "export default ({ IDL }) => {" > build/canisters/hello/main.js
    echo "  const Text = IDL.Text;" >> build/canisters/hello/main.js
    echo "  return new IDL.ActorInterface({" >> build/canisters/hello/main.js
    # echo "    'greet': IDL.Func(IDL.Obj({'0': Text}), IDL.Obj({'0': Text}))" >> build/canisters/hello/main.js
    echo "    'greet': IDL.Func([Text], [Text])" >> build/canisters/hello/main.js
    echo "  });" >> build/canisters/hello/main.js
    echo "};" >> build/canisters/hello/main.js

    npm run bundle

    dfx start

    npx forever start -c "npx serve -l 1234" dist
    sleep 5s
    open http://localhost:1234

    set +e

    # Clean up before we exit the shell
    trap "{ \
      npx forever stopall
      kill $(pgrep nodemanager)
      kill $(pgrep client)
      exit 255; \
    }" EXIT
  '';
}
