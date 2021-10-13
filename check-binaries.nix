{ pkgs ? import ./nix {}
, dfx ? import ./dfx.nix { inherit pkgs; }
}:
let
   lib = pkgs.lib;
in
pkgs.runCommandNoCCLocal "check-binaries" {
      nativeBuildInputs = with pkgs; [
        dfx.build
      ] ++ lib.optional stdenv.isDarwin darwin.binutils;
    } ''
    echo "check the binaries!"
    mkdir -p $out
    export DFX_CONFIG_ROOT="$out"
    dfx cache install
    CACHE_DIR="$(dfx cache show)"
    echo "Cache dir is $CACHE_DIR"

    if uname -a | grep Linux; then
        echo "checking linux.."
        for a in dfx replica ic-starter ic-ref;
        do
            echo "checking $a"
            ldd "$CACHE_DIR/$a"
        done
    else
        echo "checking osx.."
        for a in dfx replica ic-starter ic-ref;
        do
            echo "checking $a"
            # will have to find otool....
            otool -L "$CACHE_DIR/$a"
        done
    fi

''