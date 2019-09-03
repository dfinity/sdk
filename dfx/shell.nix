{ pkgs ? import ../. {} }:

let assets = import ./assets.nix {
  inherit (pkgs) actorscript dfinity;
}; in

pkgs.mkCiShell {
  name = "dfinity-sdk-dfx-env";
  inputsFrom = [
    pkgs.dfinity-sdk.dfx
  ];
  shellHook = ''
    echo "{}" > dfinity.json
    ${assets.copy}

    # Clean up before we exit the shell
    trap "{ \
      rm dfinity.json
      rm -rf ${assets.subdir}; \
      exit 255; \
    }" EXIT
  '';
}
