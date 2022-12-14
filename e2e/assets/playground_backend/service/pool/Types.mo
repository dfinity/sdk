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

module {
    public type InitParams = {
        cycles_per_canister: Nat;
        max_num_canisters: Nat;
        canister_time_to_live: Nat;
        nonce_time_to_live: Nat;
        max_family_tree_size: Nat;
    };
    public let defaultParams : InitParams = {
        cycles_per_canister = 550_000_000_000;
        max_num_canisters = 100;
        canister_time_to_live = 1200_000_000_000;
        nonce_time_to_live = 300_000_000_000;
        max_family_tree_size = 5;
    };
    public type InstallArgs = {
        arg : Blob;
        wasm_module : Blob;
        mode : { #reinstall; #upgrade; #install };
        canister_id : Principal;
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

    /*
    * Main data structure of the playground. The splay tree is the source of truth for
    * what canisters live in the playground. Metadata map reflects the state of the tree
    * to allow Map-style lookups on the canister data. Childrens and parents define the
    * controller relationships for dynmically spawned canisters by actor classes.
    */
    public class CanisterPool(size: Nat, ttl: Nat, max_family_tree_size: Nat) {
        var len = 0;
        var tree = Splay.Splay<CanisterInfo>(canisterInfoCompare);
        var metadata = TrieMap.TrieMap<Principal, (Int, Bool)>(Principal.equal, Principal.hash);
        var childrens = TrieMap.TrieMap<Principal, List.List<Principal>>(Principal.equal, Principal.hash);
        var parents = TrieMap.TrieMap<Principal, Principal>(Principal.equal, Principal.hash);

        public type NewId = { #newId; #reuse:CanisterInfo; #outOfCapacity:Nat };

        public func getExpiredCanisterId() : NewId {
            if (len < size) {
                #newId
            } else {
                switch (tree.entries().next()) {
                    case null { assert false; loop(); };
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
                            #outOfCapacity(ttl - elapsed)
                        }
                     };
                };
            };
        };

        public func add(info: CanisterInfo) {
            if (len >= size) {
                assert false;
            };
            len += 1;
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
            return true;
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

        public func share() : ([CanisterInfo], [(Principal, (Int, Bool))], [(Principal, [Principal])]) {
            let stableInfos = Iter.toArray(tree.entries());
            let stableMetadata = Iter.toArray(metadata.entries());
            let stableChildrens = 
                Iter.toArray(
                    Iter.map<(Principal, List.List<Principal>), (Principal, [Principal])>(
                        childrens.entries(),
                        func((parent, children)) = (parent, List.toArray(children))
                    )
                );
            (stableInfos, stableMetadata, stableChildrens)
        };

        public func unshare(stableInfos: [CanisterInfo], stableMetadata: [(Principal, (Int, Bool))], stableChildrens : [(Principal, [Principal])]) {
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
        };

        public func getChildren(parent: Principal) : List.List<Principal> {
            switch(childrens.get parent) {
                case null List.nil();
                case (?children) {
                    let now = Time.now();
                    List.filter(children, func(p: Principal) : Bool = notExpired(Option.unwrap(info p), now));
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
