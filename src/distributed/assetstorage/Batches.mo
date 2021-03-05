import Debug "mo:base/Debug";
import Int "mo:base/Int";
import Time "mo:base/Time";

import H "mo:base/HashMap";
import T "Types";
import U "Utils";

module {

public class Batches() {
  // We group the staged chunks into batches.  Uploading a chunk refreshes the batch's expiry timer.
  // We delete expired batches so that they don't consume space after an interrupted install.
  var nextBatchId = 1;
  let batches = H.HashMap<Int, T.Batch>(7, Int.equal, Int.hash);

  public func get(batchId: T.BatchId) : ?T.Batch {
    batches.get(batchId)
  };
  public func delete(batchId: T.BatchId) {
    batches.delete(batchId)
  };

  public func startBatch(): T.BatchId {
    let batch_id = nextBatchId;
    nextBatchId += 1;
    let batch = T.Batch();
    batches.put(batch_id, batch);
    batch_id
  };

  public func deleteExpired() : () {
    let now = Time.now();
    /*let batchesToDelete: H.HashMap<Int, T.Batch> = H.mapFilter<Int, T.Batch, T.Batch>(batches, Int.equal, Int.hash,
      func(k: Int, batch: T.Batch) : ?T.Batch {
        if (batch.expired(now))
          ?batch
        else
          null
      }
    );
    for ((k,_) in batchesToDelete.entries()) {
      Debug.print("delete expired batch " # Int.toText(k));

      batches.delete(k);
    };*/
    U.deleteFromHashMap(batches, Int.equal, Int.hash,
      func(k: Int, batch: T.Batch) : Bool {
        batch.expired(now)
      }
    );
  };

  public func reset() {
    nextBatchId := 1;
    U.clearHashMap(batches);
  }
}

}