import Lib "lib";

actor {
  public query func greet(name : Text) : async Text {
    return Lib.greet(name);
  };
};
