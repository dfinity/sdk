# see README.adoc; this is referenced by hydra evaluation for master branch commits
with import <nixpkgs> {};
stdenv.mkDerivation {
    name = "ci";
}
