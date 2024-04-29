import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface _SERVICE {
  'erc1155_balance_of' : ActorMethod<[string, string, string, bigint], bigint>,
  'erc721_owner_of' : ActorMethod<[string, string, bigint], string>,
  'verify_ecdsa' : ActorMethod<[string, string, string], boolean>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
