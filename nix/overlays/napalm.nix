_: pkgs: {
  napalm = pkgs.callPackage pkgs.sources.napalm {
    pkgs = pkgs // { nodejs = pkgs.nodejs-10_x; };
  };
}
