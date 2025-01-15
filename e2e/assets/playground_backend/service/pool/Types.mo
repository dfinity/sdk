import Principal "mo:base/Principal";
import Splay "mo:splay";
import Time "mo:base/Time";
import Buffer "mo:base/Buffer";
import TrieMap "mo:base/TrieMap";
import TrieSet "mo:base/TrieSet";
import Iter "mo:base/Iter";
import Array "mo:base/Array";
import List "mo:base/List";
import Option "mo:base/Option";
import Int "mo:base/Int";
import Timer "mo:base/Timer";
import Debug "mo:base/Debug";
import Error "mo:base/Error";
import Cycles "mo:base/ExperimentalCycles";
import ICType "./IC";

module {
    type CyclesSettings = {
        max_cycles_per_call: Nat;
        max_cycles_total: Nat;
    };
    public type InitParams = {
        cycles_per_canister: Nat;
        max_num_canisters: Nat;
        canister_time_to_live: Nat;
        nonce_time_to_live: Nat;
        max_family_tree_size: Nat;
        // Used for installing asset canister. If set, will not use timer to kill expired canisters, and will not uninstall code when fetching an expired canister (unless the module hash changed).
        stored_module: ?{hash: Blob; arg: Blob};
        // Disable getCanisterId endpoint
        admin_only: ?Bool;
        // Cycle add limit for whitelisted methods
        cycles_settings: ?CyclesSettings;
        wasm_utils_principal: ?Text;
    };
    public let defaultParams : InitParams = {
        cycles_per_canister = 550_000_000_000;
        max_num_canisters = 100;
        canister_time_to_live = 1200_000_000_000;
        nonce_time_to_live = 300_000_000_000;
        max_family_tree_size = 5;
        stored_module = null;
        admin_only = null;
        cycles_settings = null;
        wasm_utils_principal = ?"ozk6r-tyaaa-aaaab-qab4a-cai";
    };
    public type InstallArgs = {
        arg : Blob;
        wasm_module : Blob;
        mode : ICType.canister_install_mode;
        canister_id : Principal;
    };
    public type DeployArgs = {
        arg : Blob;
        wasm_module : Blob;
        bypass_wasm_transform : ?Bool;
        mode : ?ICType.canister_install_mode;
    };
    public type InstallConfig = {
        profiling: Bool;
        is_whitelisted: Bool;
        origin: { origin: Text; tags: [Text] };
        start_page: ?Nat32;
        page_limit: ?Nat32;
    };
    public type ProfilingConfig = {
        start_page: ?Nat32;
        page_limit: ?Nat32;
    };
    public type CanisterInfo = {
        id: Principal;
        timestamp: Int;
    };
    func canisterInfoCompare(x: CanisterInfo, y: CanisterInfo): {#less;#equal;#greater} {
        if (x.timestamp < y.timestamp) { #less }
        else if (x.timestamp == y.timestamp and x.id < y.id) { #less }
        else if (x.timestamp == y.timestamp and x.id == y.id) { #equal }
        else { #greater }
    };
    public func getCyclesSettings(params: InitParams) : CyclesSettings {
        Option.get(params.cycles_settings, { max_cycles_per_call = 250_000_000_000; max_cycles_total = 550_000_000_000 })
    };

    /*
    * Main data structure of the playground. The splay tree is the source of truth for
    * what canisters live in the playground. Metadata map reflects the state of the tree
    * to allow Map-style lookups on the canister data. Childrens and parents define the
    * controller relationships for dynmically spawned canisters by actor classes.
    */
    public class CanisterPool(params: InitParams) {
        let size = params.max_num_canisters;
        let ttl = params.canister_time_to_live;
        let max_family_tree_size = params.max_family_tree_size;
        var len = 0;
        var tree = Splay.Splay<CanisterInfo>(canisterInfoCompare);
        // Metadata is a replicate of splay tree, which allows lookup without timestamp. Internal use only.
        var metadata = TrieMap.TrieMap<Principal, (Int, Bool)>(Principal.equal, Principal.hash);
        var childrens = TrieMap.TrieMap<Principal, List.List<Principal>>(Principal.equal, Principal.hash);
        var parents = TrieMap.TrieMap<Principal, Principal>(Principal.equal, Principal.hash);
        let timers = TrieMap.TrieMap<Principal, Timer.TimerId>(Principal.equal, Principal.hash);
        var snapshots = TrieMap.TrieMap<Principal, Blob>(Principal.equal, Principal.hash);
        // Cycles spent by each canister, not persisted for upgrades
        let cycles = TrieMap.TrieMap<Principal, Int>(Principal.equal, Principal.hash);

        public type NewId = { #newId; #reuse:CanisterInfo; #outOfCapacity:Nat };

        public func rollbackLen() {
            len -= 1;
        };
        public func getExpiredCanisterId() : NewId {
            switch (tree.entries().next()) {
            case null {
                     if (len < size) {
                         len += 1;
                         #newId
                     } else {
                         Debug.trap "No canister in the pool"
                     };
                 };
            case (?info) {
                     let now = Time.now();
                     let elapsed : Nat = Int.abs(now) - Int.abs(info.timestamp);
                     if (elapsed >= ttl) {
                         // Lazily cleanup pool state before reusing canister
                         tree.remove info;
                         let newInfo = { timestamp = now; id = info.id; };
                         tree.insert newInfo;
                         metadata.put(newInfo.id, (newInfo.timestamp, false));
                         deleteFamilyNode(newInfo.id);
                         #reuse newInfo
                     } else {
                         if (len < size) {
                             len += 1;
                             #newId
                         } else {
                             #outOfCapacity(ttl - elapsed)
                         }
                     }
                 };
            };
        };
        public func removeCanister(info: CanisterInfo) {
            tree.remove info;
            metadata.delete(info.id);
            deleteFamilyNode(info.id);
            cycles.delete(info.id);
            // Note that we didn't remove snapshots, as users can continue to use them after the transfer
            switch (timers.remove(info.id)) {
                case null {};
                case (?tid) {
                    Timer.cancelTimer(tid);
                };
            };
            len -= 1;
        };

        public func add(info: CanisterInfo) {
            if (len > size) {
                assert false;
            };
            // len already incremented in getExpiredCanisterId
            tree.insert info;
            metadata.put(info.id, (info.timestamp, false));
        };

        public func find(info: CanisterInfo) : Bool = tree.find info;
        public func findId(id: Principal) : Bool = Option.isSome(metadata.get id);
        public func profiling(id: Principal) : Bool = Option.getMapped<(Int, Bool), Bool>(metadata.get id, func p = p.1, false);

        public func info(id: Principal) : ?CanisterInfo {
            do ? {
                let (timestamp, _) = metadata.get(id)!;
                { timestamp; id }
            }
        };

        public func refresh(info: CanisterInfo, profiling: Bool) : ?CanisterInfo {
            if (not tree.find info) { return null };
            tree.remove info;
            let newInfo = { timestamp = Time.now(); id = info.id };
            tree.insert newInfo;
            metadata.put(newInfo.id, (newInfo.timestamp, profiling));
            ?newInfo
        };

        public func retire(info: CanisterInfo) : Bool {
            if (not tree.find info) {
                return false;
            };
            let id = info.id;
            tree.remove info;
            tree.insert { timestamp = 0; id };
            metadata.put(id, (0, false));
            deleteFamilyNode id;
            cycles.delete id;
            // snapshots already removed with pool_uninstall_code
            return true;
        };

        public func updateTimer<system>(info: CanisterInfo, job : () -> async ()) {
            let elapsed = Time.now() - info.timestamp;
            let duration = if (elapsed > ttl) { 0 } else { Int.abs(ttl - elapsed) };
            let tid = Timer.setTimer<system>(#nanoseconds duration, job);
            switch (timers.replace(info.id, tid)) {
            case null {};
            case (?old_id) {
                     // The old job can still run when it has expired, but the future
                     // just started to run. To be safe, the job needs to check for timestamp.
                     Timer.cancelTimer(old_id);
                 };
            };
        };
        
        public func removeTimer(cid: Principal) {
            timers.delete cid;
        };
        public func getSnapshot(cid: Principal) : ?Blob {
            snapshots.get cid
        };
        public func setSnapshot(cid: Principal, snapshot: Blob) {
            snapshots.put(cid, snapshot);
        };
        public func removeSnapshot(cid: Principal) {
            snapshots.delete cid;
        };
        public func addCycles<system>(cid: Principal, config: { #method: Text; #refund }) : async* () {
            switch (config) {
            case (#method(method)) {
                     if (not findId cid) {
                         throw Error.reject("Canister pool: Only a canister managed by the pool can call " # method);
                     };
                     let curr = Option.get(cycles.get(cid), 0);
                     let settings = getCyclesSettings(params);
                     let new = curr + settings.max_cycles_per_call;
                     if (new > settings.max_cycles_total) {
                         throw Error.reject("Canister pool: Cycles limit exceeded when calling " # method # ". Already used " # Int.toText(curr) # " cycles. Deploy with your own wallet to avoid cycle limit.");
                     };
                     cycles.put(cid, new);
                     Cycles.add<system>(settings.max_cycles_per_call);
                 };
            case (#refund) {
                     let refund = Cycles.refunded();
                     let curr = Option.get(cycles.get(cid), 0);
                     let new = curr - refund;
                     if (new < 0) {
                         throw Error.reject("Canister pool: Cycles refund exceeds the balance. This should not happen.");
                     };
                     cycles.put(cid, new);
                 };
            };
        };
        private func notExpired(info: CanisterInfo, now: Int) : Bool = (info.timestamp > now - ttl);

        // Return a list of canister IDs from which to uninstall code
        public func gcList() : Buffer.Buffer<Principal> {
            let now = Time.now();
            let result = Buffer.Buffer<Principal>(len);
            for (info in tree.entries()) {
                if (info.timestamp > 0) {
                    // assumes when timestamp == 0, uninstall_code is already done
                    if (notExpired(info, now)) { return result };
                    result.add(info.id);
                    ignore retire info;
                }
            };
            result
        };
        public func getAllCanisters() : Iter.Iter<CanisterInfo> {
            tree.entries();
        };

        public func share() : ([CanisterInfo], [(Principal, (Int, Bool))], [(Principal, [Principal])], [CanisterInfo], [(Principal, Blob)]) {
            let stableInfos = Iter.toArray(tree.entries());
            let stableMetadata = Iter.toArray(metadata.entries());
            let stableChildren = 
                Iter.toArray(
                    Iter.map<(Principal, List.List<Principal>), (Principal, [Principal])>(
                        childrens.entries(),
                        func((parent, children)) = (parent, List.toArray(children))
                    )
                );
            let stableTimers = Iter.toArray(
              Iter.filter<CanisterInfo>(
                tree.entries(),
                func (info) = Option.isSome(timers.get(info.id))
              ));
            let stableSnapshots = Iter.toArray(snapshots.entries());
            (stableInfos, stableMetadata, stableChildren, stableTimers, stableSnapshots)
        };

        public func unshare(stableInfos: [CanisterInfo], stableMetadata: [(Principal, (Int, Bool))], stableChildrens : [(Principal, [Principal])], stableSnapshots: [(Principal, Blob)]) {
            len := stableInfos.size();
            tree.fromArray stableInfos;

            // Ensure that metadata reflects tree
            let profilingMap = TrieMap.fromEntries<Principal, (Int, Bool)>(Iter.fromArray stableMetadata, Principal.equal, Principal.hash);
            Iter.iterate<CanisterInfo>(
                stableInfos.vals(),
                func(info, _) {
                    let profiling = Option.getMapped<(Int, Bool), Bool>(profilingMap.get(info.id), func p = p.1, false);
                    metadata.put(info.id, (info.timestamp, profiling));
                    }
                );

            childrens := 
                TrieMap.fromEntries(
                    Array.map<(Principal, [Principal]), (Principal, List.List<Principal>)>(
                        stableChildrens,
                        func((parent, children)) = (parent, List.fromArray children)
                    ).vals(), 
                    Principal.equal,
                    Principal.hash
                );
            
            let parentsEntries = 
                Array.flatten(
                    Array.map<(Principal, [Principal]), [(Principal, Principal)]>(
                        stableChildrens, 
                        func((parent, children)) = 
                            Array.map<Principal, (Principal, Principal)>(
                                children,
                                func child = (child, parent)
                            )
                    )
                );
            parents := TrieMap.fromEntries(parentsEntries.vals(), Principal.equal, Principal.hash);
            snapshots := TrieMap.fromEntries(stableSnapshots.vals(), Principal.equal, Principal.hash);
        };

        public func getChildren(parent: Principal) : List.List<Principal> {
            switch(childrens.get parent) {
                case null List.nil();
                case (?children) {
                    let now = Time.now();
                    List.filter(children, func(p: Principal) : Bool {
                        let ?cinfo = info p else { Debug.trap "unwrap info(p)" };
                        notExpired(cinfo, now);
                    });
                }
            }
        };

        public func isRoot(node: Principal) : Bool = Option.isNull(parents.get node);

        private func treeSize(node: Principal) : Nat {
            switch (parents.get node) {
                // found root
                case null {
                    countActiveNodes(node)
                };
                case (?parent) {
                    treeSize(parent)
                }
            }
        };

        // Counts number of nodes in the tree rooted at root, excluding expired nodes at time `now
        private func countActiveNodes(root: Principal) : Nat {
            var count = 1;
            let now = Time.now();
            ignore do ? {
                let children = childrens.get(root)!;
                for (child in List.toIter(children)) {
                    if (notExpired((info child)!, now)) {
                        count := count + countActiveNodes(child)
                    }
                };
            };
            count
        };

        public func setChild(parent: Principal, child: Principal) : Bool {
            if (treeSize(parent) >= max_family_tree_size) {
                return false;
            };
            let children = getChildren parent;
            childrens.put(parent, List.push(child, children));
            parents.put(child, parent);
            return true;
        };

        public func isParentOf(parent: Principal, child: Principal) : Bool {
            switch(parents.get child) {
                case null {
                    false
                };
                case (?registerdParent) {
                    Principal.equal(registerdParent, parent)
                };
            };
        };

        private func deleteFamilyNode(id: Principal) {
            // Remove children edges
            ignore do ? {
                List.iterate(childrens.get(id)!, parents.delete);
            };
            childrens.delete id;

            // Remove parent edges
            ignore do ? {
                let parent = parents.get(id)!;
                childrens.put(parent, List.filter<Principal>(childrens.get(parent)!, func child = not Principal.equal(child, id)));
            };
            parents.delete id;
        };
    };
}
