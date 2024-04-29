import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export type AttributeValue = { 'int' : bigint } |
  { 'float' : number } |
  { 'tuple' : Array<AttributeValuePrimitive> } |
  { 'bool' : boolean } |
  { 'text' : string } |
  { 'arrayBool' : Array<boolean> } |
  { 'arrayText' : Array<string> } |
  { 'arrayInt' : Array<bigint> } |
  { 'arrayFloat' : Array<number> };
export type AttributeValuePrimitive = { 'int' : bigint } |
  { 'float' : number } |
  { 'bool' : boolean } |
  { 'text' : string };
export type Direction = { 'bwd' : null } |
  { 'fwd' : null };
export interface GetByOuterPartitionKeyOptions {
  'sk' : SK,
  'outer' : OuterPair,
}
export interface GetUserDataOuterOptions { 'outer' : OuterPair }
export type InnerSubDBKey = bigint;
export interface NacDBIndex {
  'createPartition' : ActorMethod<[], Principal>,
  'createPartitionImpl' : ActorMethod<[], Principal>,
  'createSubDB' : ActorMethod<
    [Uint8Array | number[], { 'userData' : string, 'hardCap' : [] | [bigint] }],
    {
      'outer' : { 'key' : OuterSubDBKey, 'canister' : Principal },
      'inner' : { 'key' : InnerSubDBKey, 'canister' : Principal },
    }
  >,
  'delete' : ActorMethod<
    [
      Uint8Array | number[],
      { 'sk' : SK, 'outerKey' : OuterSubDBKey, 'outerCanister' : Principal },
    ],
    undefined
  >,
  'deleteSubDB' : ActorMethod<
    [
      Uint8Array | number[],
      { 'outerKey' : OuterSubDBKey, 'outerCanister' : Principal },
    ],
    undefined
  >,
  'getAllItemsStream' : ActorMethod<[], Order>,
  'getCanisters' : ActorMethod<[], Array<Principal>>,
  'getOwners' : ActorMethod<[], Array<Principal>>,
  'init' : ActorMethod<[Array<Principal>], undefined>,
  'insert' : ActorMethod<
    [
      Uint8Array | number[],
      {
        'sk' : SK,
        'value' : AttributeValue,
        'hardCap' : [] | [bigint],
        'outerKey' : OuterSubDBKey,
        'outerCanister' : Principal,
      },
    ],
    Result
  >,
  'setOwners' : ActorMethod<[Array<Principal>], undefined>,
  'upgradeCanistersInRange' : ActorMethod<
    [Uint8Array | number[], bigint, bigint],
    undefined
  >,
}
export interface Order {
  'reverse' : [OuterCanister, OuterSubDBKey],
  'order' : [OuterCanister, OuterSubDBKey],
}
export interface OuterCanister {
  'createOuter' : ActorMethod<
    [
      {
        'part' : Principal,
        'outerKey' : OuterSubDBKey,
        'innerKey' : InnerSubDBKey,
      },
    ],
    {
      'outer' : { 'key' : OuterSubDBKey, 'canister' : Principal },
      'inner' : { 'key' : InnerSubDBKey, 'canister' : Principal },
    }
  >,
  'deleteInner' : ActorMethod<
    [{ 'sk' : SK, 'innerKey' : InnerSubDBKey }],
    undefined
  >,
  'deleteSubDBInner' : ActorMethod<[{ 'innerKey' : InnerSubDBKey }], undefined>,
  'deleteSubDBOuter' : ActorMethod<[{ 'outerKey' : OuterSubDBKey }], undefined>,
  'getByInner' : ActorMethod<
    [{ 'sk' : SK, 'innerKey' : InnerSubDBKey }],
    [] | [AttributeValue]
  >,
  'getByOuter' : ActorMethod<
    [{ 'sk' : SK, 'outerKey' : OuterSubDBKey }],
    [] | [AttributeValue]
  >,
  'getInner' : ActorMethod<
    [{ 'outerKey' : OuterSubDBKey }],
    [] | [{ 'key' : InnerSubDBKey, 'canister' : Principal }]
  >,
  'getOuter' : ActorMethod<
    [GetByOuterPartitionKeyOptions],
    [] | [AttributeValue]
  >,
  'getSubDBUserDataInner' : ActorMethod<
    [{ 'innerKey' : InnerSubDBKey }],
    [] | [string]
  >,
  'getSubDBUserDataOuter' : ActorMethod<
    [GetUserDataOuterOptions],
    [] | [string]
  >,
  'hasByInner' : ActorMethod<
    [{ 'sk' : SK, 'innerKey' : InnerSubDBKey }],
    boolean
  >,
  'hasByOuter' : ActorMethod<
    [{ 'sk' : SK, 'outerKey' : OuterSubDBKey }],
    boolean
  >,
  'hasSubDBByInner' : ActorMethod<[{ 'innerKey' : InnerSubDBKey }], boolean>,
  'hasSubDBByOuter' : ActorMethod<[{ 'outerKey' : OuterSubDBKey }], boolean>,
  'isOverflowed' : ActorMethod<[], boolean>,
  'putLocation' : ActorMethod<
    [
      {
        'newInnerSubDBKey' : InnerSubDBKey,
        'innerCanister' : Principal,
        'outerKey' : OuterSubDBKey,
      },
    ],
    undefined
  >,
  'rawDeleteSubDB' : ActorMethod<[{ 'innerKey' : InnerSubDBKey }], undefined>,
  'rawGetSubDB' : ActorMethod<
    [{ 'innerKey' : InnerSubDBKey }],
    [] | [{ 'map' : Array<[SK, AttributeValue]>, 'userData' : string }]
  >,
  'rawInsertSubDB' : ActorMethod<
    [
      {
        'map' : Array<[SK, AttributeValue]>,
        'userData' : string,
        'hardCap' : [] | [bigint],
        'innerKey' : [] | [InnerSubDBKey],
      },
    ],
    { 'innerKey' : InnerSubDBKey }
  >,
  'rawInsertSubDBAndSetOuter' : ActorMethod<
    [
      {
        'map' : Array<[SK, AttributeValue]>,
        'userData' : string,
        'keys' : [] | [
          { 'outerKey' : OuterSubDBKey, 'innerKey' : InnerSubDBKey }
        ],
        'hardCap' : [] | [bigint],
      },
    ],
    { 'outerKey' : OuterSubDBKey, 'innerKey' : InnerSubDBKey }
  >,
  'scanLimitInner' : ActorMethod<
    [
      {
        'dir' : Direction,
        'lowerBound' : SK,
        'limit' : bigint,
        'upperBound' : SK,
        'innerKey' : InnerSubDBKey,
      },
    ],
    ScanLimitResult
  >,
  'scanLimitOuter' : ActorMethod<
    [
      {
        'dir' : Direction,
        'lowerBound' : SK,
        'limit' : bigint,
        'upperBound' : SK,
        'outerKey' : OuterSubDBKey,
      },
    ],
    ScanLimitResult
  >,
  'scanSubDBs' : ActorMethod<
    [],
    Array<[OuterSubDBKey, { 'key' : InnerSubDBKey, 'canister' : Principal }]>
  >,
  'startInsertingImpl' : ActorMethod<
    [{ 'sk' : SK, 'value' : AttributeValue, 'innerKey' : InnerSubDBKey }],
    undefined
  >,
  'subDBSizeByInner' : ActorMethod<
    [{ 'innerKey' : InnerSubDBKey }],
    [] | [bigint]
  >,
  'subDBSizeByOuter' : ActorMethod<
    [{ 'outerKey' : OuterSubDBKey }],
    [] | [bigint]
  >,
  'subDBSizeOuterImpl' : ActorMethod<[SubDBSizeOuterOptions], [] | [bigint]>,
  'superDBSize' : ActorMethod<[], bigint>,
}
export interface OuterPair { 'key' : OuterSubDBKey, 'canister' : OuterCanister }
export type OuterSubDBKey = bigint;
export type Result = {
    'ok' : {
      'outer' : { 'key' : OuterSubDBKey, 'canister' : Principal },
      'inner' : { 'key' : InnerSubDBKey, 'canister' : Principal },
    }
  } |
  { 'err' : string };
export type SK = string;
export interface ScanLimitResult {
  'results' : Array<[string, AttributeValue]>,
  'nextKey' : [] | [string],
}
export interface SubDBSizeOuterOptions { 'outer' : OuterPair }
export interface _SERVICE extends NacDBIndex {}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
