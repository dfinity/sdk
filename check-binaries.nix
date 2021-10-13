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

  result=0
  for a in dfx ic-ref ic-starter icx-proxy mo-doc mo-ide moc replica;
  do
      echo
      echo "checking $a"

      if ! output="$(${lib_list_tool} "$CACHE_DIR/$a" 2>&1)"; then
          echo "$output"
          if echo "$output" | grep -q "not a dynamic executable"; then
              continue
          else
              result=1
          fi
      else
          echo "$output"
          echo
          if matches="$(echo "$output" | grep -v '^\/' | grep "/nix/store")"; then
              echo "** fails because $a references /nix/store:"
              echo "$matches"
              result=1
          fi
      fi
  done
  exit $result
''
