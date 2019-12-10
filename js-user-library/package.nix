{ napalm }:

let package = napalm.buildPackage ./. {
  # ci script now does everything CI should do. Bundle is needed because it's the output
  # of the nix derivation.
  npmCommands = [
    "npm install"
    "npm run ci"
    "npm run bundle"
  ];
}; in

package.overrideAttrs (oldAttrs: {
  name = "dfinity-sdk-js-user-library";
  installPhase = ''
    mkdir -p $out
    cp -R dist $out
    cp package.json $out
    cp README.adoc $out
  '';
})
