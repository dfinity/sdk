import Cycles "mo:base/ExperimentalCycles";
import InternetComputer "mo:base/ExperimentalInternetComputer";
import Time "mo:base/Time";
import Error "mo:base/Error";
import Option "mo:base/Option";
import Nat "mo:base/Nat";
import Text "mo:base/Text";
import Array "mo:base/Array";
import Buffer "mo:base/Buffer";
import List "mo:base/List";
import Deque "mo:base/Deque";
import Result "mo:base/Result";
import Principal "mo:base/Principal";
import Debug "mo:base/Debug";
import Types "./Types";
import ICType "./IC";
import PoW "./PoW";
import Logs "./Logs";
import Metrics "./Metrics";
import Wasm "canister:wasm-utils";

shared (creator) actor class Self(opt_params : ?Types.InitParams) = this {
    let IC : ICType.Self = actor "aaaaa-aa";
    let params = Option.get(opt_params, Types.defaultParams);
    var pool = Types.CanisterPool(params.max_num_canisters, params.canister_time_to_live, params.max_family_tree_size);
    let nonceCache = PoW.NonceCache(params.nonce_time_to_live);
    var statsByOrigin = Logs.StatsByOrigin();

    stable let controller = creator.caller;
    stable var stats = Logs.defaultStats;
    stable var stablePool : [Types.CanisterInfo] = [];
    stable var stableMetadata : [(Principal, (Int, Bool))] = [];
    stable var stableChildren : [(Principal, [Principal])] = [];
    stable var stableTimers : [Types.CanisterInfo] = [];
    stable var previousParam : ?Types.InitParams = null;
    stable var stableStatsByOrigin : Logs.SharedStatsByOrigin = (#leaf, #leaf);

    system func preupgrade() {
        let (tree, metadata, children, timers) = pool.share();
        stablePool := tree;
        stableMetadata := metadata;
        stableChildren := children;
        stableTimers := timers;
        previousParam := ?params;
        stableStatsByOrigin := statsByOrigin.share();
    };

    system func postupgrade() {
        ignore do ? {
            if (previousParam!.max_num_canisters > params.max_num_canisters) {
                Debug.trap("Cannot reduce canisterPool for upgrade");
            };
        };
        pool.unshare(stablePool, stableMetadata, stableChildren);
        for (info in stableTimers.vals()) {
            updateTimer(info);
        };
        statsByOrigin.unshare(stableStatsByOrigin);
    };

    public query func getInitParams() : async Types.InitParams {
        params;
    };

    public query func getStats() : async (Logs.Stats, [(Text, Nat)], [(Text, Nat)]) {
        let (canister, install) = statsByOrigin.dump();
        (stats, canister, install);
    };

    public query func balance() : async Nat {
        Cycles.balance();
    };

    public func wallet_receive() : async () {
        let amount = Cycles.available();
        ignore Cycles.accept amount;
    };

    private func getExpiredCanisterInfo(origin : Logs.Origin) : async Types.CanisterInfo {
        switch (pool.getExpiredCanisterId()) {
            case (#newId) {
                Cycles.add(params.cycles_per_canister);
                let cid = await IC.create_canister { settings = null };
                let now = Time.now();
                let info = { id = cid.canister_id; timestamp = now };
                pool.add info;
                stats := Logs.updateStats(stats, #getId(params.cycles_per_canister));
                statsByOrigin.addCanister(origin);
                info;
            };
            case (#reuse info) {
                let cid = { canister_id = info.id };
                let status = await IC.canister_status cid;
                let topUpCycles : Nat = if (status.cycles < params.cycles_per_canister) {
                    params.cycles_per_canister - status.cycles;
                } else { 0 };
                if (topUpCycles > 0) {
                    Cycles.add topUpCycles;
                    await IC.deposit_cycles cid;
                };
                if (Option.isSome(status.module_hash)) {
                    await IC.uninstall_code cid;
                };
                switch (status.status) {
                    case (#stopped or #stopping) {
                        await IC.start_canister cid;
                    };
                    case _ {};
                };
                stats := Logs.updateStats(stats, #getId topUpCycles);
                statsByOrigin.addCanister(origin);
                info;
            };
            case (#outOfCapacity time) {
                let second = time / 1_000_000_000;
                stats := Logs.updateStats(stats, #outOfCapacity second);
                throw Error.reject("No available canister id, wait for " # debug_show (second) # " seconds.");
            };
        };
    };
    func validateOrigin(origin: Logs.Origin) : Bool {
        if (origin.origin == "") {
            return false;
        };
        for (tag in origin.tags.vals()) {
            // reject server side tags
            if (tag == "mode:install" or tag == "mode:reinstall" or tag == "mode:upgrade" or tag == "wasm:profiling" or tag == "wasm:asset") {
                return false;
            }
        };
        return true;
    };

    public shared ({ caller }) func getCanisterId(nonce : PoW.Nonce, origin : Logs.Origin) : async Types.CanisterInfo {
        if (not validateOrigin(origin)) {
            throw Error.reject "Please specify a valid origin";
        };
        if (caller != controller and not nonceCache.checkProofOfWork(nonce)) {
            stats := Logs.updateStats(stats, #mismatch);
            throw Error.reject "Proof of work check failed";
        };
        nonceCache.pruneExpired();
        if (nonceCache.contains nonce) {
            stats := Logs.updateStats(stats, #mismatch);
            throw Error.reject "Nonce already used";
        };
        nonceCache.add nonce;
        await getExpiredCanisterInfo(origin);
    };

    type InstallConfig = { profiling: Bool; is_whitelisted: Bool; origin: Logs.Origin };
    public shared ({ caller }) func installCode(info : Types.CanisterInfo, args : Types.InstallArgs, install_config : InstallConfig) : async Types.CanisterInfo {
        if (not validateOrigin(install_config.origin)) {
            throw Error.reject "Please specify a valid origin";
        };
        if (info.timestamp == 0) {
            stats := Logs.updateStats(stats, #mismatch);
            throw Error.reject "Cannot install removed canister";
        };
        if (not pool.find info) {
            stats := Logs.updateStats(stats, #mismatch);
            throw Error.reject "Cannot find canister";
        } else {
            let config = {
                profiling = install_config.profiling;
                remove_cycles_add = true;
                limit_stable_memory_page = ?(16384 : Nat32); // Limit to 1G of stable memory
                backend_canister_id = ?Principal.fromActor(this);
            };
            let wasm = if (caller == controller and install_config.is_whitelisted) {
                args.wasm_module;
            } else if (install_config.is_whitelisted) {
                await Wasm.is_whitelisted(args.wasm_module);
            } else {
                await Wasm.transform(args.wasm_module, config);
            };
            let newArgs = {
                arg = args.arg;
                wasm_module = wasm;
                mode = args.mode;
                canister_id = args.canister_id;
            };
            await IC.install_code newArgs;
            stats := Logs.updateStats(stats, #install);

            // Build tags from install arguments
            let tags = Buffer.fromArray<Text>(install_config.origin.tags);
            if (install_config.profiling) {
                tags.add("wasm:profiling");
            };
            if (install_config.is_whitelisted) {
                tags.add("wasm:asset");
            };
            switch (args.mode) {
            case (#install) { tags.add("mode:install") };
            case (#upgrade) { tags.add("mode:upgrade") };
            case (#reinstall) { tags.add("mode:reinstall") };
            };
            let origin = { origin = install_config.origin.origin; tags = Buffer.toArray(tags) };
            statsByOrigin.addInstall(origin);
            switch (pool.refresh(info, install_config.profiling)) {
                case (?newInfo) {
                     updateTimer(newInfo);
                     newInfo;
                 };
                case null { throw Error.reject "Cannot find canister" };
            };
        };
    };

    func updateTimer(info: Types.CanisterInfo) {
        func job() : async () {
            pool.removeTimer(info.id);
            // It is important that the timer job checks for the timestamp first.
            // This prevents late-runner jobs from deleting newly installed code.
            await removeCode(info);
        };
        pool.updateTimer(info, job);
    };

    public func callForward(info : Types.CanisterInfo, function : Text, args : Blob) : async Blob {
        if (pool.find info) {
            await InternetComputer.call(info.id, function, args);
        } else {
            stats := Logs.updateStats(stats, #mismatch);
            throw Error.reject "Cannot find canister";
        };
    };

    public func removeCode(info : Types.CanisterInfo) : async () {
        if (pool.find info) {
            await IC.uninstall_code { canister_id = info.id };
            ignore pool.retire info;
        } else {
            stats := Logs.updateStats(stats, #mismatch);
        };
    };

    public func GCCanisters() {
        for (id in pool.gcList().vals()) {
            await IC.uninstall_code { canister_id = id };
        };
    };

    public query func getSubtree(parent : Types.CanisterInfo) : async [(Principal, [Types.CanisterInfo])] {
        if (not pool.find(parent)) {
            throw Error.reject "Canister not found";
        };
        // Do not return subtree for non-root parent to save cost
        if (not pool.isRoot(parent.id)) {
            return [];
        };
        var result = List.nil<(Principal, [Types.CanisterInfo])>();
        var queue = Deque.empty<Principal>();
        queue := Deque.pushBack(queue, parent.id);
        label l loop {
            switch (Deque.popFront(queue)) {
                case null break l;
                case (?(id, tail)) {
                    queue := tail;
                    let children = List.map(
                        pool.getChildren(id),
                        func(child : Principal) : Types.CanisterInfo {
                            queue := Deque.pushBack(queue, child);
                            Option.unwrap(pool.info(child));
                        },
                    );
                    result := List.push((id, List.toArray children), result);
                };
            };
        };
        List.toArray(result);
    };

    public query ({ caller }) func dump() : async [Types.CanisterInfo] {
        if (caller != controller) {
            throw Error.reject "Only called by controller";
        };
        pool.share().0;
    };

    public shared ({ caller }) func resetStats() : async () {
        if (caller != controller) {
            throw Error.reject "Only called by controller";
        };
        stats := Logs.defaultStats;
        statsByOrigin := Logs.StatsByOrigin();
    };
    public shared ({ caller }) func mergeTags(from: Text, to: ?Text) : async () {
        if (caller != controller) {
            throw Error.reject "Only called by controller";
        };
        statsByOrigin.merge_tag(from, to);
    };

    // Metrics
    public query func http_request(req : Metrics.HttpRequest) : async Metrics.HttpResponse {
        if (req.url == "/metrics") {
            let body = Metrics.metrics(stats);
            {
                status_code = 200;
                headers = [("Content-Type", "text/plain; version=0.0.4"), ("Content-Length", Nat.toText(body.size()))];
                body = body;
            };
        } else {
            {
                status_code = 404;
                headers = [];
                body = Text.encodeUtf8 "Not supported";
            };
        };
    };

    /*
    * The following methods are wrappers/immitations of the management canister's methods that require controller permissions.
    * In general, the backend is the sole controller of all playground pool canisters. Any canister that attempts to call the
    * management canister will be redirected here instead by the wasm transformation above.
    */
    private func sanitizeInputs(caller : Principal, callee : Principal) : Result.Result<Types.CanisterInfo, Text -> Text> {
        if (not pool.findId caller) {
            return #err(func methodName = "Only a canister managed by the Motoko Playground can call " # methodName);
        };
        switch (pool.info callee) {
            case null {
                #err(func methodName = "Can only call " # methodName # " on canisters in the Motoko Playground");
            };
            case (?info) {
                // Also allow the canister to manage itself, as we don't allow canisters to change settings.
                if (not (caller == callee) and not pool.isParentOf(caller, callee)) {
                    #err(func methodName = "Can only call " # methodName # " on canisters spawned by your own code");
                } else {
                    #ok info;
                };
            };
        };
    };

    public shared ({ caller }) func create_canister({
        settings : ?ICType.canister_settings;
    }) : async { canister_id : ICType.canister_id } {
        if (Option.isSome(settings)) {
            throw Error.reject "Can only call create_canister with null settings";
        };
        if (not pool.findId caller) {
            throw Error.reject "Only a canister managed by the Motoko Playground can call create_canister";
        };
        let info = await getExpiredCanisterInfo({origin="spawned"; tags=[]});
        let result = pool.setChild(caller, info.id);
        if (not result) {
            throw Error.reject("In the Motoko Playground, each top level canister can only spawn " # Nat.toText(params.max_family_tree_size) # " descendants including itself");
        };
        { canister_id = info.id };
    };

    // Disabled to prevent the user from updating the controller list (amongst other settings)
    public shared ({ caller }) func update_settings({
        canister_id : ICType.canister_id;
        settings : ICType.canister_settings;
    }) : async () {
        throw Error.reject "Cannot call update_settings from within Motoko Playground";
    };

    public shared ({ caller }) func install_code({
        arg : Blob;
        wasm_module : ICType.wasm_module;
        mode : { #reinstall; #upgrade; #install };
        canister_id : ICType.canister_id;
    }) : async () {
        switch (sanitizeInputs(caller, canister_id)) {
            case (#ok info) {
                let args = { arg; wasm_module; mode; canister_id };
                let config = { profiling = pool.profiling caller; is_whitelisted = false; origin = {origin = "spawned"; tags = [] } };
                ignore await installCode(info, args, config); // inherit the profiling of the parent
            };
            case (#err makeMsg) throw Error.reject(makeMsg "install_code");
        };
    };

    public shared ({ caller }) func uninstall_code({
        canister_id : ICType.canister_id;
    }) : async () {
        switch (sanitizeInputs(caller, canister_id)) {
            case (#ok _) await IC.uninstall_code { canister_id };
            case (#err makeMsg) throw Error.reject(makeMsg "uninstall_code");
        };
    };

    public shared ({ caller }) func canister_status({
        canister_id : ICType.canister_id;
    }) : async {
        status : { #stopped; #stopping; #running };
        memory_size : Nat;
        cycles : Nat;
        settings : ICType.definite_canister_settings;
        module_hash : ?Blob;
    } {
        switch (sanitizeInputs(caller, canister_id)) {
            case (#ok _) await IC.canister_status { canister_id };
            case (#err makeMsg) {
                if (caller == canister_id) {
                    await IC.canister_status { canister_id };
                } else { throw Error.reject(makeMsg "canister_status") };
            };
        };
    };

    public shared ({ caller }) func stop_canister({
        canister_id : ICType.canister_id;
    }) : async () {
        switch (sanitizeInputs(caller, canister_id)) {
            case (#ok _) await IC.stop_canister { canister_id };
            case (#err makeMsg) throw Error.reject(makeMsg "stop_canister");
        };
    };

    public shared ({ caller }) func start_canister({
        canister_id : ICType.canister_id;
    }) : async () {
        switch (sanitizeInputs(caller, canister_id)) {
            case (#ok _) await IC.start_canister { canister_id };
            case (#err makeMsg) throw Error.reject(makeMsg "start_canister");
        };
    };

    public shared ({ caller }) func delete_canister({
        canister_id : ICType.canister_id;
    }) : async () {
        switch (sanitizeInputs(caller, canister_id)) {
            case (#ok info) await removeCode(info); // retire the canister back into pool instead of deleting
            case (#err makeMsg) throw Error.reject(makeMsg "delete_canister");
        };
    };

    system func inspect({
        msg : {
            #GCCanisters : Any;
            #balance : Any;
            #callForward : Any;
            #dump : Any;
            #getCanisterId : Any;
            #getSubtree : Any;
            #getInitParams : Any;
            #getStats : Any;
            #http_request : Any;
            #installCode : Any;
            #removeCode : Any;
            #resetStats : Any;
            #mergeTags : Any;
            #wallet_receive : Any;

            #create_canister : Any;
            #update_settings : Any;
            #install_code : Any;
            #uninstall_code : Any;
            #canister_status : Any;
            #start_canister : Any;
            #stop_canister : Any;
            #delete_canister : Any;
        };
    }) : Bool {
        switch msg {
            case (#create_canister _) false;
            case (#update_settings _) false;
            case (#install_code _) false;
            case (#uninstall_code _) false;
            case (#canister_status _) false;
            case (#start_canister _) false;
            case (#stop_canister _) false;
            case (#delete_canister _) false;
            case _ true;
        };
    };
};
