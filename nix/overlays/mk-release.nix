{ stdenv, lib, gzip, jo, patchelf }:
rname: version: from: what:
stdenv.mkDerivation {
  name = "${rname}-release";
  inherit version;
  phases = [ "buildPhase" ];
  buildInputs = [ gzip jo patchelf ];
  allowedRequisites = [];
  buildPhase = ''
    # Building the artifacts
    mkdir -p $out
    # we embed the system into the name of the archive
    the_release="${rname}-$version.tar.gz"
    # Assemble the fully standalone archive
    collection=$(mktemp -d)
    cp ${from}/bin/${what} $collection/${what}
    chmod 0755 $collection/${what}

    ${lib.optionalString stdenv.isLinux ''
    mkdir $collection/lib
    ldd ${from}/bin/${what} | { grep "=> /" || true; } | awk '{print $3}' | xargs -I '{}' cp '{}' $collection/lib/
    # $ORIGIN is relative to the executable (ld convention)
    patchelf --set-rpath '$ORIGIN/lib' $collection/${what}
    # Wrapper to use system's dynamic loader, as it might be the case
    # that the binary still references one from the nix store
    # This will not work on NixOS
    mv $collection/${what}{,.wrapped}
    cat >$collection/${what} <<'EOF'
    #!/usr/bin/env bash
    CURRENT_SCRIPT_DIR="$(
    cd "''${BASH_SOURCE[0]%/*}"
    pwd
    )"
    $CURRENT_SCRIPT_DIR/lib/ld-linux-x86-64.so.* $CURRENT_SCRIPT_DIR/${what}.wrapped "$@"
    EOF
    chmod 0755 $collection/${what}
    ''}

    tar -cvzf "$out/$the_release" -C $collection/ .

    # Creating the manifest
    manifest_file=$out/manifest.json

    sha256hash=($(sha256sum "$out/$the_release")) # using this to autosplit on space
    sha1hash=($(sha1sum "$out/$the_release")) # using this to autosplit on space

    jo -pa \
      $(jo package="${rname}" \
          version="$version" \
          system="${stdenv.system}" \
          name="${stdenv.system}/$the_release" \
          file="$out/$the_release" \
          sha256hash="$sha256hash" \
          sha1hash="$sha1hash") >$manifest_file

    # Marking the manifest for publishing
    mkdir -p $out/nix-support
    echo "upload manifest $manifest_file" >> \
      $out/nix-support/hydra-build-products
  '';
}
