# This adds our rust builder 'naersk' to the set of packages.
_: pkgs: { naersk = pkgs.callPackage pkgs.sources.naersk {} ; }
