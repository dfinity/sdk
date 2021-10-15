{ pkgs ? import ./nix {}
, dfx ? import ./dfx.nix { inherit pkgs; }
}:
let
  lib = pkgs.lib;

  lib_list_tool = if pkgs.stdenv.isDarwin then "otool -L" else "ldd";

in
pkgs.runCommand "check-binaries" {
  nativeBuildInputs = with pkgs; [
    which
    dfx.standalone
  ] ++ lib.optional stdenv.isDarwin darwin.binutils
  ++ lib.optional stdenv.isLinux [ glibc.bin patchelf ];
} ''
  mkdir -p $out
  export DFX_CONFIG_ROOT="$out"
  cp ${dfx.standalone}/bin/dfx dfx

  ${pkgs.lib.optionalString pkgs.stdenv.isLinux ''
  # distributed dfx needs some surgery in order to run under nix
  local LD_LINUX_SO=$(ldd $(which iconv)|grep ld-linux-x86|cut -d' ' -f3)
  chmod +rw ./dfx
  patchelf --set-interpreter "$LD_LINUX_SO" ./dfx
''}

  ./dfx cache install
  CACHE_DIR="$(./dfx cache show)"

  check_binary() {
      path=$1

      echo
      echo "checking $path"

      if ! output="$(${lib_list_tool} "$path" 2>&1)"; then
          echo "$output"
          if echo "$output" | grep -q "not a dynamic executable"; then
              return 0
          else
              return 1
          fi
      else
          libraries="$(echo "$output" | grep -v '^\/' | cut -f 1 -d ' ')"
          echo "$output"
          echo "Libraries:"
          echo "$libraries"
          echo
          if found="$(echo "$libraries" | grep "/nix/store")"; then
              echo "** fails because $path references /nix/store:"
              echo "$found"
              return 1
          else
              return 0
          fi
      fi
  }

  result=0

  # On linux, the dfx binary in the cache will be copied from the one we patched above,
  # so check the original binary separately:
  if ! check_binary "${dfx.standalone}/bin/dfx"; then
    result=1
  fi

  for a in ic-ref ic-starter icx-proxy mo-doc mo-ide moc replica;
  do
      if ! check_binary "$CACHE_DIR/$a"; then
          result=1
      fi
  done
  exit $result
''
