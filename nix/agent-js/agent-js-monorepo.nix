{ pkgs ? import ../. { inherit system; }
, system ? builtins.currentSystem
  # This should be via sourcesnix for the git monorepo
, agent-js-monorepo-src
}:
let
  src = agent-js-monorepo-src;
  # Ideally this is a function that takes a directory path (e.g. to the monorepo),
  # and returns a path to a JSON file shaped like ./package-lock.json.
  # But the package-lock.json should have all the .dependencies of subpackages in ./packages/*/ too.
  # Without something like this, napalm will only install deps of the top-level package,
  # and not deps of subpackages.
  lernaPackageLock = dir:
    let
      packageLock = (builtins.fromJSON (builtins.readFile "${dir}/package-lock.json"));
      packageNames = (builtins.attrNames (builtins.readDir "${dir}/packages"));
      packagePaths = (builtins.map (packageName: "${dir}/packages/${packageName}") packageNames);
      subpackageLockJsonPaths = (builtins.map (packagePath: "${packagePath}/package-lock.json") packagePaths);
      subpackageJsons = (builtins.map (file: (builtins.fromJSON (builtins.readFile file))) subpackageLockJsonPaths);
      packagesFoo = (builtins.trace "packages: ${packageNames}" packageNames);
      mergedDependencies = { };
      mergedPackageLock = (builtins.toJSON (pkgs.lib.attrsets.recursiveUpdate packageLock {
        dependencies = mergedDependencies;
      }));
      ret = (builtins.trace "packages: ${builtins.toJSON packageNames}" mergedPackageLock);
    in
    ret;
  mergedPackageLock = (lernaPackageLock src);
  mergedPackageLockFilename = "merged-packge-lock.json";
  mergedPackageLockFile = (pkgs.writeText mergedPackageLockFilename mergedPackageLock);
  src_with_merged = pkgs.stdenv.mkDerivation {
    name = "agent-js-monorepo-with-merged-package-lock-json";
    src = src;
    buildPhase = ''
      cp ${mergedPackageLockFile} merged-packge-lock.json;
    '';
    installPhase = ''
      mkdir -p $out
      cp -R ./* $out/
    '';
  };
  monorepo = pkgs.napalm.buildPackage src_with_merged {
    name = "agent-js-monorepo";
    packageLock = (src_with_merged + "/" + mergedPackageLockFilename);
    buildInputs = [
      pkgs.nodejs
      pkgs.jq
    ];
    outputs = [
      "out"
      "lib"
    ];
    HUSKY_DEBUG = "1";
    HUSKY_SKIP_INSTALL = "1";
    npmCommands = [
      "echo ${mergedPackageLockFilename}"
      "cat ${mergedPackageLockFilename}"
      "npm install --ignore-scripts"
      "patchShebangs ./node_modules/lerna/cli.js"
      "npm i"
    ];
    installPhase = ''
      mkdir -p $out

      cp -R ./* $out/

      # Copy node_modules to be reused elsewhere.
      mkdir -p $lib
      test -d node_modules && cp -R node_modules $lib || true
    '';
  };
in
monorepo
