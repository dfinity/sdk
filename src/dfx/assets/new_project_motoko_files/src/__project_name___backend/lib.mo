import Text "mo:base/Text";
import Iter "mo:base/Iter";
import Buffer "mo:base/Buffer";
import List "mo:base/List";
module {
  /**
    * A simple of example of a Motoko method.
    * @param {Text} name - The name to greet. 
    * @return {Text} A greeting.
    */
  public func greet(name : Text) : Text {
    "Hello, " # name # "!";
  };
};
