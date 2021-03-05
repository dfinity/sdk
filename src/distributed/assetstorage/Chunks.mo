import Debug "mo:base/Debug";
import HashMap "mo:base/HashMap";
import Int "mo:base/Int";
import Result "mo:base/Result";
import Time "mo:base/Time";

import T "Types";
import U "Utils";

module {

public class Chunks() {
  // We stage asset content chunks here,
  // before assigning them to asset content encodings.
  var nextChunkId = 1;
  let chunks = HashMap.HashMap<Int, T.Chunk>(7, Int.equal, Int.hash);

  public func create(batch: T.Batch, content: Blob) : T.ChunkId {
    let chunkId = nextChunkId;
    nextChunkId += 1;
    let chunk : T.Chunk = {
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
      /*let chunksToDelete = HashMap.mapFilter<Int, T.Chunk, T.Chunk>(chunks, Int.equal, Int.hash,
        func(k: Int, chunk: T.Chunk) : ?T.Chunk {
          if (chunk.batch.expired(now))
            ?chunk
          else
            null
        }
      );
      for ((k,_) in chunksToDelete.entries()) {
        Debug.print("delete expired chunk " # Int.toText(k));
        chunks.delete(k);
      };*/
          U.deleteFromHashMap(chunks, Int.equal, Int.hash,
            func(k: Int, chunk: T.Chunk) : Bool {
              chunk.batch.expired(now)
            }
          );

  };
}

}