self: with self.rustBuilder.rustLib;

let
  inherit (self.stdenv) isDarwin isLinux;
in

[
  (
    addEnvironment "dfx" {
      DFX_ASSETS = self.callPackage ./assets.nix {};
    }
  )
  (
    addEnvironment "openssl-sys" {
      OPENSSL_STATIC = true;
      OPENSSL_LIB_DIR = "${self.pkgsStatic.openssl.out}/lib";
      OPENSSL_INCLUDE_DIR = "${self.pkgsStatic.openssl.dev}/include";
    }
  )
  (
    if !isDarwin then nullOverride else addEnvironment "sysinfo" {
      # don't use CARGO_TARGET_*_RUSTFLAGS, it doesn't apply to build scripts
      NIX_x86_64_apple_darwin_LDFLAGS = [
        "-F${self.darwin.apple_sdk.frameworks.IOKit}/Library/Frameworks"
        "-F${self.darwin.CF}/Library/Frameworks"
      ];
    }
  )
  (
    if !isDarwin then nullOverride else addEnvironment "webpki-roots" {
      # don't use CARGO_TARGET_*_RUSTFLAGS, it doesn't apply to build scripts
      NIX_x86_64_apple_darwin_LDFLAGS = [
        "-F${self.darwin.apple_sdk.frameworks.Security}/Library/Frameworks"
        "-F${self.darwin.CF}/Library/Frameworks"
      ];
    }
  )
]
