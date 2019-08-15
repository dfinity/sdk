{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
, overlays ? []
}:
import (import ./sources.nix).nixpkgs {
  inherit system crossSystem config;
  overlays = (import ./overlays) ++ overlays;
}
