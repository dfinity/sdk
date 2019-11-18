{ pkgs ? (import ../. {}).pkgs }:

let e2e-tests = pkgs.dfinity-sdk.packages.e2e-tests; in

pkgs.mkCiShell {
  name = "dfinity-e2e-tests-env";
  inputsFrom = [
    e2e-tests
  ];
}
