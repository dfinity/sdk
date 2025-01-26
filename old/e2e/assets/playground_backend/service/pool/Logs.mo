import Map "mo:base/RBTree";
import {compare} "mo:base/Text";
import {toArray} "mo:base/Iter";
import {now = timeNow} "mo:base/Time";
import {toText} "mo:base/Int";
import {get} "mo:base/Option";

module {
    public type Origin = { origin: Text; tags: [Text] };
    public type SharedStatsByOrigin = (Map.Tree<Text,Nat>, Map.Tree<Text,Nat>);
    public class StatsByOrigin() {
        var canisters = Map.RBTree<Text, Nat>(compare);
        var installs = Map.RBTree<Text, Nat>(compare);
        public func share() : SharedStatsByOrigin = (canisters.share(), installs.share());
        public func unshare(x : SharedStatsByOrigin) {
            canisters.unshare(x.0);
            installs.unshare(x.1);
        };
        func addTags(map: Map.RBTree<Text,Nat>, list: [Text]) {
            for (tag in list.vals()) {
                switch (map.get(tag)) {
                case null { map.put(tag, 1) };
                case (?n) { map.put(tag, n + 1) };
                };
            };
        };
        // if to is null, delete the from tag
        func merge_tag_(map: Map.RBTree<Text,Nat>, from: Text, opt_to: ?Text) {
            ignore do ? {
                let n1 = map.remove(from)!;
                let to = opt_to!;
                switch (map.get(to)) {
                case null { map.put(to, n1) };
                case (?n2) { map.put(to, n1 + n2) };
                };
            };
        };
        public func merge_tag(from: Text, to: ?Text) {
            merge_tag_(canisters, from, to);
            merge_tag_(installs, from, to);
        };
        public func addCanister(origin: Origin) {
            addTags(canisters, ["origin:" # origin.origin]);
            addTags(canisters, origin.tags);
        };
        public func addInstall(origin: Origin) {
            addTags(installs, ["origin:" # origin.origin]);
            addTags(installs, origin.tags);
        };
        public func dump() : ([(Text, Nat)], [(Text, Nat)]) {
            (toArray<(Text, Nat)>(canisters.entries()),
             toArray<(Text, Nat)>(installs.entries()),
            )
        };
        public func metrics() : Text {
            var result = "";
            let now = timeNow() / 1_000_000;
            let canister_playground = get(canisters.get("origin:playground"), 0);
            let canister_dfx = get(canisters.get("origin:dfx"), 0);
            let install_playground = get(installs.get("origin:playground"), 0);
            let install_dfx = get(installs.get("origin:dfx"), 0);
            let profiling = get(installs.get("wasm:profiling"), 0);
            let asset = get(installs.get("wasm:asset"), 0);
            let install = get(installs.get("mode:install"), 0);
            let reinstall = get(installs.get("mode:reinstall"), 0);
            let upgrade = get(installs.get("mode:upgrade"), 0);
            result := result
            # encode_single_value("counter", "create_from_playground", canister_playground, "Number of canisters created from playground", now)
            # encode_single_value("counter", "install_from_playground", install_playground, "Number of Wasms installed from playground", now)
            # encode_single_value("counter", "create_from_dfx", canister_dfx, "Number of canisters created from dfx", now)
            # encode_single_value("counter", "install_from_dfx", install_dfx, "Number of Wasms installed from dfx", now)
            # encode_single_value("counter", "profiling", profiling, "Number of Wasms profiled", now)
            # encode_single_value("counter", "asset", asset, "Number of asset Wasms canister installed", now)
            # encode_single_value("counter", "install", install, "Number of Wasms with install mode", now)
            # encode_single_value("counter", "reinstall", reinstall, "Number of Wasms with reinstall mode", now)
            # encode_single_value("counter", "upgrade", upgrade, "Number of Wasms with upgrad mode", now);
            result;
        };
    };
    public func encode_single_value(kind: Text, name: Text, number: Int, desc: Text, time: Int) : Text {
        "# HELP " # name # " " # desc # "\n" #
        "# TYPE " # name # " " # kind # "\n" #
        name # " " # toText(number) # " " # toText(time) # "\n"
    };

    public type Stats = {
        num_of_canisters: Nat;
        num_of_installs: Nat;
        cycles_used: Nat;
        error_out_of_capacity: Nat;
        error_total_wait_time: Nat;
        error_mismatch: Nat;
    };
    public let defaultStats : Stats = {
        num_of_canisters = 0;
        num_of_installs = 0;
        cycles_used = 0;
        error_out_of_capacity = 0;
        error_total_wait_time = 0;
        error_mismatch = 0;
    };
    public type EventType = {
        #getId : Nat;
        #outOfCapacity : Nat;
        #install;
        #mismatch;
    };
    public func updateStats(stats: Stats, event: EventType) : Stats {
        switch (event) {
        case (#getId(cycles)) { {
                 num_of_canisters = stats.num_of_canisters + 1;
                 cycles_used = stats.cycles_used + cycles;
                 num_of_installs = stats.num_of_installs;
                 error_out_of_capacity = stats.error_out_of_capacity;
                 error_total_wait_time = stats.error_total_wait_time;
                 error_mismatch = stats.error_mismatch;
                                } };
        case (#outOfCapacity(time)) { {
                 num_of_canisters = stats.num_of_canisters;
                 cycles_used = stats.cycles_used;
                 num_of_installs = stats.num_of_installs;
                 error_out_of_capacity = stats.error_out_of_capacity + 1;
                 error_total_wait_time = stats.error_total_wait_time + time;                 
                 error_mismatch = stats.error_mismatch;
                                  } };
        case (#install) { {
                 num_of_canisters = stats.num_of_canisters;
                 cycles_used = stats.cycles_used;
                 num_of_installs = stats.num_of_installs + 1;
                 error_out_of_capacity = stats.error_out_of_capacity;
                 error_total_wait_time = stats.error_total_wait_time;
                 error_mismatch = stats.error_mismatch;
                          } };
        case (#mismatch) { {
                 num_of_canisters = stats.num_of_canisters;
                 cycles_used = stats.cycles_used;
                 num_of_installs = stats.num_of_installs;
                 error_out_of_capacity = stats.error_out_of_capacity; 
                 error_total_wait_time = stats.error_total_wait_time;
                 error_mismatch = stats.error_mismatch + 1;
                           } };
        };
    };
}
