# This file adds our Rust workspace to the global package set.
# It makes it more ergonomic to add inter-workspace dependencies
# when overriding certain crates. For example, many of our crates
# require the `replica` binary to run their testsuite.
self: super: with self.lib; {
  rustBuilder = super.rustBuilder.overrideScope' (
    self_rs: super_rs: {
      rustLib = super_rs.rustLib // {
        fetchCrateGit = { url, name, version, rev } @ args:
          let
            repo = builtins.fetchGit {
              inherit rev url;
              # This is a known issue in nix upstream, see https://github.com/NixOS/nix/issues/2431.
              # builtins.fetchGit doesn't know how to locate tree objects that aren't directly reachable
              # from origin/master.
              # If you add a new git dependency in Cargo.toml, and then run a Nix build, you might get
              # an error that looks like: "fatal: not a tree object: 01f5e794913a18494642b5f237bd76c054339d61".
              # In that case, add the ref (e.g. 01f5e79...) as a key here, with the corresponding
              # branch or tag name as the value.
              ref = {
                "43fe1ba0c803766f86bbd90a335229b42833e68e" = "fix-remove-index-ordmap";
                "73b51950cfc4f438bb71acb213be05a5eb81d9f9" = "v0.6-deterministic";
                "770cb8194342b8d3f1237edafb378338de541891" = "v0.13.0-fix-names-of-libz-and-libbz2";
                "858e6f3805abb2cb86d11bc2c0d6e70fd61b71c4" = "v0.19.0";
                "63d5b919306ebecc00cd39090910d89c02dcda9b" = "main";
                "9d611f08cb55923fc6ff5bf0136e8f4b848dd2cc" = "next";
              }.${rev} or "master";
            };
          in
            self.buildPackages.runCommandNoCC "find-crate-${name}-${version}" {
              nativeBuildInputs = [ self.buildPackages.jq self.buildPackages.remarshal ];
            } ''
              shopt -s globstar
              for f in ${repo}/**/Cargo.toml; do
                if [ "$(remarshal -if toml -of json "$f" | jq '.package.name == "${name}" and .package.version == "${version}"')" = "true" ]; then
                  cp -pr --no-preserve=all "''${f%/*}" $out
                  exit 0
                fi
              done

              echo Crate ${name}-${version} not found in ${url}
              exit 1
            '';
      };
      workspace = self_rs.mkDfinityWorkspace {
        cargoFile = ./Cargo.nix;
        crateOverrides = import ./overrides.nix self;
      };
    }
  );

  dfinity-foreach-crate = f: listToAttrs (
    filter (x: x != null) (
      mapAttrsToList (
        name: crate:
          if name == "dfn_macro" || name == "shell" then null else
            let attrs = { inherit name; value = f crate; }; in if attrs.value == null then null else attrs
      ) self.dfinity-sdk
    )
  );

  dfinity-sdk = (
    self.rustBuilder.overrideScope' (
      self_rs: super_rs: {
        inherit (self.pkgsStatic.rustBuilder) makePackageSet;
      }
    )
  ).workspace;
}
