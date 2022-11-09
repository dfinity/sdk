import Command "Command";
import CommandResults "CommandResults";
import Network "Network";
import Parameters "Parameters";
import Platform "Platform";

module Interface {
  type Command = Command.Command;
  type Network = Network.Network;
  type Parameters = Parameters.Parameters;
  type Platform = Platform.Platform;

  public type ReportCommandArgs = {
    dfxVersion : Text;
    platform : Platform;
    network : Network;
    command : Command;
    parameters : ?Parameters;
    success : Bool;
  };

  public type CommandSuccessRatesEntry = {
    command : Command;
    parameters : ?Parameters;
    successRate : Nat;
  };

  public type CommandResultsEntry = {
    command : Command;
    parameters : ?Parameters;
    successes : Nat;
    failures : Nat;
  };
}
