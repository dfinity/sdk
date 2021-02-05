{ pkgs ? import ../. { inherit system; }, system ? builtins.currentSystem }:
pkgs.runCommandNoCC "agent-js-monorepo-package-bootstrap" {
  nativeBuildInputs = with pkgs; [
    nodejs
  ];
} ''

    mkdir -p $out
    cd $out

    npm pack @dfinity/bootstrap@0.0.0
    tar xzvf dfinity-bootstrap-*.tgz package/dist/
    rm dfinity-bootstrap-*.tgz
    mv package/dist/* .
    rmdir package/dist
    rmdir package/

''
