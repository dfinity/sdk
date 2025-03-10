import Cycles "mo:base/ExperimentalCycles";
import InternetComputer "mo:base/ExperimentalInternetComputer";
import Time "mo:base/Time";
import Error "mo:base/Error";
import Option "mo:base/Option";
import Nat "mo:base/Nat";
import Text "mo:base/Text";
import Buffer "mo:base/Buffer";
import List "mo:base/List";
import Deque "mo:base/Deque";
import Result "mo:base/Result";
import Principal "mo:base/Principal";
import Debug "mo:base/Debug";
import Types "./Types";
import ICType "./IC";
import EVM "./EVM";
import PoW "./PoW";
import Logs "./Logs";
import Metrics "./Metrics";
import WasmUtilsType "./Wasm-utils";

shared (creator) actor class Self(opt_params : ?Types.InitParams) = this {
    let IC : ICType.Self = actor "aaaaa-aa";
    let evm : EVM.Self = actor "7hfb6-caaaa-aaaar-qadga-cai";
    let params = Option.get(opt_params, Types.defaultParams);
    let Wasm : WasmUtilsType.Self = actor(Option.get(params.wasm_utils_principal, "ozk6r-tyaaa-aaaab-qab4a-cai"));
    var pool = Types.CanisterPool(params);
    let nonceCache = PoW.NonceCache(params.nonce_time_to_live);
    var statsByOrigin = Logs.StatsByOrigin();

    stable let controller = creator.caller;
    stable var stats = Logs.defaultStats;
    stable var stablePool : [Types.CanisterInfo] = [];
    stable var stableMetadata : [(Principal, (Int, Bool))] = [];
    stable var stableChildren : [(Principal, [Principal])] = [];
    stable var stableTimers : [Types.CanisterInfo] = [];
    stable var stableSnapshots : [(Principal, Blob)] = [];
    stable var previousParam : ?Types.InitParams = null;
    stable var stableStatsByOrigin : Logs.SharedStatsByOrigin = (#leaf, #leaf);

    system func preupgrade() {
        let (tree, metadata, children, timers, snapshots) = pool.share();
        stablePool := tree;
        stableMetadata := metadata;
        stableChildren := children;
        stableTimers := timers;
        stableSnapshots := snapshots;
        previousParam := ?params;
        stableStatsByOrigin := statsByOrigin.share();
    };

    system func postupgrade() {
        ignore do ? {
            if (previousParam!.max_num_canisters > params.max_num_canisters) {
                Debug.trap("Cannot reduce canisterPool for upgrade");
            };
        };
        pool.unshare(stablePool, stableMetadata, stableChildren, stableSnapshots);
        for (info in stableTimers.vals()) {
            updateTimer<system>(info);
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
        ignore Cycles.accept<system> amount;
    };
    private func pool_uninstall_code(cid : Principal) : async* () {
        let f1 = IC.uninstall_code { canister_id = cid };
        let f2 = removeSnapshot cid;
        await f1;
        await* f2;
    };
    private func getExpiredCanisterInfo(origin : Logs.Origin) : async* (Types.CanisterInfo, ICType.canister_install_mode) {
        let res = switch (pool.getExpiredCanisterId()) {
            case (#newId) {
              try {
                Cycles.add<system>(params.cycles_per_canister);
                let settings = ?{
                    controllers = null;
                    log_visibility = ?#public_;
                    freezing_threshold = null;
                    memory_allocation = null;
                    compute_allocation = null;
                    wasm_memory_limit = null;
                };
                let cid = await IC.create_canister { settings };
                let now = Time.now();
                let info = { id = cid.canister_id; timestamp = now };
                pool.add info;
                stats := Logs.updateStats(stats, #getId(params.cycles_per_canister));
                statsByOrigin.addCanister(origin);
                (info, #install);
              } catch(e) {
                  pool.rollbackLen();
                  throw e;
              };
            };
            case (#reuse info) {
                let cid = { canister_id = info.id };
                let status = await IC.canister_status cid;
                let topUpCycles : Nat = if (status.cycles < params.cycles_per_canister) {
                    params.cycles_per_canister - status.cycles;
                } else { 0 };
                if (topUpCycles > 0) {
                    Cycles.add<system> topUpCycles;
                    await IC.deposit_cycles cid;
                };
                let need_uninstall = switch ((params.stored_module, status.module_hash)) {
                case ((null, ?_)) { true };
                case ((_, null)) { false };
                case (?stored, ?current) { stored.hash != current };
                };
                if (need_uninstall) {
                    await* pool_uninstall_code(cid.canister_id);
                };
                switch (status.status) {
                    case (#stopped or #stopping) {
                        await IC.start_canister cid;
                    };
                    case _ {};
                };
                stats := Logs.updateStats(stats, #getId topUpCycles);
                statsByOrigin.addCanister(origin);
                let mode = if (need_uninstall and Option.isNull(params.stored_module)) { #install } else { #reinstall };
                (info, mode);
            };
            case (#outOfCapacity time) {
                let second = time / 1_000_000_000;
                stats := Logs.updateStats(stats, #outOfCapacity second);
                throw Error.reject("No available canister id, wait for " # debug_show (second) # " seconds.");
            };
        };
        switch (params.stored_module) {
        case null {};
        case (?stored) {
                 await IC.install_chunked_code {
                     arg = stored.arg;
                     target_canister = res.0.id;
                     store_canister = ?(Principal.fromActor this);
                     chunk_hashes_list = [{ hash = stored.hash }];
                     wasm_module_hash = stored.hash;
                     mode = res.1;
                 }
             };
        };
        res;
    };
    func validateOrigin(origin: Logs.Origin) : Bool {
        if (origin.origin == "") {
            return false;
        };
        for (tag in origin.tags.vals()) {
            // reject server side tags
            if (tag == "mode:install" or tag == "mode:reinstall" or tag == "mode:upgrade" or tag == "wasm:profiling" or tag == "wasm:asset" or tag == "wasm:profiling:stable") {
                return false;
            }
        };
        return true;
    };

    // Before this call, make sure the installed wasm is not instrumented
    public shared ({ caller }) func transferOwnership(info: Types.CanisterInfo, controllers: [Principal]) : async () {
        if (not Principal.isController(caller)) {
            throw Error.reject "Only called by controller";
        };
        if (pool.find info) {
            pool.removeCanister(info);
            let settings = {
                controllers = ?controllers;
                log_visibility = ?#controllers;
                freezing_threshold = null;
                memory_allocation = null;
                compute_allocation = null;
                wasm_memory_limit = null;
            };
            await IC.update_settings { canister_id = info.id; settings };
            statsByOrigin.addCanister({ origin = "external"; tags = [] });
        } else {
            stats := Logs.updateStats(stats, #mismatch);
            throw Error.reject "transferOwnership: Cannot find canister";
        };
    };
    // Install code after transferOwnership. This call can fail if the user removes the playground from its controllers.
    public shared ({ caller }) func installExternalCanister(args : Types.InstallArgs) : async () {
        if (not Principal.isController(caller)) {
            throw Error.reject "Only called by controller";
        };
        if (pool.findId(args.canister_id)) {
            stats := Logs.updateStats(stats, #mismatch);
            throw Error.reject "Canister is still solely controlled by the playground";
        };
        await IC.install_code args;
        statsByOrigin.addInstall({ origin = "external"; tags = [] });
    };
    // Combine create_canister and install_code into a single update call. Returns the current available canister id.
    public shared ({ caller }) func deployCanister(opt_info: ?Types.CanisterInfo, args: ?Types.DeployArgs) : async (Types.CanisterInfo, ICType.canister_install_mode) {
        if (not Principal.isController(caller)) {
            throw Error.reject "Only called by controller";
        };
        if (Option.isSome(params.stored_module) and Option.isSome(args)) {
            throw Error.reject "args should be null when stored_module is set";
        };
        let origin = { origin = "admin"; tags = [] };
        let (info, default_mode) = switch (opt_info) {
        case null { await* getExpiredCanisterInfo(origin) };
        case (?info) {
                 if (pool.find info) {
                     (info, #upgrade(null))
                 } else {
                     if (pool.findId(info.id)) {
                         await* getExpiredCanisterInfo(origin)
                     } else {
                         stats := Logs.updateStats(stats, #mismatch);
                         throw Error.reject "deployCanister: Cannot find canister";
                     };
                 };
             };
        };
        let mode = switch (args) {
            case (?args) {
                Option.get(args.mode, default_mode);
            };
            case (null) {
                default_mode;
            };
        };
        switch (args) {
        case (?args) {
                 let wasm = if (Option.get(args.bypass_wasm_transform, false)) {
                     args.wasm_module
                 } else {
                     let config = {
                         profiling = null;
                         remove_cycles_add = true;
                         limit_stable_memory_page = ?(16384 : Nat32); // Limit to 1G of stable memory
                         limit_heap_memory_page = ?(16384 : Nat32); // Limit to 1G of heap memory
                         backend_canister_id = ?Principal.fromActor(this);
                     };
                     await Wasm.transform(args.wasm_module, config);
                 };
                 await IC.install_code {
                     arg = args.arg;
                     wasm_module = wasm;
                     mode = mode;
                     canister_id = info.id;
                 };
                 stats := Logs.updateStats(stats, #install);
             };
        case null {};
        };
        switch (pool.refresh(info, false)) {
        case (?newInfo) {
                 updateTimer<system>(newInfo);
                 (newInfo, mode);
             };
        case null { throw Error.reject "pool.refresh: Cannot find canister" };
        };
    };

    public shared ({ caller }) func getCanisterId(nonce : PoW.Nonce, origin : Logs.Origin) : async Types.CanisterInfo {
        if (Option.get(params.admin_only, false)) {
            throw Error.reject "Cannot call this endpoint when admin_only is true";
        };
        if (not validateOrigin(origin)) {
            throw Error.reject "Please specify a valid origin";
        };
        if (not Principal.isController(caller) and not nonceCache.checkProofOfWork(nonce)) {
            stats := Logs.updateStats(stats, #mismatch);
            throw Error.reject "Proof of work check failed";
        };
        nonceCache.pruneExpired();
        if (nonceCache.contains nonce) {
            stats := Logs.updateStats(stats, #mismatch);
            throw Error.reject "Nonce already used";
        };
        nonceCache.add nonce;
        (await* getExpiredCanisterInfo(origin)).0;
    };
    // Only used by webcontainer frontend. Once we have a dedicated frontend pool, this endpoint should be deprecated.
    public func installStoredWasm(info : Types.CanisterInfo, args: Types.InstallArgs, origin: Logs.Origin) : async Types.CanisterInfo {
        if (not validateOrigin(origin)) {
            throw Error.reject "Please specify a valid origin";
        };
        if (not pool.find info) {
            stats := Logs.updateStats(stats, #mismatch);
            throw Error.reject "Cannot find canister";
        };
        assert(info.id == args.canister_id);
        if (info.timestamp == 0) {
            stats := Logs.updateStats(stats, #mismatch);
            throw Error.reject "Cannot install removed canister";
        };
        let module_hash = args.wasm_module;
        await IC.install_chunked_code {
            arg = args.arg;
            target_canister = args.canister_id;
            store_canister = ?(Principal.fromActor this);
            chunk_hashes_list = [{ hash = module_hash }];
            wasm_module_hash = module_hash;
            mode = args.mode;
        };
        statsByOrigin.addInstall({ origin = origin.origin; tags = ["wasm:asset"] });
        switch (pool.refresh(info, false)) {
        case (?newInfo) {
                 updateTimer<system>(newInfo);
                 newInfo;
             };
        case null { throw Error.reject "Cannot find canister" };
        };
    };
    public shared ({ caller }) func installCode(info : Types.CanisterInfo, args : Types.InstallArgs, install_config : Types.InstallConfig) : async Types.CanisterInfo {
        if (not validateOrigin(install_config.origin)) {
            throw Error.reject "Please specify a valid origin";
        };
        assert(info.id == args.canister_id);
        if (info.timestamp == 0) {
            stats := Logs.updateStats(stats, #mismatch);
            throw Error.reject "Cannot install removed canister";
        };
        if (not pool.find info) {
            stats := Logs.updateStats(stats, #mismatch);
            throw Error.reject "Cannot find canister";
        } else {
            let profiling_config: ?Types.ProfilingConfig = if (install_config.profiling) {
                ?{ start_page = install_config.start_page; page_limit = install_config.page_limit }
            } else {
                null
            };
            let config = {
                profiling = profiling_config;
                remove_cycles_add = true;
                limit_stable_memory_page = ?(16384 : Nat32); // Limit to 1G of stable memory
                limit_heap_memory_page = ?(16384 : Nat32); // Limit to 1G of heap memory
                backend_canister_id = ?Principal.fromActor(this);
            };
            let wasm = if (Principal.isController(caller) and install_config.is_whitelisted) {
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
                switch (install_config.start_page) {
                case null { tags.add("wasm:profiling") };
                case _ { tags.add("wasm:profiling:stable") };
                };
            };
            if (install_config.is_whitelisted) {
                tags.add("wasm:asset");
            };
            switch (args.mode) {
            case (#install) { tags.add("mode:install") };
            case (#upgrade(_)) { tags.add("mode:upgrade") };
            case (#reinstall) { tags.add("mode:reinstall") };
            };
            let origin = { origin = install_config.origin.origin; tags = Buffer.toArray(tags) };
            statsByOrigin.addInstall(origin);
            switch (pool.refresh(info, install_config.profiling)) {
                case (?newInfo) {
                     updateTimer<system>(newInfo);
                     newInfo;
                 };
                case null { throw Error.reject "Cannot find canister" };
            };
        };
    };

    func updateTimer<system>(info: Types.CanisterInfo) {
        if (Option.isSome(params.stored_module)) {
            return;
        };
        func job() : async () {
            pool.removeTimer(info.id);
            // It is important that the timer job checks for the timestamp first.
            // This prevents late-runner jobs from deleting newly installed code.
            await removeCode(info);
        };
        pool.updateTimer<system>(info, job);
    };

    public func callForward(info : Types.CanisterInfo, function : Text, args : Blob) : async Blob {
        if (pool.find info or not pool.findId(info.id)) {
            await InternetComputer.call(info.id, function, args);
        } else {
            stats := Logs.updateStats(stats, #mismatch);
            throw Error.reject "Cannot find canister";
        };
    };
    public func takeSnapshot(info : Types.CanisterInfo) : async ?Blob {
        if (pool.find info or not pool.findId(info.id)) {
            let snapshot = await IC.take_canister_snapshot({ canister_id = info.id; replace_snapshot = pool.getSnapshot(info.id) });
            pool.setSnapshot(info.id, snapshot.id);
            ?snapshot.id;
        } else {
            stats := Logs.updateStats(stats, #mismatch);
            null;
        }
    };
    public func loadSnapshot(info : Types.CanisterInfo) : async () {
        if (pool.find info or not pool.findId(info.id)) {
            switch (pool.getSnapshot(info.id)) {
              case (?snapshot) await IC.load_canister_snapshot({ canister_id = info.id; snapshot_id = snapshot });
              case null throw Error.reject "Cannot find snapshot";
            };
        } else {
            stats := Logs.updateStats(stats, #mismatch);
        }
    };
    private func removeSnapshot(id : Principal) : async* () {
        switch (pool.getSnapshot(id)) {
          case (?snapshot) {
                 await IC.delete_canister_snapshot({ canister_id = id; snapshot_id = snapshot });
                 pool.removeSnapshot(id);
             };
          case null {};
        };
    };
    public func deleteSnapshot(info : Types.CanisterInfo) : async () {
        if (pool.find info or not pool.findId(info.id)) {
            await* removeSnapshot(info.id);
        } else {
            stats := Logs.updateStats(stats, #mismatch);
        }
    };
    public func listSnapshots(info : Types.CanisterInfo) : async [ICType.snapshot] {
        if (pool.find info or not pool.findId(info.id)) {
            await IC.list_canister_snapshots({ canister_id = info.id });
        } else {
            stats := Logs.updateStats(stats, #mismatch);
            []
        }
    };

    public func removeCode(info : Types.CanisterInfo) : async () {
        if (pool.find info) {
            await* pool_uninstall_code(info.id);
            ignore pool.retire info;
        } else {
            stats := Logs.updateStats(stats, #mismatch);
        };
    };
    public shared({caller}) func releaseAllCanisters() : async () {
        if (not Principal.isController(caller)) {
            throw Error.reject "only called by controllers";
        };
        for (info in pool.getAllCanisters()) {
            if (Option.isNull(params.stored_module)) {
                await* pool_uninstall_code(info.id);
            };
            ignore pool.retire info;
        };
    };

    public func GCCanisters() {
        for (id in pool.gcList().vals()) {
            await* pool_uninstall_code(id);
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
                            let ?info = pool.info(child) else { Debug.trap "unwrap pool.info" };
                            info;
                        },
                    );
                    result := List.push((id, List.toArray children), result);
                };
            };
        };
        List.toArray(result);
    };

    public query ({ caller }) func dump() : async [Types.CanisterInfo] {
        if (not Principal.isController(caller)) {
            throw Error.reject "Only called by controller";
        };
        pool.share().0;
    };

    public shared ({ caller }) func resetStats() : async () {
        if (not Principal.isController(caller)) {
            throw Error.reject "Only called by controller";
        };
        stats := Logs.defaultStats;
        statsByOrigin := Logs.StatsByOrigin();
    };
    public shared ({ caller }) func mergeTags(from: Text, to: ?Text) : async () {
        if (not Principal.isController(caller)) {
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
        let info = (await* getExpiredCanisterInfo({origin="spawned"; tags=[]})).0;
        let result = pool.setChild(caller, info.id);
        if (not result) {
            throw Error.reject("In the Motoko Playground, each top level canister can only spawn " # Nat.toText(params.max_family_tree_size) # " descendants including itself");
        };
        { canister_id = info.id };
    };

    public shared ({ caller }) func update_settings({canister_id:Principal; settings: ICType.canister_settings}) : async () {
        if (Principal.isController(caller)) {
            assert(pool.findId canister_id);
            assert(settings.controllers == null);
            await IC.update_settings { canister_id; settings };
        } else {
            // Disabled to prevent the user from updating the controller list (amongst other settings)
            throw Error.reject "Cannot call update_settings from within Motoko Playground";
        }
    };

    public shared ({ caller }) func install_code({
        arg : Blob;
        wasm_module : ICType.wasm_module;
        mode : ICType.canister_install_mode;
        canister_id : ICType.canister_id;
    }) : async () {
        switch (sanitizeInputs(caller, canister_id)) {
            case (#ok info) {
                let args = { arg; wasm_module; mode; canister_id };
                // TODO: propagate start_page and page_limit
                let config = { profiling = pool.profiling caller; is_whitelisted = false; origin = {origin = "spawned"; tags = [] }; start_page = null; page_limit = null };
                ignore await installCode(info, args, config); // inherit the profiling of the parent
            };
            case (#err makeMsg) throw Error.reject(makeMsg "install_code");
        };
    };

    public shared ({ caller }) func uninstall_code({
        canister_id : ICType.canister_id;
    }) : async () {
        switch (sanitizeInputs(caller, canister_id)) {
            case (#ok _) await* pool_uninstall_code(canister_id);
            case (#err makeMsg) throw Error.reject(makeMsg "uninstall_code");
        };
    };

    public shared ({ caller }) func canister_status({
        canister_id : ICType.canister_id;
    }) : async ICType.canister_status_result {
        if (Principal.isController(caller)) {
            assert(pool.findId canister_id);
            return await IC.canister_status { canister_id };
        };
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
    public shared ({ caller }) func list_canister_snapshots({ canister_id : Principal }) : async [ICType.snapshot] {
        switch (sanitizeInputs(caller, canister_id)) {
            case (#ok info) await IC.list_canister_snapshots({ canister_id });
            case (#err makeMsg) throw Error.reject(makeMsg "list_canister_snapshots");
        };
    };
    public shared ({ caller }) func take_canister_snapshot({ canister_id : Principal; replace_snapshot : ?Blob }) : async ICType.snapshot {
        switch (sanitizeInputs(caller, canister_id)) {
            case (#ok info) {
                     let snapshot = await IC.take_canister_snapshot({ canister_id; replace_snapshot });
                     pool.setSnapshot(canister_id, snapshot.id);
                     snapshot;
             };
            case (#err makeMsg) throw Error.reject(makeMsg "take_canister_snapshots");
        };
    };
    public shared ({ caller }) func delete_canister_snapshot({ canister_id : Principal; snapshot_id : Blob }) : async () {
        switch (sanitizeInputs(caller, canister_id)) {
            case (#ok info) {
                     await IC.delete_canister_snapshot({ canister_id; snapshot_id });
                     pool.removeSnapshot(canister_id);
             };
            case (#err makeMsg) throw Error.reject(makeMsg "delete_canister_snapshots");
        };
    };
    public shared func load_canister_snapshot({}) : async () {
        throw Error.reject("Cannot call load_canister_snapshot from canister itself");
    };
    public shared ({ caller }) func sign_with_ecdsa(arg: ICType.sign_with_ecdsa_args) : async ICType.sign_with_ecdsa_result {
        await* pool.addCycles(caller, #method "sign_with_ecdsa");
        let res = await IC.sign_with_ecdsa(arg);
        await* pool.addCycles(caller, #refund);
        res
    };
    public shared ({ caller }) func sign_with_schnorr(arg: ICType.sign_with_schnorr_args) : async ICType.sign_with_schnorr_result {
        await* pool.addCycles(caller, #method "sign_with_schnorr");
        let res = await IC.sign_with_schnorr(arg);
        await* pool.addCycles(caller, #refund);
        res
    };
    public shared ({ caller }) func _ttp_request(request : ICType.http_request_args) : async ICType.http_request_result {
        await* pool.addCycles(caller, #method "http_request");
        let new_request = switch (request.transform) {
        case null {
                 { request with transform = null };
             };
        case (?transform) {
                 let payload = { caller; transform };
                 let fake_actor: actor { __transform: ICType.transform_function } = actor(Principal.toText(Principal.fromActor this));
                 let new_transform = ?{ function = fake_actor.__transform; context = to_candid(payload) };
                 { request with transform = new_transform };
             };
        };
        let res = await IC.http_request(new_request);
        await* pool.addCycles(caller, #refund);
        res;
    };
    public shared composite query({ caller }) func __transform({context: Blob; response: ICType.http_request_result}) : async ICType.http_request_result {
        // TODO Remove anonymous identity once https://github.com/dfinity/ic/pull/1337 is released
        if (caller != Principal.fromText("aaaaa-aa") and caller != Principal.fromText("2vxsx-fae")) {
            throw Error.reject "Only the management canister can call __transform";
        };
        let ?raw : ?{ caller: Principal; transform: {context: Blob; function: ICType.transform_function} } = from_candid context else {
            throw Error.reject "__transform: Invalid context";
        };
        if (not pool.findId(raw.caller)) {
            throw Error.reject "__transform: Only a canister managed by the Motoko Playground can call __transform";
        };
        await raw.transform.function({ context = raw.transform.context; response });
    };
    // Endpoints for EVM RPC canister
    public shared ({ caller }) func eth_call(service: EVM.RpcServices, config: ?EVM.RpcConfig, arg: EVM.CallArgs) : async EVM.MultiCallResult {
        await* pool.addCycles(caller, #method "eth_call");
        let res = await evm.eth_call(service, config, arg);
        await* pool.addCycles(caller, #refund);
        res
    };
    public shared ({ caller }) func eth_feeHistory(service: EVM.RpcServices, config: ?EVM.RpcConfig, arg: EVM.FeeHistoryArgs) : async EVM.MultiFeeHistoryResult {
        await* pool.addCycles(caller, #method "eth_feeHistory");
        let res = await evm.eth_feeHistory(service, config, arg);
        await* pool.addCycles(caller, #refund);
        res
    };
    public shared ({ caller }) func eth_getBlockByNumber(service: EVM.RpcServices, config: ?EVM.RpcConfig, arg: EVM.BlockTag) : async EVM.MultiGetBlockByNumberResult {
        await* pool.addCycles(caller, #method "eth_getBlockByNumber");
        let res = await evm.eth_getBlockByNumber(service, config, arg);
        await* pool.addCycles(caller, #refund);
        res
    };
    public shared ({ caller }) func eth_getLogs(service: EVM.RpcServices, config: ?EVM.RpcConfig, arg: EVM.GetLogsArgs) : async EVM.MultiGetLogsResult {
        await* pool.addCycles(caller, #method "eth_getLogs");
        let res = await evm.eth_getLogs(service, config, arg);
        await* pool.addCycles(caller, #refund);
        res
    };
    public shared ({ caller }) func eth_getTransactionCount(service: EVM.RpcServices, config: ?EVM.RpcConfig, arg: EVM.GetTransactionCountArgs) : async EVM.MultiGetTransactionCountResult {
        await* pool.addCycles(caller, #method "eth_getTransactionCount");
        let res = await evm.eth_getTransactionCount(service, config, arg);
        await* pool.addCycles(caller, #refund);
        res
    };
    public shared ({ caller }) func eth_getTransactionReceipt(service: EVM.RpcServices, config: ?EVM.RpcConfig, arg: Text) : async EVM.MultiGetTransactionReceiptResult {
        await* pool.addCycles(caller, #method "eth_getTransactionReceipt");
        let res = await evm.eth_getTransactionReceipt(service, config, arg);
        await* pool.addCycles(caller, #refund);
        res
    };
    public shared ({ caller }) func eth_sendRawTransaction(service: EVM.RpcServices, config: ?EVM.RpcConfig, arg: Text) : async EVM.MultiSendRawTransactionResult {
        await* pool.addCycles(caller, #method "eth_sendRawTransaction");
        let res = await evm.eth_sendRawTransaction(service, config, arg);
        await* pool.addCycles(caller, #refund);
        res
    };
    public shared ({ caller }) func request(service: EVM.RpcService, arg: Text, id: Nat64) : async EVM.RequestResult {
        await* pool.addCycles(caller, #method "request");
        let res = await evm.request(service, arg, id);
        await* pool.addCycles(caller, #refund);
        res
    };

    system func inspect({
        caller : Principal;
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
            #installStoredWasm : Any;
            #installCode : Any;
            #deployCanister : Any;
            #releaseAllCanisters : Any;
            #removeCode : Any;
            #resetStats : Any;
            #mergeTags : Any;
            #wallet_receive : Any;
            #takeSnapshot : Any;
            #loadSnapshot : Any;
            #deleteSnapshot : Any;
            #listSnapshots : Any;
            #transferOwnership : Any;
            #installExternalCanister : Any;

            #create_canister : Any;
            #update_settings : Any;
            #install_code : Any;
            #uninstall_code : Any;
            #canister_status : Any;
            #start_canister : Any;
            #stop_canister : Any;
            #delete_canister : Any;
            #list_canister_snapshots : Any;
            #take_canister_snapshot : Any;
            #delete_canister_snapshot : Any;
            #load_canister_snapshot : Any;
            #_ttp_request : Any;
            #__transform : Any;
            #sign_with_ecdsa: Any;
            #sign_with_schnorr: Any;
            #eth_call: Any;
            #eth_feeHistory: Any;
            #eth_getBlockByNumber: Any;
            #eth_getLogs: Any;
            #eth_getTransactionCount: Any;
            #eth_getTransactionReceipt: Any;
            #eth_sendRawTransaction: Any;
            #request: Any;
        };
    }) : Bool {
        switch msg {
            case (#create_canister _) false;
            case (#install_code _) false;
            case (#uninstall_code _) false;
            case (#start_canister _) false;
            case (#stop_canister _) false;
            case (#delete_canister _) false;
            case (#list_canister_snapshots _) false;
            case (#take_canister_snapshot _) false;
            case (#delete_canister_snapshot _) false;
            case (#load_canister_snapshot _) false;
            case (#_ttp_request _) false;
            case (#__transform _) false;
            case (#sign_with_ecdsa _) false;
            case (#sign_with_schnorr _) false;
            case (#eth_call _) false;
            case (#eth_feeHistory _) false;
            case (#eth_getBlockByNumber _) false;
            case (#eth_getLogs _) false;
            case (#eth_getTransactionCount _) false;
            case (#eth_getTransactionReceipt _) false;
            case (#eth_sendRawTransaction _) false;
            case (#request _) false;
            case (#canister_status _) Principal.isController(caller);
            case (#update_settings _) Principal.isController(caller);
            case _ true;
        };
    };
};
