# The function call
#
#   gitSource ./toplevel subpath
#
# creates a Nix store path of ./toplevel/subpath that includes only those files
# tracked by git. More precisely: mentioned in the git index (i.e. git add is enough
# to get them to be included, you do not have to commit).
#
# This is a whitelist-based alternative to manually listing files or using
# nix-gitignore.

# Internally, it works by calling git ls-files at evaluation time. To
# avoid copying all of `.git` to the git store, it only copies the least amount
# of files necessary for `git ls-files` to work; this is a bit fragile, but
# very fast.

nixpkgs:

with builtins;

# We read the git index once, before getting the subdir parameter, so that it
# is shared among multiple invocations of gitSource:

let
  filter_from_list = root: files:
    let
      all_paren_dirs = p:
        if p == "." || p == "/"
        then []
        else [ p ] ++ all_paren_dirs (dirOf p);

      whitelist_set = listToAttrs (
        concatMap (p:
          let full_path = toString (root + "/${p}"); in
          map (p': { name = p'; value = true; }) (all_paren_dirs full_path)
        ) files
      );
    in
    p: t: hasAttr (toString p) whitelist_set;

  has_prefix = prefix: s:
    prefix == builtins.substring 0 (builtins.stringLength prefix) s;
  remove_prefix = prefix: s:
    builtins.substring
      (builtins.stringLength prefix)
      (builtins.stringLength s - builtins.stringLength prefix)
      s;

  lines = s: filter (x : x != [] && x != "") (split "\n" s);

  repo-root = ../../..;

  git-dir-path = repo-root + "/.git";
in

if builtins.pathExists git-dir-path
then
  let
    git_dir =
      if builtins.pathExists (git-dir-path + "/index")
      then git-dir-path
      else # likely a git worktree, so follow the indirection
        let
          git_content = lines (readFile git-dir-path);
          first_line = head git_content;
          prefix = "gitdir: ";
          ok = length git_content == 1 && has_prefix prefix first_line;
        in
          if ok
          then /. + remove_prefix prefix first_line
          else abort "gitSource.nix: Cannot parse ${toString git-dir-path}";

    whitelist_file =
      nixpkgs.runCommand "git-ls-files" {envVariable = true;} ''
        cp ${git_dir + "/index"} index
        echo "ref: refs/heads/master" > HEAD
        mkdir objects refs
        ${nixpkgs.git}/bin/git --git-dir . ls-files > $out
      '';

    whitelist = lines (readFile (whitelist_file));

    filter = filter_from_list repo-root whitelist;
  in
    subdir: nixpkgs.lib.cleanSourceWith {
      src = if isString subdir then (repo-root + "/${subdir}") else subdir;
      inherit filter;
    }

else
  let warn_unless = b: m: x: if b then x else trace m x; in
  # No .git directory found, we should warn the user.
  # But when this repository is imported using something like
  # `builtins.fetchGit` then the source is extracted to /nix/store without a
  # .git directory, but in this case we know that it is clean, so do not warn
  warn_unless
    (has_prefix "/nix/store" (toString repo-root))
    "gitSource.nix: ${toString ../.} does not seem to be a git repository,\nassuming it is a clean checkout."
    (subdir: nixpkgs.lib.cleanSourceWith {
      src = if isString subdir then (repo-root + "/${subdir}") else subdir;
      filter = _p: _t: true;
    })
