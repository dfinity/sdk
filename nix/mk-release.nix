{ runCommandNoCC }:
rname: version: from: what:
runCommandNoCC "${rname}-${version}.tar.gz" {
  inherit from what;
  allowedRequisites = [];
} ''
  collection=$(mktemp -d)
  cp $from/bin/$what $collection/$what
  chmod 0755 $collection/$what
  tar -czf "$out" -C $collection/ .
''
