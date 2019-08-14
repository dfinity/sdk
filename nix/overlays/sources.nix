# This nixpkgs overlay adds the `sources` attribute to the nixpkgs set which is defined
# as an attribute set of fetched source code managed by `niv`.
#
# Why use an overlay?
#
# Without an overlay derivations that need to reference external source code
# have to use `import ../../../nix/sources.nix` where the path of `../../../`
# depends on where the derivation is defined relative to `/nix/sources.nix`.
#
# With this overlay derivations can just use the `sources` argument from pkgs.
self: super: { sources = import ../sources.nix; }
