{ napalm }:

let package = napalm.buildPackage ./. {
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
