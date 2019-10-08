{ napalm }:

let package = napalm.buildPackage ./. {}; in

package.overrideAttrs (oldAttrs: {
  name = "dfinity-sdk-js-user-library";
  installPhase = ''
    mkdir -p $out
    cp package.json $out
    cp README.adoc $out
  '';
})
