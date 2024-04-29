import BTree "mo:stableheapbtreemap/BTree";
import Nac "mo:nacdb/NacDB";
import Principal "mo:base/Principal";
import Bool "mo:base/Bool";
import Nat "mo:base/Nat";
import MyCycles "mo:nacdb/Cycles";
import Text "mo:base/Text";
import Debug "mo:base/Debug";
import Array "mo:base/Array";
import Iter "mo:base/Iter";
import DBConfig "../libs/configs/db.config";

shared({caller}) actor class Partition(
    initialOwners: [Principal],
) = this {
    stable var owners = initialOwners;

    func checkCaller(caller: Principal) {
        if (caller == Principal.fromActor(this)) {
            return;
        };
        if (Array.find(owners, func(e: Principal): Bool { e == caller; }) == null) {
            Debug.trap("NacDBPartition: not allowed from " # Principal.toText(caller));
        }
    };

    public shared({caller = caller}) func setOwners(_owners: [Principal]): async () {
        checkCaller(caller);

        owners := _owners;
    };

    public query func getOwners(): async [Principal] { owners };

    // func ownersOrSelf(): [Principal] {
    //     let buf = Buffer.fromArray<Principal>(owners);
    //     Buffer.add(buf, Principal.fromActor(this));
    //     Buffer.toArray(buf);
    // };
    
    stable let index: Nac.IndexCanister = actor(Principal.toText(caller));

    stable let superDB = Nac.createSuperDB(DBConfig.dbOptions);

    // Mandatory methods //

    public shared({caller}) func rawInsertSubDB({
        hardCap : ?Nat;
        innerKey : ?Nac.InnerSubDBKey;
        map : [(Nac.SK, Nac.AttributeValue)];
        userData : Text
    })
        : async {innerKey: Nac.InnerSubDBKey}
    {
        checkCaller(caller);

        ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        Nac.rawInsertSubDB({superDB; map; innerKey; userData; hardCap = DBConfig.dbOptions.hardCap});
    };

    public shared({caller}) func rawInsertSubDBAndSetOuter({
        map: [(Nac.SK, Nac.AttributeValue)];
        keys: ?{
            innerKey: Nac.InnerSubDBKey;
            outerKey: Nac.OuterSubDBKey;
        };
        userData: Text;
        hardCap: ?Nat;
    })
        : async {innerKey: Nac.InnerSubDBKey; outerKey: Nac.OuterSubDBKey}
    {
        checkCaller(caller);

        ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        Nac.rawInsertSubDBAndSetOuter({superDB; canister = this; map; keys; userData; hardCap});
    };

    public query func isOverflowed() : async Bool {
        Nac.isOverflowed({dbOptions = DBConfig.dbOptions; superDB});
    };

    // Some data access methods //

    public query func superDBSize() : async Nat {
        // ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        Nac.superDBSize(superDB);
    };

    public shared({caller}) func deleteSubDBInner({innerKey: Nac.InnerSubDBKey}) : async () {
        checkCaller(caller);

        ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        await* Nac.deleteSubDBInner({superDB; innerKey});
    };

    public shared func putLocation({outerKey: Nac.OuterSubDBKey; innerCanister: Principal; newInnerSubDBKey: Nac.InnerSubDBKey}) : async () {
        checkCaller(caller);

        ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        let inner2: Nac.InnerCanister = actor(Principal.toText(innerCanister));
        Nac.putLocation({outerSuperDB = superDB; outerKey; innerCanister = inner2; innerKey = newInnerSubDBKey});
    };

    public shared func createOuter({part: Principal; outerKey: Nac.OuterSubDBKey; innerKey: Nac.InnerSubDBKey})
        : async {inner: {canister: Principal; key: Nac.InnerSubDBKey}; outer: {canister: Principal; key: Nac.OuterSubDBKey}}
    {
        checkCaller(caller);

        ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        let part2: Nac.PartitionCanister = actor(Principal.toText(part));
        let { inner; outer } = Nac.createOuter({outerSuperDB = superDB; part = part2; outerKey; innerKey});
        {
            inner = {canister = Principal.fromActor(inner.canister); key = inner.key};
            outer = {canister = Principal.fromActor(outer.canister); key = outer.key};
        };
    };

    public shared({caller}) func deleteInner({innerKey: Nac.InnerSubDBKey; sk: Nac.SK}): async () {
        checkCaller(caller);

        ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        await* Nac.deleteInner({innerSuperDB = superDB; innerKey; sk});
    };

    public query func scanLimitInner({innerKey: Nac.InnerSubDBKey; lowerBound: Nac.SK; upperBound: Nac.SK; dir: BTree.Direction; limit: Nat})
        : async BTree.ScanLimitResult<Text, Nac.AttributeValue>
    {
        // ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        Nac.scanLimitInner({innerSuperDB = superDB; innerKey; lowerBound; upperBound; dir; limit});
    };

    public shared func scanLimitOuter({outerKey: Nac.OuterSubDBKey; lowerBound: Nac.SK; upperBound: Nac.SK; dir: BTree.Direction; limit: Nat})
        : async BTree.ScanLimitResult<Text, Nac.AttributeValue>
    {
        // ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        await* Nac.scanLimitOuter({outerSuperDB = superDB; outerKey; lowerBound; upperBound; dir; limit});
    };

    public query func scanSubDBs(): async [(Nac.OuterSubDBKey, {canister: Principal; key: Nac.InnerSubDBKey})] {
        // ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        type T1 = (Nac.OuterSubDBKey, Nac.InnerPair);
        type T2 = (Nac.OuterSubDBKey, {canister: Principal; key: Nac.InnerSubDBKey});
        let array: [T1] = Nac.scanSubDBs({superDB});
        let iter = Iter.map(array.vals(), func ((outerKey, {canister = inner; key = innerKey}): T1): T2 {
            (outerKey, {canister = Principal.fromActor(inner); key = innerKey});
        });
        Iter.toArray(iter);
    };

    public query func getByInner({innerKey: Nac.InnerSubDBKey; sk: Nac.SK}): async ?Nac.AttributeValue {
        // ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        Nac.getByInner({superDB; innerKey; sk});
    };

    public query func hasByInner({innerKey: Nac.InnerSubDBKey; sk: Nac.SK}): async Bool {
        // ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        Nac.hasByInner({superDB; innerKey; sk});
    };

    public shared func getByOuter({outerKey: Nac.OuterSubDBKey; sk: Nac.SK}): async ?Nac.AttributeValue {
        ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        await* Nac.getByOuter({outerSuperDB = superDB; outerKey; sk});
    };

    public shared func hasByOuter({outerKey: Nac.OuterSubDBKey; sk: Nac.SK}): async Bool {
        ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        await* Nac.hasByOuter({outerSuperDB = superDB; outerKey; sk});
    };

    public shared func hasSubDBByOuter(options: {outerKey: Nac.OuterSubDBKey}): async Bool {
        ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        await* Nac.hasSubDBByOuter({outerSuperDB = superDB; outerKey = options.outerKey});
    };

    public query func hasSubDBByInner(options: {innerKey: Nac.InnerSubDBKey}): async Bool {
        // ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        Nac.hasSubDBByInner({innerSuperDB = superDB; innerKey = options.innerKey});
    };

    public shared func subDBSizeByOuter({outerKey: Nac.OuterSubDBKey}): async ?Nat {
        ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        await* Nac.subDBSizeByOuter({outerSuperDB = superDB; outerKey});
    };

    public query func subDBSizeByInner({innerKey: Nac.InnerSubDBKey}): async ?Nat {
        // ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        Nac.subDBSizeByInner({superDB; innerKey});
    };

    public shared func startInsertingImpl({
        innerKey: Nac.InnerSubDBKey;
        sk: Nac.SK;
        value: Nac.AttributeValue;
    }): async () {
        checkCaller(caller);

        ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        await* Nac.startInsertingImpl({
            innerKey;
            sk;
            value;
            innerSuperDB = superDB;
        });
    };

    // TODO: These...
    public shared func getSubDBUserDataOuter(options: Nac.GetUserDataOuterOptions) : async ?Text {
        await* Nac.getSubDBUserDataOuter(options, DBConfig.dbOptions);
    };

    // TODO: .., two functions should have similar arguments
    public func getSubDBUserDataInner(options: {innerKey: Nac.InnerSubDBKey}) : async ?Text {
        Nac.getSubDBUserDataInner({superDB; subDBKey = options.innerKey});
    };

    // TODO: Add this function to the public interface in NacDB?
    public query func getInner(options: {outerKey: Nac.OuterSubDBKey}) : async ?{canister: Principal; key: Nac.InnerSubDBKey} {
        do ? {
            let {canister; key} = Nac.getInner({superDB; outerKey = options.outerKey})!;
            {canister = Principal.fromActor(canister); key};
        };
    };

    public shared({caller}) func deleteSubDBOuter({outerKey: Nac.OuterSubDBKey}) : async () {
        checkCaller(caller);
        await* Nac.deleteSubDBOuter({superDB; outerKey});
    };

    public shared func rawDeleteSubDB({innerKey: Nac.InnerSubDBKey}): async () {
        checkCaller(caller);

        ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        Nac.rawDeleteSubDB(superDB, innerKey);
    };

    public query func rawGetSubDB({innerKey: Nac.InnerSubDBKey}): async ?{map: [(Nac.SK, Nac.AttributeValue)]; userData: Text} {
        // ignore MyCycles.topUpCycles<system>(DBConfig.dbOptions.partitionCycles);
        Nac.rawGetSubDB(superDB, innerKey);
    };

    public func subDBSizeOuterImpl(options: Nac.SubDBSizeOuterOptions): async ?Nat {
        MyCycles.addPart<system>(DBConfig.dbOptions.partitionCycles);
        await options.outer.canister.subDBSizeByOuter({outerKey = options.outer.key});
    };

    public shared func getOuter(options: Nac.GetByOuterPartitionKeyOptions): async ?Nac.AttributeValue {
        await* Nac.getOuter(options, DBConfig.dbOptions);
    };

    // TODO: Remove superfluous functions from above.
}