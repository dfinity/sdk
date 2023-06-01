import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface Config {
  'backend_canister_id' : [] | [Principal],
  'remove_cycles_add' : boolean,
  'profiling' : boolean,
  'limit_stable_memory_page' : [] | [number],
}
export interface _SERVICE {
  'transform' : ActorMethod<
    [Uint8Array | number[], Config],
    Uint8Array | number[]
  >,
}
