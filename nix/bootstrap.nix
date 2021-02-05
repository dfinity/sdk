{ pkgs ? import ../. { inherit system; }, system ? builtins.currentSystem }:
pkgs.runCommandNoCC "assets-bootstrap" {
  version = "0.0.0";
  nativeBuildInputs = [
    pkgs.nodejs
  ];
} ''
    # npm looks for config and cache files in the HOME folder, but tries to create HOME
    # if it doesn't exist. Make sure we have a HOME.
    export HOME=$(mktemp -d)

    mkdir -p $out
    cd $out

    npm pack @dfinity/bootstrap@$version
    tar xzvf dfinity-bootstrap-*.tgz package/dist/
    rm dfinity-bootstrap-*.tgz
    mv package/dist/* .
    rmdir package/dist
    rmdir package/

''
