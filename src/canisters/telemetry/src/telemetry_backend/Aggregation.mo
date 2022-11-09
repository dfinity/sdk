import Hash "mo:base/Hash";
import Int "mo:base/Int";
import Time "mo:base/Time";

module Aggregation {
  type Hash = Hash.Hash;
  public type AggregationPeriodStart = Time.Time;

  public type AggregationTrieKey = {
    key : AggregationPeriodStart;
    hash : Hash;
  };

  public func trieKey(v : AggregationPeriodStart) : AggregationTrieKey {
    {
      key = v;
      hash = Int.hash v;
    }
  };
}
