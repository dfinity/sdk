import Hash "mo:base/Hash";

module Parameters {
  type Hash = Hash.Hash;

  public type IdentityStorageMode = {
    #plaintext;
    #keyring;
    #passwordProtected
  };

  public type DfxStartOptions = {
    clean : Bool;
    emulator : Bool;
    host : Bool;
  };
  public type DfxDeployOptions = {
    singleCanister : Bool;
  };
  public type DfxIdentityNewOptions = {
    storageMode: IdentityStorageMode;
  };
  public type Parameters = {
    #dfxStart : DfxStartOptions;
    #dfxDeploy : DfxDeployOptions;
    #dfxIdentityNew : DfxIdentityNewOptions;
  };

  func encodeSingleBit(b : Bool) : Nat32 {
    if b {
      1
    } else {
      0
    }
  };
  func encodeThreeBits(b0 : Bool, b1 : Bool, b2 : Bool)  : Nat32 {
    var v : Nat32 = 0;
    if b0 {
      v |= 1;
    };
    if b1 {
      v |= 2;
    };
    if b2 {
      v |= 4;
    };
    v
  };

  func encodeDfxDeployOptions(opts : DfxDeployOptions) : Nat32 {
    encodeSingleBit(opts.singleCanister)
  };

  func encodeDfxIdentityNewOptions(
    opts : DfxIdentityNewOptions
  ) : Hash {
    switch (opts.storageMode) {
      case (#plaintext) 0;
      case (#keyring) 1;
      case (#passwordProtected) 2;
    };
  };

  func encodeDfxStartOptions(opts : DfxStartOptions) : Nat32 {
    encodeThreeBits(opts.clean, opts.emulator, opts.host)
  };

  public func encodeForHash(p : Parameters) : Nat32 {
    switch (p) {
      case (#dfxDeploy deployOptions) encodeDfxDeployOptions(deployOptions);
      case (#dfxIdentityNew identityNewOptions)
        encodeDfxIdentityNewOptions(identityNewOptions);
      case (#dfxStart startOptions) encodeDfxStartOptions(startOptions);
    }
  };
}
