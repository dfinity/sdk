let pkgs = (import ./. {}).pkgs; in
let sdk = pkgs.dfinity-sdk.packages; in

pkgs.runCommand
  "temp"
  {
    buildInputs = [
      pkgs.binutils
      pkgs.mktemp
      #sdk.rust-workspace # for dfx
      sdk.rust-workspace-standalone # for dfx
    ];
  }
  ''
    export HOME=$(mktemp -d)

    dfx new temp
    cd temp
    dfx start --background
    dfx build hello
    #dfx canister install 42 build/canisters/hello/main.wasm # SDK-546
    dfx canister install 42 canisters/src/hello/main.wasm
    
    mkdir -p $out
    cp -R ./. $out
  ''
