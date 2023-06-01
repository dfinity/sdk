import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface CanisterInfo { 'id' : Principal, 'timestamp' : bigint }
export interface HttpRequest {
  'url' : string,
  'method' : string,
  'body' : Uint8Array | number[],
  'headers' : Array<[string, string]>,
}
export interface HttpResponse {
  'body' : Uint8Array | number[],
  'headers' : Array<[string, string]>,
  'status_code' : number,
}
export interface InitParams {
  'max_num_canisters' : bigint,
  'canister_time_to_live' : bigint,
  'cycles_per_canister' : bigint,
  'nonce_time_to_live' : bigint,
  'max_family_tree_size' : bigint,
}
export interface InstallArgs {
  'arg' : Uint8Array | number[],
  'wasm_module' : Uint8Array | number[],
  'mode' : { 'reinstall' : null } |
    { 'upgrade' : null } |
    { 'install' : null },
  'canister_id' : Principal,
}
export interface Nonce { 'nonce' : bigint, 'timestamp' : bigint }
export interface Self {
  'GCCanisters' : ActorMethod<[], undefined>,
  'balance' : ActorMethod<[], bigint>,
  'callForward' : ActorMethod<
    [CanisterInfo, string, Uint8Array | number[]],
    Uint8Array | number[]
  >,
  'canister_status' : ActorMethod<
    [{ 'canister_id' : canister_id }],
    {
      'status' : { 'stopped' : null } |
        { 'stopping' : null } |
        { 'running' : null },
      'memory_size' : bigint,
      'cycles' : bigint,
      'settings' : definite_canister_settings,
      'module_hash' : [] | [Uint8Array | number[]],
    }
  >,
  'create_canister' : ActorMethod<
    [{ 'settings' : [] | [canister_settings] }],
    { 'canister_id' : canister_id }
  >,
  'delete_canister' : ActorMethod<[{ 'canister_id' : canister_id }], undefined>,
  'dump' : ActorMethod<[], Array<CanisterInfo>>,
  'getCanisterId' : ActorMethod<[Nonce], CanisterInfo>,
  'getInitParams' : ActorMethod<[], InitParams>,
  'getStats' : ActorMethod<[], Stats>,
  'getSubtree' : ActorMethod<
    [CanisterInfo],
    Array<[Principal, Array<CanisterInfo>]>
  >,
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
  'installCode' : ActorMethod<
    [CanisterInfo, InstallArgs, boolean],
    CanisterInfo
  >,
  'install_code' : ActorMethod<
    [
      {
        'arg' : Uint8Array | number[],
        'wasm_module' : wasm_module,
        'mode' : { 'reinstall' : null } |
          { 'upgrade' : null } |
          { 'install' : null },
        'canister_id' : canister_id,
      },
    ],
    undefined
  >,
  'removeCode' : ActorMethod<[CanisterInfo], undefined>,
  'resetStats' : ActorMethod<[], undefined>,
  'start_canister' : ActorMethod<[{ 'canister_id' : canister_id }], undefined>,
  'stop_canister' : ActorMethod<[{ 'canister_id' : canister_id }], undefined>,
  'uninstall_code' : ActorMethod<[{ 'canister_id' : canister_id }], undefined>,
  'update_settings' : ActorMethod<
    [{ 'canister_id' : canister_id, 'settings' : canister_settings }],
    undefined
  >,
  'wallet_receive' : ActorMethod<[], undefined>,
}
export interface Stats {
  'num_of_installs' : bigint,
  'num_of_canisters' : bigint,
  'error_mismatch' : bigint,
  'error_out_of_capacity' : bigint,
  'cycles_used' : bigint,
  'error_total_wait_time' : bigint,
}
export type canister_id = Principal;
export interface canister_settings {
  'freezing_threshold' : [] | [bigint],
  'controllers' : [] | [Array<Principal>],
  'memory_allocation' : [] | [bigint],
  'compute_allocation' : [] | [bigint],
}
export interface definite_canister_settings {
  'freezing_threshold' : bigint,
  'controllers' : Array<Principal>,
  'memory_allocation' : bigint,
  'compute_allocation' : bigint,
}
export type wasm_module = Uint8Array | number[];
export interface _SERVICE extends Self {}
