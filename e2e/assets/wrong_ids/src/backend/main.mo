import Nac "mo:nacdb/NacDB";
import Principal "mo:base/Principal";
import Debug "mo:base/Debug";
import Text "mo:base/Text";
import Nat "mo:base/Nat";
import Buffer "mo:base/Buffer";
import Array "mo:base/Array";
import Reorder "mo:nacdb-reorder/Reorder";
import order "canister:order";
import GUID "mo:nacdb/GUID";
import Entity "mo:candb/Entity";
import Itertools "mo:itertools/Iter";

import CanDBIndex "canister:CanDBIndex";
import NacDBIndex "canister:NacDBIndex";
import CanDBPartition "../storage/CanDBPartition";
import MyCycles "mo:nacdb/Cycles";
import DBConfig "../libs/configs/db.config";
import lib "lib";
// import ICRC1Types "mo:icrc1/ICRC1/Types";

shared actor class ZonBackend() = this {
  /// External Canisters ///

  /// Some Global Variables ///
  stable let guidGen = GUID.init(Array.tabulate<Nat8>(16, func _ = 0)); // FIXME: Gather randomness.

  stable let orderer = Reorder.createOrderer({queueLengths = 20}); // TODO: What's the number?

  // See ARCHITECTURE.md for database structure

  // TODO: Avoid duplicate user nick names.

  stable var maxId: Nat = 0;

  stable var founder: ?Principal = null;

  /// Initialization ///

  stable var initialized: Bool = false;

  public shared({ caller }) func init(): async () {
    ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);

    if (initialized) {
      Debug.trap("already initialized");
    };

    founder := ?caller;

    initialized := true;
  };

  /// Owners ///

  func onlyMainOwner(caller: Principal) {
    if (?caller != founder) {
      Debug.trap("not the main owner");
    }
  };

  public shared({caller}) func setMainOwner(_founder: Principal) {
    onlyMainOwner(caller);

    founder := ?_founder;
  };

  // TODO: probably, superfluous.
  public shared({caller}) func removeMainOwner() {
    onlyMainOwner(caller);
    
    founder := null;
  };

  public shared({caller}) func setUserData(partitionId: ?Principal, user: lib.User) {
    let key = "u/" # Principal.toText(caller); // TODO: Should use binary encoding.
    // TODO: Add Hint to CanDBMulti
    ignore await CanDBIndex.putAttributeNoDuplicates("user", {
        sk = key;
        key = "u";
        value = lib.serializeUser(user);
      },
    );
  };

  // TODO: Should also remove all his/her items?
  public shared({caller}) func removeUser(canisterId: Principal) {
    var db: CanDBPartition.CanDBPartition = actor(Principal.toText(canisterId));
    let key = "u/" # Principal.toText(caller);
    await db.delete({sk = key});
  };

  /// Items ///

  stable var rootItem: ?(CanDBPartition.CanDBPartition, Nat) = null;

  public shared({caller}) func setRootItem(part: Principal, id: Nat)
    : async ()
  {
    onlyMainOwner(caller);

    rootItem := ?(actor(Principal.toText(part)), id);
  };

  public query func getRootItem(): async ?(Principal, Nat) {
    do ? {
      let (part, n) = rootItem!;
      (Principal.fromActor(part), n);
    };
  };

  public shared({caller}) func createItemData(item: lib.ItemTransferWithoutOwner)
    : async (Principal, Nat)
  {
    let (canisterId, itemId) = if (item.communal) {
      let variant: lib.ItemVariant = { creator = caller; item = item.data; };
      let variantId = maxId;
      maxId += 1;
      let variantKey = "r/" # Nat.toText(variantId);
      let variantCanisterId = await CanDBIndex.putAttributeWithPossibleDuplicate(
        "main", { sk = variantKey; key = "i"; value = lib.serializeItemVariant(variant) }
      );
      let itemId = maxId;
      maxId += 1;
      let itemKey = "i/" # Nat.toText(itemId);
      let timeStream = await* Reorder.createOrder(GUID.nextGuid(guidGen), NacDBIndex, orderer, ?10000); // FIXME: max length
      let votesStream = await* Reorder.createOrder(GUID.nextGuid(guidGen), NacDBIndex, orderer, ?10000); // FIXME: max length
      let item2 = #communal { timeStream; votesStream; isFolder = item.data.details == #folder };
      let variantValue = Nat.toText(variantId) # "@" # Principal.toText(variantCanisterId);
      await* Reorder.add(GUID.nextGuid(guidGen), NacDBIndex, orderer, {
        hardCap = ?100; key = -2; order = votesStream; value = variantValue; // TODO: Take position `key` configurable.
      });

      // Put variant in time stream // TODO: duplicate code
      let scanResult = await timeStream.order.0.scanLimitOuter({
        dir = #fwd;
        outerKey = timeStream.order.1;
        lowerBound = "";
        upperBound = "x";
        limit = 1;
        ascending = ?true;
      });
      let timeScanSK = if (scanResult.results.size() == 0) { // empty list
        0;
      } else {
        let t = scanResult.results[0].0;
        let n = lib.decodeInt(Text.fromIter(Itertools.takeWhile(t.chars(), func (c: Char): Bool { c != '#' })));
        n - 1;
      };
      let guid = GUID.nextGuid(guidGen);
      // TODO: race condition
      await* Reorder.add(guid, NacDBIndex, orderer, {
        order = timeStream;
        key = timeScanSK;
        value = variantValue;
        hardCap = DBConfig.dbOptions.hardCap;
      });

      let itemCanisterId = await CanDBIndex.putAttributeWithPossibleDuplicate(
        "main", { sk = itemKey; key = "i"; value = lib.serializeItem(item2) }
      );
      (itemCanisterId, itemId);
    } else {
      let item2: lib.Item = #owned { creator = caller; item = item.data; edited = false };
      let itemId = maxId;
      maxId += 1;
      let key = "i/" # Nat.toText(itemId);
      let canisterId = await CanDBIndex.putAttributeWithPossibleDuplicate(
        "main", { sk = key; key = "i"; value = lib.serializeItem(item2) }
      );
      (canisterId, itemId);
    };

    await order.insertIntoAllTimeStream((canisterId, itemId));
    (canisterId, itemId);
  };

  // We don't check that owner exists: If a user lost his/her item, that's his/her problem, not ours.
  public shared({caller}) func setItemData(canisterId: Principal, _itemId: Nat, item: lib.ItemDataWithoutOwner) {
    var db: CanDBPartition.CanDBPartition = actor(Principal.toText(canisterId));
    let key = "i/" # Nat.toText(_itemId); // TODO: better encoding
    switch (await db.getAttribute({sk = key}, "i")) {
      case (?oldItemRepr) {
        let oldItem = lib.deserializeItem(oldItemRepr);
        let item2: lib.ItemData = { item = item; creator = caller; edited = true }; // TODO: edited only if actually changed
        lib.onlyItemOwner(caller, oldItem); // also rejects changing communal items.
        await db.putAttribute({sk = key; key = "i"; value = lib.serializeItem(#owned item2)});
      };
      case null { Debug.trap("no item") };
    };
  };

  public shared({caller}) func setPostText(canisterId: Principal, _itemId: Nat, text: Text) {
    var db: CanDBPartition.CanDBPartition = actor(Principal.toText(canisterId));
    let key = "i/" # Nat.toText(_itemId); // TODO: better encoding
    switch (await db.getAttribute({sk = key}, "i")) {
      case (?oldItemRepr) {
        let oldItem = lib.deserializeItem(oldItemRepr);
        lib.onlyItemOwner(caller, oldItem);
        switch (oldItem) {
          case (#owned data) {
            switch (data.item.details) {
              case (#post) {};
              case _ { Debug.trap("not a post"); };
            };
          };
          case (#communal _) { Debug.trap("programming error") };
        };
        await db.putAttribute({ sk = key; key = "t"; value = #text(text) });
      };
      case _ { Debug.trap("no item") };
    };
  };

  // TODO: Also remove voting data.
  public shared({caller}) func removeItem(canisterId: Principal, _itemId: Nat) {
    // We first remove links, then the item itself, in order to avoid race conditions when displaying.
    await order.removeItemLinks((canisterId, _itemId));
    var db: CanDBPartition.CanDBPartition = actor(Principal.toText(canisterId));
    let key = "i/" # Nat.toText(_itemId);
    let ?oldItemRepr = await db.getAttribute({sk = key}, "i") else {
      Debug.trap("no item");
    };
    let oldItem = lib.deserializeItem(oldItemRepr);
    // if (oldItem.item.communal) { // FIXME
    //   Debug.trap("it's communal");
    // };
    lib.onlyItemOwner(caller, oldItem);
    await db.delete({sk = key});
  };

  // TODO: Set maximum lengths on user nick, chirp length, etc.

  /// Affiliates ///

  // public shared({caller}) func setAffiliate(canister: Principal, buyerAffiliate: ?Principal, sellerAffiliate: ?Principal): async () {
  //   var db: CanDBPartition.CanDBPartition = actor(Principal.toText(canister));
  //   if (buyerAffiliate == null and sellerAffiliate == null) {
  //     await db.delete({sk = "a/" # Principal.toText(caller)});
  //   };
  //   let buyerAffiliateStr = switch (buyerAffiliate) {
  //     case (?user) { Principal.toText(user) };
  //     case (null) { "" }
  //   };
  //   let sellerAffiliateStr = switch (sellerAffiliate) {
  //     case (?user) { Principal.toText(user) };
  //     case (null) { "" }
  //   };
  //   // await db.put({sk = "a/" # Principal.toText(caller); attributes = [("v", #text (buyerAffiliateStr # "/" # sellerAffiliateStr))]});
  // };

  public shared func get_trusted_origins(): async [Text] {
    return [];
  };
}
