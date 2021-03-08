import Debug "mo:base/Debug";
import HashMap "mo:base/HashMap";
import Int "mo:base/Int";
import Time "mo:base/Time";

import T "Types";
import U "Utils";

module {

object batch {
  public func nextExpireTime() : T.Time {
    let expiryNanos = 5 * 60 * 1000 * 1000 * 1000;
    Time.now() + expiryNanos
  }
};

// A batch associates a bunch of chunks that are being uploaded, so that none
// of them time out or all of them do.
public class Batch(initBatchId: T.BatchId) {
  public let batchId = initBatchId;
  var expiresAt : T.Time = batch.nextExpireTime();

  public func refreshExpiry() {
    expiresAt := batch.nextExpireTime();
  };

  public func isExpired(asOf : T.Time) : Bool {
    expiresAt <= asOf
  };
};

// We group the staged chunks into batches.  Uploading a chunk refreshes the batch's expiry timer.
// We delete expired batches so that they don't consume space forever after an interrupted install.
public class Batches() {
  var nextBatchId = 1;
  let batches = HashMap.HashMap<Int, Batch>(7, Int.equal, Int.hash);

  public func get(batchId: T.BatchId) : ?Batch {
    batches.get(batchId)
  };

  public func delete(batchId: T.BatchId) {
    batches.delete(batchId)
  };

  public func create(): Batch {
    let batchId = nextBatchId;
    nextBatchId += 1;
    let batch = Batch(batchId);
    batches.put(batchId, batch);
    batch
  };

  public func deleteExpired() : () {
    let now = Time.now();
    U.deleteFromHashMap(batches, Int.equal, Int.hash, func(k: Int, batch: Batch) : Bool = batch.isExpired(now));
  };

  public func reset() {
    nextBatchId := 1;
    U.clearHashMap(batches);
  }
}

}
