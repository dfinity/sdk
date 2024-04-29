import Cycles "mo:base/ExperimentalCycles";
import Debug "mo:base/Debug";
import Text "mo:base/Text";
import CA "mo:candb/CanisterActions";
import Utils "mo:candb/Utils";
import CanisterMap "mo:candb/CanisterMap";
import Buffer "mo:stable-buffer/StableBuffer";
import CanDBPartition "CanDBPartition";
import Admin "mo:candb/CanDBAdmin";
import Principal "mo:base/Principal";
import Array "mo:base/Array";
import Time "mo:base/Time";
import CanDB "mo:candb/CanDB";
import Multi "mo:CanDBMulti/Multi";
import Entity "mo:candb/Entity";
import lib "../backend/lib";
import PassportConfig "../libs/configs/passport.config";

shared({caller = initialOwner}) actor class CanDBIndex() = this {
  stable var owners: [Principal] = [initialOwner];

  stable var initialized: Bool = false;

  public shared func init(_owners: [Principal]): async () {
    if (initialized) {
      Debug.trap("already initialized");
    };

    owners := _owners;
    ignore await* createStorageCanister("main", ownersOrSelf());
    ignore await* createStorageCanister("user", ownersOrSelf()); // user data

    initialized := true;
  };

  func checkCaller(caller: Principal) {
    if (Array.find(owners, func(e: Principal): Bool { e == caller; }) == null) {
      Debug.trap("CanDBIndex: not allowed");
    }
  };

  public shared({caller = caller}) func setOwners(_owners: [Principal]): async () {
    checkCaller(caller);

    owners := _owners;
  };

  public query func getOwners(): async [Principal] { owners };

  func ownersOrSelf(): [Principal] {
    let buf = Buffer.fromArray<Principal>(owners);
    Buffer.add(buf, Principal.fromActor(this));
    Buffer.toArray(buf);
  };

  let maxSize = #heapSize(500_000_000);

  stable var pkToCanisterMap = CanisterMap.init();

  /// @required API (Do not delete or change)
  ///
  /// Get all canisters for an specific PK
  ///
  /// This method is called often by the candb-client query & update methods. 
  public shared query({caller}) func getCanistersByPK(pk: Text): async [Text] {
    getCanisterIdsIfExists(pk);
  };
  
  /// @required function (Do not delete or change)
  ///
  /// Helper method acting as an interface for returning an empty array if no canisters
  /// exist for the given PK
  func getCanisterIdsIfExists(pk: Text): [Text] {
    switch(CanisterMap.get(pkToCanisterMap, pk)) {
      case null { [] };
      case (?canisterIdsBuffer) { Buffer.toArray(canisterIdsBuffer) } 
    }
  };

  /// This hook is called by CanDB for AutoScaling the User Service Actor.
  ///
  /// If the developer does not spin up an additional User canister in the same partition within this method, auto-scaling will NOT work
  /// Upgrade user canisters in a PK range, i.e. rolling upgrades (limit is fixed at upgrading the canisters of 5 PKs per call)
  public shared({caller}) func upgradeAllPartitionCanisters(wasmModule: Blob): async Admin.UpgradePKRangeResult {
    checkCaller(caller);

    await Admin.upgradeCanistersInPKRange({
      canisterMap = pkToCanisterMap;
      lowerPK = "";
      upperPK = "\u{FFFF}";
      limit = 5;
      wasmModule = wasmModule;
      scalingOptions = {
        autoScalingHook = autoScaleCanister;
        sizeLimit = maxSize;
      };
      owners = ?ownersOrSelf();
    });
  };

  public shared({caller}) func autoScaleCanister(pk: Text): async Text {
    checkCaller(caller);

    if (Utils.callingCanisterOwnsPK(caller, pkToCanisterMap, pk)) {
      await* createStorageCanister(pk, ownersOrSelf());
    } else {
      Debug.trap("error, called by non-controller=" # debug_show(caller));
    };
  };

  func createStorageCanister(pk: Text, controllers: [Principal]): async* Text {
    Debug.print("creating new storage canister with pk=" # pk);
    // Pre-load 300 billion cycles for the creation of a new storage canister
    // Note that canister creation costs 100 billion cycles, meaning there are 200 billion
    // left over for the new canister when it is created
    Cycles.add(210_000_000_000); // TODO: Choose the number.
    let newStorageCanister = await CanDBPartition.CanDBPartition({
      partitionKey = pk;
      scalingOptions = {
        autoScalingHook = autoScaleCanister;
        sizeLimit = maxSize;
      };
      owners = ?controllers;
    });
    let newStorageCanisterPrincipal = Principal.fromActor(newStorageCanister);
    await CA.updateCanisterSettings({
      canisterId = newStorageCanisterPrincipal;
      settings = {
        controllers = ?controllers;
        compute_allocation = ?0;
        memory_allocation = ?0;
        freezing_threshold = ?2592000;
      }
    });

    let newStorageCanisterId = Principal.toText(newStorageCanisterPrincipal);
    pkToCanisterMap := CanisterMap.add(pkToCanisterMap, pk, newStorageCanisterId);

    Debug.print("new storage canisterId=" # newStorageCanisterId);
    newStorageCanisterId;
  };

  // Private functions for getting canisters //

  func lastCanister(pk: Entity.PK): async* CanDBPartition.CanDBPartition {
    let canisterIds = getCanisterIdsIfExists(pk);
    let part0 = if (canisterIds == []) {
      await* createStorageCanister(pk, ownersOrSelf());
    } else {
      canisterIds[canisterIds.size() - 1];
    };
    actor(part0);
  };

  func getExistingCanister(pk: Entity.PK, options: CanDB.GetOptions, hint: ?Principal): async* ?CanDBPartition.CanDBPartition {
    switch (hint) {
      case (?hint) {
        let canister: CanDBPartition.CanDBPartition = actor(Principal.toText(hint));
        if (await canister.skExists(options.sk)) {
          return ?canister;
        } else {
          Debug.trap("wrong DB partition hint");
        };
      };
      case null {};
    };

    // Do parallel search in existing canisters:
    let canisterIds = getCanisterIdsIfExists(pk);
    let threads : [var ?(async())] = Array.init(canisterIds.size(), null);
    var foundInCanister: ?Nat = null;
    for (threadNum in threads.keys()) {
      threads[threadNum] := ?(async {
        let canister: CanDBPartition.CanDBPartition = actor(canisterIds[threadNum]);
        switch (foundInCanister) {
          case (?foundInCanister) {
            if (foundInCanister < threadNum) {
              return; // eliminate unnecessary work.
            };
          };
          case null {};
        };
        if (await canister.skExists(options.sk)) {
          foundInCanister := ?threadNum;
        };
      });
    };
    for (topt in threads.vals()) {
      let ?t = topt else {
        Debug.trap("programming error: threads");
      };
      await t;
    };

    switch (foundInCanister) {
      case (?foundInCanister) {
        ?(actor(canisterIds[foundInCanister]): CanDBPartition.CanDBPartition);
      };
      case null {
        let newStorageCanisterId = await* createStorageCanister(pk, ownersOrSelf());
        ?(actor(newStorageCanisterId): CanDBPartition.CanDBPartition);
      };
    };
  };

  // CanDBMulti //

  public shared({caller}) func getFirstAttribute(
    pk: Text,
    options: { sk: Entity.SK; key: Entity.AttributeKey }
  ) : async ?(Principal, ?Entity.AttributeValue) {
    await* Multi.getFirstAttribute(pkToCanisterMap, pk, options);
  };

  public shared({caller}) func putAttributeNoDuplicates(
      pk: Text,
      options: { sk: Entity.SK; key: Entity.AttributeKey; value: Entity.AttributeValue }
  ) : async Principal {
    checkCaller(caller);

    await* Multi.putAttributeNoDuplicates(pkToCanisterMap, pk, options);
  };

  public shared({caller}) func putAttributeWithPossibleDuplicate(
    pk: Text,
    options: { sk: Entity.SK; key: Entity.AttributeKey; value: Entity.AttributeValue }
  ) : async Principal {
    await* Multi.putAttributeWithPossibleDuplicate(pkToCanisterMap, pk, options);
  };

  func setVotingDataImpl(user: Principal, partitionId: ?Principal, voting: lib.VotingScore): async* () {
    let sk = "u/" # Principal.toText(user); // TODO: Should use binary encoding.
    // TODO: Add Hint to CanDBMulti
    ignore await* Multi.putAttributeNoDuplicates(pkToCanisterMap, "user", {
      sk;
      key = "v";
      value = lib.serializeVoting(voting);
    });
  };

  public shared({caller}) func setVotingData(user: Principal, partitionId: ?Principal, voting: lib.VotingScore): async () {
    checkCaller(caller); // necessary
    await* setVotingDataImpl(user, partitionId, voting);
  };

  // public shared({caller}) func getUser(userId: Principal, partitionId: ?Principal): async ?lib.User {
  //   let sk = "u/" # Principal.toText(userId); // TODO: Should use binary encoding.
  //   // TODO: Add Hint to CanDBMulti
  //   let res = await* Multi.getAttributeByHint(pkToCanisterMap, "user", partitionId, {sk; key = "u"});
  //   do ? { lib.deserializeUser(res!.1!) };
  // };

  // TODO: Here should be a shared function:
  func getVotingData(caller: Principal, partitionId: ?Principal): async* ?lib.VotingScore {
    let sk = "u/" # Principal.toText(caller); // TODO: Should use binary encoding.
    // TODO: Add Hint to CanDBMulti
    let res = await* Multi.getAttributeByHint(pkToCanisterMap, "user", partitionId, {sk; key = "v"});
    do ? { lib.deserializeVoting(res!.1!) };
  };

  func sybilScoreImpl(user: Principal): async* (Bool, Float) {
    // checkCaller(user); // TODO: enable?

    let voting = await* getVotingData(user, null); // TODO: hint `partitionId`, not null
    switch (voting) {
      case (?voting) {
        Debug.print("VOTING: " # debug_show(voting));
        if (voting.lastChecked + 150 * 24 * 3600 * 1_000_000_000 >= Time.now() and // TODO: Make configurable.
          voting.points >= PassportConfig.minimumScore)
        {
          (true, voting.points);
        } else {
          (false, 0.0);
        };
      };
      case null { (false, 0.0) };
    };
  };

  public shared({caller}) func sybilScore(): async (Bool, Float) {
    await* sybilScoreImpl(caller);
  };

  public shared func checkSybil(user: Principal): async () {
    // checkCaller(user); // TODO: enable?
    if (PassportConfig.skipSybil) {
      return;
    };
    let (allowed, score) = await* sybilScoreImpl(user);
    if (not allowed) {
      Debug.trap("Sybil check failed");
    };
  };
}