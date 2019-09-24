# This nixpkgs overlay extends `lib` with our own Nix functions.
self: super: {
  lib = super.lib // {
    gitOnlySource = src: super.lib.cleanSourceWith {
      src = import ./gitSource.nix self src;
      filter = path: type:
        !(super.lib.hasSuffix ".nix" (toString path)
         && !(type == "directory"));
    };

    # Filter sources by including all directories but filtering files by a list of
    # regular expressions.
    #
    # E.g. `src = sourceFilesByRegex ./my-subproject [".*\.py$" "^database.sql$"]`
    sourceFilesByRegex = src: regexes:
      let
        isFiltered = src ? _isLibCleanSourceWith;
        origSrc = if isFiltered then src.origSrc else src;
      in super.lib.cleanSourceWith {
        filter = path: type:
          let relPath = super.lib.removePrefix (toString origSrc + "/") (toString path);
          in type == "directory" || super.lib.any (re: builtins.match re relPath != null) regexes;
        inherit src;
      };

    # The "root" of the dfinity repository. On Hydra, this is ".". Outside of
    # Hydra, this is the absolute path to the root of the repository on the
    # user's system.
    dfinityRoot =
      # We use a bit of a hack to figure out if we're on Hydra: if <src> is
      # set, then yes, otherwise we assume not.
      if (builtins.tryEval <src>).success
      then # we're on Hydra
        "."
      else # this is a local build
        builtins.toString ../../..;
  };
}
