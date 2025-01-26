import Principal "mo:base/Principal";
import List "mo:base/List";
import Array "mo:base/Array";
import Debug "mo:base/Debug";

actor FakeRegistry {

    // list of (canister id -> subnet id) mappings
    var subnet_per_canister : [(Principal, Principal)] = [];

    public func set_subnet_for_canister(mappings : [(Principal, Principal)]) {
        subnet_per_canister := mappings;
    };

    public query func get_subnet_for_canister(arg : { principal : ?Principal }) : async ({
        #Ok : { subnet_id : ?Principal };
        #Err : Text;
    }) {
        switch (Array.find<(Principal, Principal)>(subnet_per_canister, func pair { ?pair.0 == arg.principal })) {
            case (null) { #Err("mapping not defined") };
            case (?mapping) { #Ok { subnet_id = ?mapping.1 } };
        };
    };

};
