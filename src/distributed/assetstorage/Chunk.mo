import Debug "mo:base/Debug";
import HashMap "mo:base/HashMap";
import Int "mo:base/Int";
import Result "mo:base/Result";
import Time "mo:base/Time";

import B "Batch";
import T "Types";
import U "Utils";

module Chunk {

// A chunks holds a staged piece of content until we assign it to
// an asset by content-encoding.
public type Chunk = {
  batch: B.Batch;
  content: Blob;
};

public class Chunks() {
  var nextChunkId = 1;
  let chunks = HashMap.HashMap<Int, Chunk>(7, Int.equal, Int.hash);

  // Create a new chunk for a piece of content.  This refreshes the batch's
  // expiry timer.
  public func create(batch: B.Batch, content: Blob) : T.ChunkId {
    let chunkId = nextChunkId;
    nextChunkId += 1;
    let chunk : Chunk = {
      batch = batch;
      content = content;
    };

    batch.refreshExpiry();
    chunks.put(chunkId, chunk);

    chunkId
  };

  public func take(chunkId: T.ChunkId): Result.Result<Blob, Text> {
    switch (chunks.remove(chunkId)) {
      case null #err("chunk not found");
      case (?chunk) #ok(chunk.content);
    }
  };

  public func reset() {
    nextChunkId := 1;
    U.clearHashMap(chunks);
  };

  public func deleteExpired() : () {
    let now = Time.now();

    U.deleteFromHashMap(chunks, Int.equal, Int.hash, func(k: Int, chunk: Chunk) : Bool = chunk.batch.isExpired(now));
  };
}

}
