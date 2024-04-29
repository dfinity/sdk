import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface ItemDataWithoutOwner {
  'title' : string,
  'locale' : string,
  'description' : string,
  'details' : ItemDetails,
  'price' : number,
}
export type ItemDetails = { 'link' : string } |
  { 'post' : null } |
  { 'message' : null } |
  { 'folder' : null };
export interface ItemTransferWithoutOwner {
  'data' : ItemDataWithoutOwner,
  'communal' : boolean,
}
export interface User {
  'title' : string,
  'link' : string,
  'nick' : string,
  'locale' : string,
  'description' : string,
}
export interface ZonBackend {
  'createItemData' : ActorMethod<
    [ItemTransferWithoutOwner],
    [Principal, bigint]
  >,
  'getRootItem' : ActorMethod<[], [] | [[Principal, bigint]]>,
  'get_trusted_origins' : ActorMethod<[], Array<string>>,
  'init' : ActorMethod<[], undefined>,
  'removeItem' : ActorMethod<[Principal, bigint], undefined>,
  'removeMainOwner' : ActorMethod<[], undefined>,
  'removeUser' : ActorMethod<[Principal], undefined>,
  'setItemData' : ActorMethod<
    [Principal, bigint, ItemDataWithoutOwner],
    undefined
  >,
  'setMainOwner' : ActorMethod<[Principal], undefined>,
  'setPostText' : ActorMethod<[Principal, bigint, string], undefined>,
  'setRootItem' : ActorMethod<[Principal, bigint], undefined>,
  'setUserData' : ActorMethod<[[] | [Principal], User], undefined>,
}
export interface _SERVICE extends ZonBackend {}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
