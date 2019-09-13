{ pkgs ? import ../. {} }:

pkgs.mkCiShell {
  name = "dfinity-js-user-library-env";
  buildInputs = [
    pkgs.nodejs-10_x
  ];
}
