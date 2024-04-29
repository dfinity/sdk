import Array "mo:base/Array";
import CA "mo:candb/CanisterActions";
import Entity "mo:candb/Entity";
import CanDB "mo:candb/CanDB";
import Multi "mo:CanDBMulti/Multi";
import RBT "mo:stable-rbtree/StableRBTree";
import Principal "mo:base/Principal";
import Bool "mo:base/Bool";
import Debug "mo:base/Debug";
import Text "mo:base/Text";
import Nat "mo:base/Nat";
import lib "../backend/lib";

shared actor class CanDBPartition(options: {
  partitionKey: Text;
  scalingOptions: CanDB.ScalingOptions;
  owners: ?[Principal];
}) = this {
  stable var owners = switch (options.owners) {
    case (?p) { p };
    case _ { [] };
  };

  /// @required (may wrap, but must be present in some form in the canister)
  stable let db = CanDB.init({
    pk = options.partitionKey;
    scalingOptions = options.scalingOptions;
    btreeOrder = null;
  });

  func checkCaller(caller: Principal) {
    if (Array.find(owners, func(e: Principal): Bool { e == caller; }) == null) {
      Debug.print("CanDBPartition owners = " # debug_show(owners));
      Debug.trap("CanDBPartition: not allowed");
    }
  };

  public shared({caller}) func setOwners(_owners: [Principal]): async () {
    checkCaller(caller);

    owners := _owners;
  };

  public query func getOwners(): async [Principal] { owners };

  /// @recommended (not required) public API
  public query func getPK(): async Text { db.pk };

  /// @required public API (Do not delete or change)
  public query func skExists(sk: Text): async Bool { 
    CanDB.skExists(db, sk);
  };

  public query func get(options: CanDB.GetOptions): async ?Entity.Entity { 
    CanDB.get(db, options);
  };

  public shared({caller}) func put(options: CanDB.PutOptions): async () {
    checkCaller(caller);

    await* CanDB.put(db, options);
  };

  // public shared({caller}) func replace(options: CanDB.ReplaceOptions): async ?Entity.Entity {
  //   checkCaller(caller);

  //   await* CanDB.replace(db, options);
  // };

  // public shared({caller = caller}) func update(options: CanDB.UpdateOptions): async ?Entity.Entity {
  //   if (checkCaller(caller)) {
  //     CanDB.update(db, options);
  //   } else {
  //     null;
  //   };
  // };

  public shared({caller}) func delete(options: CanDB.DeleteOptions): async () {
    // checkCaller(caller); // FIXME: Uncomment.

    CanDB.delete(db, options);
  };

  // public shared({caller}) func remove(options: CanDB.RemoveOptions): async ?Entity.Entity {
  //   checkCaller(caller);

  //   CanDB.remove(db, options);
  // };

  public query func scan(options: CanDB.ScanOptions): async CanDB.ScanResult {
    CanDB.scan(db, options);
  };

  /// @required public API (Do not delete or change)
  public shared({caller}) func transferCycles(): async () {
    checkCaller(caller);

    return await CA.transferCycles(caller);
  };

  // public shared({caller}) func tryPut(options: CanDB.PutOptions): async () {
  //   checkCaller(caller);

  //   if (not CanDB.skExists(db, options.sk)) {
  //     await* CanDB.put(db, options);
  //   };
  // };

  func _getAttribute(options: CanDB.GetOptions, subkey: Text): ?Entity.AttributeValue {
    let all = CanDB.get(db, options);
    do ? { RBT.get(all!.attributes, Text.compare, subkey)! };
  };

  public query func getAttribute(options: CanDB.GetOptions, subkey: Text): async ?Entity.AttributeValue {
    // checkCaller(caller);
    _getAttribute(options, subkey);
  };

  // Application-specific code //

  // TODO: Retrieve virtual items.
  // TODO: `?lib.ItemData` -> `lib.ItemData`?
  public shared func getItem(itemId: Nat): async ?lib.ItemTransfer {
    let data = _getAttribute({sk = "i/" # Nat.toText(itemId)}, "i");
    switch (data) {
      case (?data) {
        let item = lib.deserializeItem(data);
        switch (item) {
          case (#owned item2) { ?{ data = { creator = item2.creator; item = item2.item; edited = item2.edited }; communal = false } };
          case (#communal c) {
            let scanResult = await c.votesStream.order.0.scanLimitOuter({
              dir = #fwd;
              outerKey = c.votesStream.order.1;
              lowerBound = "";
              upperBound = "x";
              limit = 1;
              ascending = ?true;
            });
            let ref = scanResult.results[0].1;
            let #text ref2 = ref else {
              Debug.trap("programming error");
            };
            let words = Text.split(ref2, #char '@');
            let w1o = words.next();
            let w2o = words.next();
            let (?itemId, ?itemCanisterId) = (w1o, w2o) else {
              Debug.trap("order: programming error");
            };
            // let ?itemId = Nat.fromText(w1) else {
            //   Debug.trap("order: programming error");
            // };
            let itemCanister: CanDBPartition = actor(itemCanisterId);
            switch (await itemCanister.getAttribute({sk = "r/" # itemId}, "i")) {
              case (?data) {
                let variant = lib.deserializeItemVariant(data);
                ?{ data = { creator = Principal.fromText("aaaaa-aa"); item = variant.item; edited = true }; communal = true }; // FIXME: principal
              };
              case null { return null };
            };
          };
        };
      };
      case null { null };
    };
  };

  public query func getStreams(itemId: Nat, kind: Text): async ?lib.Streams {
    // TODO: Duplicate code
    if (kind != "st" and kind != "rst" and kind != "sv" and kind != "rsv") {
      Debug.trap("wrong streams");
    };
    let data = _getAttribute({sk = "i/" # Nat.toText(itemId)}, kind);
    do ? { lib.deserializeStreams(data!) };
  };

  // CanDBMulti //

  public shared({caller}) func putAttribute(options: { sk: Entity.SK; key: Entity.AttributeKey; value: Entity.AttributeValue }): async () {
    checkCaller(caller);
    ignore await* Multi.replaceAttribute(db, options);
  };

  public shared({caller}) func putExisting(options: CanDB.PutOptions): async Bool {
    checkCaller(caller);
    await* Multi.putExisting(db, options);
  };

  public shared({caller}) func putExistingAttribute(options: { sk: Entity.SK; key: Entity.AttributeKey; value: Entity.AttributeValue })
    : async Bool
  {
    checkCaller(caller);
    await* Multi.putExistingAttribute(db, options);
  };
}