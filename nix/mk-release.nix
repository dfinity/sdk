{ stdenv
, lib
, gzip
, jo
, patchelf
}:
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

    tar -cvzf "$out/$the_release" -C $collection/ .
  '';
}
