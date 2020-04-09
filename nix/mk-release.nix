{ stdenv
, lib
, gzip
, jo
, patchelf
}:
rname: version: from: what:
stdenv.mkDerivation {
  name = "${rname}-release";
  inherit rname version from what;
  phases = [ "buildPhase" ];
  buildInputs = [ gzip jo patchelf ];
  allowedRequisites = [];
  buildPhase = ''
    mkdir -p $out
    the_release="$rname-$version.tar.gz"
    # Assemble the fully standalone archive
    collection=$(mktemp -d)
    cp $from/bin/$what $collection/$what
    chmod 0755 $collection/$what
    tar -cvzf "$out/$the_release" -C $collection/ .
  '';
}
