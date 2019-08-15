{ system ? builtins.currentSystem
, crossSystem ? null
, config ? {}
, overlays ? []
}: import ./nix/nixpkgs.nix { inherit system crossSystem config overlays; }
