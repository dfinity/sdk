actor WhoAmI {
  public shared ({caller}) func whoami() : async Principal {
    return caller;
  };
};
