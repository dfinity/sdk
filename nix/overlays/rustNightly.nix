self: super: {
  rustNightlyLib = super.callPackage super.sources.rust-nightly-nix { };

  rustNightly = self.rustNightlyLib.rust {
    # To update:
    # * change the date to desired date,
    # * replace the hashes below with `self.lib.fakeSha256`
    # * run the following for both architectures:
    #   `nix-build -A rustNightly --argstr system <x86_64-darwin / x86_64-linux>`
    # * replace the fake hash with the gotten hash
    date = "2019-08-13";
    hash = {
      "x86_64-darwin" = "08k1gl2kcy17pavpzlf4nkhfw0dyq19kaly6b9f4zjp3q00vjlsj";
      "x86_64-linux"  = "0xjkprbmqidzaxbby7ky1g0j5xy0bbybx7c547qmbzjrqcplqwcc";
    }."${self.system}";
  };
}
