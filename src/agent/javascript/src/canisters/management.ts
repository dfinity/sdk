import { Actor, CallConfig } from '../actor';
import managementCanisterIdl from './management_idl';
import { Principal } from 'src/principal';

/* tslint:disable */
export interface ManagementCanisterRecord {
  create_canister(): Promise<{ canister_id: Principal }>;
  install_code(arg0: {
    mode: { install: null } | { reinstall: null } | { upgrade: null };
    canister_id: Principal;
    wasm_module: number[];
    arg: number[];
    compute_allocation: [] | [number];
    memory_allocation: [] | [number];
  }): Promise<void>;
}
/* tslint:enable */

/**
 * Create a management canister actor.
 * @param config
 */
export function getManagementCanister(config: CallConfig) {
  return Actor.createActor<ManagementCanisterRecord>(managementCanisterIdl, {
    ...config,
    canisterId: Principal.fromHexWithChecksum(''),
  });
}
