import Time "mo:base/Time";
import Trie "mo:base/Trie";

import CommandResults "CommandResults";

module Data {
  public type Time = Time.Time;
  public type Trie<K, V> = Trie.Trie<K, V>;
  type CommandResults = CommandResults.CommandResults;

  public type V0 = {
    var dailyAggregationPeriodStart : Time;
    var dailyAggregationPeriodEnd : Time;

    var thirtyDayAggregationPeriodStart : Time;
    var thirtyDayAggregationPeriodEnd : Time;

    var commandResults : CommandResults;

    var overrideTime : ?Time;
  };

  public type Data = V0;
  public type Versioned = {
    #v0 : V0;
  };

  public func new() : Data {
    {
      var dailyAggregationPeriodStart = 0;
      var dailyAggregationPeriodEnd = 0;

      var thirtyDayAggregationPeriodStart = 0;
      var thirtyDayAggregationPeriodEnd = 0;

      var commandResults = Trie.empty();

      var overrideTime = null;
    }
  };
}
