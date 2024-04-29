import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Orders {
  'addItemToFolder' : ActorMethod<
    [
      [Principal, bigint],
      [Principal, bigint],
      boolean,
      { 'end' : null } |
        { 'beginning' : null },
    ],
    undefined
  >,
  'getOwners' : ActorMethod<[], Array<Principal>>,
  'init' : ActorMethod<[Array<Principal>], undefined>,
  'insertIntoAllTimeStream' : ActorMethod<[[Principal, bigint]], undefined>,
  'removeFromAllTimeStream' : ActorMethod<[[Principal, bigint]], undefined>,
  'removeItemLinks' : ActorMethod<[[Principal, bigint]], undefined>,
  'setOwners' : ActorMethod<[Array<Principal>], undefined>,
  'vote' : ActorMethod<
    [Principal, bigint, Principal, bigint, bigint, boolean],
    undefined
  >,
}
export interface _SERVICE extends Orders {}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
