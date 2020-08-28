# Returns the nixpkgs set overridden and extended with DFINITY specific
# packages.
{ system ? builtins.currentSystem
}:
let
  sourcesnix = builtins.fetchurl {
    url = https://raw.githubusercontent.com/nmattia/niv/d13bf5ff11850f49f4282d59b25661c45a851936/nix/sources.nix;
    sha256 = "0a2rhxli7ss4wixppfwks0hy3zpazwm9l3y2v9krrnyiska3qfrw";
  };
  sources = import sourcesnix { sourcesFile = ./sources.json; };
in
  sources
