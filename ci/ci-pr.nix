# see README.adoc; this is referenced by hydra evaluation for pull requests
with import <nixpkgs> {};
stdenv.mkDerivation {
    name = "ci-pr";
}
