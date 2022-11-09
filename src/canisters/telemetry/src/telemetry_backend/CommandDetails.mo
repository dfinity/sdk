import Hash "mo:base/Hash";
import Option "mo:base/Option";

import Command "Command";
import Parameters "Parameters";

module CommandDetails {
  type Command = Command.Command;
  type Parameters = Parameters.Parameters;
  type Hash = Hash.Hash;

  public type CommandDetails = {
    command : Command;
    parameters : ?Parameters;
  };

  public type CommandDetailsTrieKey = {
    key : CommandDetails;
    hash : Hash;
  };

  public func hash(v : CommandDetails) : Hash {
    let key : [Hash] = [
      Command.encodeForHash(v.command),
      Option.getMapped(v.parameters, Parameters.encodeForHash, 0 : Hash)
    ];
    Hash.hashNat8(key)
  };

  public func equal(a : CommandDetails, b : CommandDetails) : Bool {
    a.command == b.command and a.parameters == b.parameters
  };

  public func trieKey(v : CommandDetails) : CommandDetailsTrieKey {
    {
      key = v;
      hash = hash v;
    }
  };
}
