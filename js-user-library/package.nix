{ napalm }:

let package = napalm.buildPackage ./. {
  npmCommands = [
    "npm install"
    "npm test"
  ];
}; in

package.overrideAttrs (oldAttrs: {
  installPhase = ''
    mkdir -p $out
    cp package.json $out
    cp README.adoc $out
    cp -r out/ $out
  '';
})
