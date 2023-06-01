export const idlFactory = ({ IDL }) => {
  const InitParams = IDL.Record({
    'max_num_canisters' : IDL.Nat,
    'canister_time_to_live' : IDL.Nat,
    'cycles_per_canister' : IDL.Nat,
    'nonce_time_to_live' : IDL.Nat,
    'max_family_tree_size' : IDL.Nat,
  });
  const CanisterInfo = IDL.Record({
    'id' : IDL.Principal,
    'timestamp' : IDL.Int,
  });
  const canister_id = IDL.Principal;
  const definite_canister_settings = IDL.Record({
    'freezing_threshold' : IDL.Nat,
    'controllers' : IDL.Vec(IDL.Principal),
    'memory_allocation' : IDL.Nat,
    'compute_allocation' : IDL.Nat,
  });
  const canister_settings = IDL.Record({
    'freezing_threshold' : IDL.Opt(IDL.Nat),
    'controllers' : IDL.Opt(IDL.Vec(IDL.Principal)),
    'memory_allocation' : IDL.Opt(IDL.Nat),
    'compute_allocation' : IDL.Opt(IDL.Nat),
  });
  const Nonce = IDL.Record({ 'nonce' : IDL.Nat, 'timestamp' : IDL.Int });
  const Stats = IDL.Record({
    'num_of_installs' : IDL.Nat,
    'num_of_canisters' : IDL.Nat,
    'error_mismatch' : IDL.Nat,
    'error_out_of_capacity' : IDL.Nat,
    'cycles_used' : IDL.Nat,
    'error_total_wait_time' : IDL.Nat,
  });
  const HttpRequest = IDL.Record({
    'url' : IDL.Text,
    'method' : IDL.Text,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
  });
  const HttpResponse = IDL.Record({
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'status_code' : IDL.Nat16,
  });
  const InstallArgs = IDL.Record({
    'arg' : IDL.Vec(IDL.Nat8),
    'wasm_module' : IDL.Vec(IDL.Nat8),
    'mode' : IDL.Variant({
      'reinstall' : IDL.Null,
      'upgrade' : IDL.Null,
      'install' : IDL.Null,
    }),
    'canister_id' : IDL.Principal,
  });
  const wasm_module = IDL.Vec(IDL.Nat8);
  const Self = IDL.Service({
    'GCCanisters' : IDL.Func([], [], ['oneway']),
    'balance' : IDL.Func([], [IDL.Nat], ['query']),
    'callForward' : IDL.Func(
        [CanisterInfo, IDL.Text, IDL.Vec(IDL.Nat8)],
        [IDL.Vec(IDL.Nat8)],
        [],
      ),
    'canister_status' : IDL.Func(
        [IDL.Record({ 'canister_id' : canister_id })],
        [
          IDL.Record({
            'status' : IDL.Variant({
              'stopped' : IDL.Null,
              'stopping' : IDL.Null,
              'running' : IDL.Null,
            }),
            'memory_size' : IDL.Nat,
            'cycles' : IDL.Nat,
            'settings' : definite_canister_settings,
            'module_hash' : IDL.Opt(IDL.Vec(IDL.Nat8)),
          }),
        ],
        [],
      ),
    'create_canister' : IDL.Func(
        [IDL.Record({ 'settings' : IDL.Opt(canister_settings) })],
        [IDL.Record({ 'canister_id' : canister_id })],
        [],
      ),
    'delete_canister' : IDL.Func(
        [IDL.Record({ 'canister_id' : canister_id })],
        [],
        [],
      ),
    'dump' : IDL.Func([], [IDL.Vec(CanisterInfo)], ['query']),
    'getCanisterId' : IDL.Func([Nonce], [CanisterInfo], []),
    'getInitParams' : IDL.Func([], [InitParams], ['query']),
    'getStats' : IDL.Func([], [Stats], ['query']),
    'getSubtree' : IDL.Func(
        [CanisterInfo],
        [IDL.Vec(IDL.Tuple(IDL.Principal, IDL.Vec(CanisterInfo)))],
        ['query'],
      ),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
    'installCode' : IDL.Func(
        [CanisterInfo, InstallArgs, IDL.Bool],
        [CanisterInfo],
        [],
      ),
    'install_code' : IDL.Func(
        [
          IDL.Record({
            'arg' : IDL.Vec(IDL.Nat8),
            'wasm_module' : wasm_module,
            'mode' : IDL.Variant({
              'reinstall' : IDL.Null,
              'upgrade' : IDL.Null,
              'install' : IDL.Null,
            }),
            'canister_id' : canister_id,
          }),
        ],
        [],
        [],
      ),
    'removeCode' : IDL.Func([CanisterInfo], [], []),
    'resetStats' : IDL.Func([], [], []),
    'start_canister' : IDL.Func(
        [IDL.Record({ 'canister_id' : canister_id })],
        [],
        [],
      ),
    'stop_canister' : IDL.Func(
        [IDL.Record({ 'canister_id' : canister_id })],
        [],
        [],
      ),
    'uninstall_code' : IDL.Func(
        [IDL.Record({ 'canister_id' : canister_id })],
        [],
        [],
      ),
    'update_settings' : IDL.Func(
        [
          IDL.Record({
            'canister_id' : canister_id,
            'settings' : canister_settings,
          }),
        ],
        [],
        [],
      ),
    'wallet_receive' : IDL.Func([], [], []),
  });
  return Self;
};
export const init = ({ IDL }) => {
  const InitParams = IDL.Record({
    'max_num_canisters' : IDL.Nat,
    'canister_time_to_live' : IDL.Nat,
    'cycles_per_canister' : IDL.Nat,
    'nonce_time_to_live' : IDL.Nat,
    'max_family_tree_size' : IDL.Nat,
  });
  return [IDL.Opt(InitParams)];
};
