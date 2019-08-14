# This nixpkgs overlay adds a mkCiShell, which is like mkShell,
# but is also a usable derivation so that we can test the shell-only
# dependencies and get them on hydra
self: super:
  { mkCiShell = {...}@attrs:
      self.mkShell (attrs // {
        nobuildPhase = ''
          echo
          echo "This derivation is meant to be used with nixShell"
          echo
          touch $out
      ''; });
  }
