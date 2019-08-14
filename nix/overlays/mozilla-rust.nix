# This imports the Mozilla Rust overlay from:
# https://github.com/mozilla/nixpkgs-mozilla
# This allows us to get the latest Rust release.
self: super: import "${super.sources.nixpkgs-mozilla}/rust-overlay.nix" self super
