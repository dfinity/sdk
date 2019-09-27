{ napalm }:

let package = napalm.buildPackage ./. {
  npmCommands = [
    "npm install"
    "npm run bundle"
  ];
}; in

package.overrideAttrs (oldAttrs: {
  installPhase = ''
    mkdir -p $out
    cp package.json $out
    cp README.adoc $out
    cp -r dist/ $out
  '';
})
