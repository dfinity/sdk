{ pkgs ? import ./nix {}
, dfx ? import ./dfx.nix { inherit pkgs; }
}:
let
  lib = pkgs.lib;

  lib_list_tool = if pkgs.stdenv.isDarwin then "otool -L" else "ldd";

in
pkgs.runCommand "check-binaries" {
  nativeBuildInputs = with pkgs; [
    dfx.build
  ] ++ lib.optional stdenv.isDarwin darwin.binutils
  ++ lib.optional stdenv.isLinux glibc.bin;
} ''
  mkdir -p $out
  export DFX_CONFIG_ROOT="$out"
  dfx cache install
  CACHE_DIR="$(dfx cache show)"

  for a in dfx ic-ref ic-starter icx-proxy mo-doc mo-ide moc replica;
  do
      echo
      ${lib_list_tool} "$CACHE_DIR/$a"
  done
''
