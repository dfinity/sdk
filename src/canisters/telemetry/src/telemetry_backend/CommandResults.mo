import Trie "mo:base/Trie";

import Aggregation "Aggregation";
import CommandDetails "CommandDetails";
import InvocationDetails "InvocationDetails";

module CommandResults {
  public type Trie3D<K1, K2, K3, V> = Trie.Trie3D<K1, K2, K3, V>;
  type AggregationPeriodStart = Aggregation.AggregationPeriodStart;
  type CommandDetails = CommandDetails.CommandDetails;
  type InvocationDetails = InvocationDetails.InvocationDetails;

  public type CommandResultCounts = {
    var successes : Nat;
    var failures : Nat;
  };

  public type CommandResults = Trie3D<
    AggregationPeriodStart, InvocationDetails, CommandDetails,
    CommandResultCounts>;
}
