import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export type AttributeKey = string;
export type AttributeMap = { 'leaf' : null } |
  { 'node' : [Color, Tree, [AttributeKey, [] | [AttributeValue]], Tree] };
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
export type AttributeValuePrimitive__1 = { 'int' : bigint } |
  { 'float' : number } |
  { 'bool' : boolean } |
  { 'text' : string };
export type AttributeValue__1 = { 'int' : bigint } |
  { 'float' : number } |
  { 'tuple' : Array<AttributeValuePrimitive__1> } |
  { 'bool' : boolean } |
  { 'text' : string } |
  { 'arrayBool' : Array<boolean> } |
  { 'arrayText' : Array<string> } |
  { 'arrayInt' : Array<bigint> } |
  { 'arrayFloat' : Array<number> };
export type AutoScalingCanisterSharedFunctionHook = ActorMethod<
  [string],
  string
>;
export interface CanDBPartition {
  'delete' : ActorMethod<[DeleteOptions], undefined>,
  'get' : ActorMethod<[GetOptions], [] | [Entity]>,
  'getAttribute' : ActorMethod<[GetOptions, string], [] | [AttributeValue]>,
  'getItem' : ActorMethod<[bigint], [] | [ItemTransfer]>,
  'getOwners' : ActorMethod<[], Array<Principal>>,
  'getPK' : ActorMethod<[], string>,
  'getStreams' : ActorMethod<[bigint, string], [] | [Streams]>,
  'put' : ActorMethod<[PutOptions], undefined>,
  'putAttribute' : ActorMethod<
    [{ 'sk' : SK, 'key' : AttributeKey, 'value' : AttributeValue }],
    undefined
  >,
  'putExisting' : ActorMethod<[PutOptions], boolean>,
  'putExistingAttribute' : ActorMethod<
    [{ 'sk' : SK, 'key' : AttributeKey, 'value' : AttributeValue }],
    boolean
  >,
  'scan' : ActorMethod<[ScanOptions], ScanResult>,
  'setOwners' : ActorMethod<[Array<Principal>], undefined>,
  'skExists' : ActorMethod<[string], boolean>,
  'transferCycles' : ActorMethod<[], undefined>,
}
export type Color = { 'B' : null } |
  { 'R' : null };
export interface DeleteOptions { 'sk' : SK }
export type Direction = { 'bwd' : null } |
  { 'fwd' : null };
export interface Entity { 'pk' : PK, 'sk' : SK, 'attributes' : AttributeMap }
export interface GetByOuterPartitionKeyOptions {
  'sk' : SK__1,
  'outer' : OuterPair,
}
export interface GetOptions { 'sk' : SK }
export interface GetUserDataOuterOptions { 'outer' : OuterPair }
export type InnerSubDBKey = bigint;
export interface ItemData {
  'creator' : Principal,
  'edited' : boolean,
  'item' : ItemDataWithoutOwner,
}
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
export interface ItemTransfer { 'data' : ItemData, 'communal' : boolean }
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
    [{ 'sk' : SK__1, 'innerKey' : InnerSubDBKey }],
    undefined
  >,
  'deleteSubDBInner' : ActorMethod<[{ 'innerKey' : InnerSubDBKey }], undefined>,
  'deleteSubDBOuter' : ActorMethod<[{ 'outerKey' : OuterSubDBKey }], undefined>,
  'getByInner' : ActorMethod<
    [{ 'sk' : SK__1, 'innerKey' : InnerSubDBKey }],
    [] | [AttributeValue__1]
  >,
  'getByOuter' : ActorMethod<
    [{ 'sk' : SK__1, 'outerKey' : OuterSubDBKey }],
    [] | [AttributeValue__1]
  >,
  'getInner' : ActorMethod<
    [{ 'outerKey' : OuterSubDBKey }],
    [] | [{ 'key' : InnerSubDBKey, 'canister' : Principal }]
  >,
  'getOuter' : ActorMethod<
    [GetByOuterPartitionKeyOptions],
    [] | [AttributeValue__1]
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
    [{ 'sk' : SK__1, 'innerKey' : InnerSubDBKey }],
    boolean
  >,
  'hasByOuter' : ActorMethod<
    [{ 'sk' : SK__1, 'outerKey' : OuterSubDBKey }],
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
    [] | [{ 'map' : Array<[SK__1, AttributeValue__1]>, 'userData' : string }]
  >,
  'rawInsertSubDB' : ActorMethod<
    [
      {
        'map' : Array<[SK__1, AttributeValue__1]>,
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
        'map' : Array<[SK__1, AttributeValue__1]>,
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
        'lowerBound' : SK__1,
        'limit' : bigint,
        'upperBound' : SK__1,
        'innerKey' : InnerSubDBKey,
      },
    ],
    ScanLimitResult
  >,
  'scanLimitOuter' : ActorMethod<
    [
      {
        'dir' : Direction,
        'lowerBound' : SK__1,
        'limit' : bigint,
        'upperBound' : SK__1,
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
    [{ 'sk' : SK__1, 'value' : AttributeValue__1, 'innerKey' : InnerSubDBKey }],
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
export type PK = string;
export interface PutOptions {
  'sk' : SK,
  'attributes' : Array<[AttributeKey, AttributeValue]>,
}
export type SK = string;
export type SK__1 = string;
export type ScalingLimitType = { 'heapSize' : bigint } |
  { 'count' : bigint };
export interface ScalingOptions {
  'autoScalingHook' : AutoScalingCanisterSharedFunctionHook,
  'sizeLimit' : ScalingLimitType,
}
export interface ScanLimitResult {
  'results' : Array<[string, AttributeValue__1]>,
  'nextKey' : [] | [string],
}
export interface ScanOptions {
  'limit' : bigint,
  'ascending' : [] | [boolean],
  'skLowerBound' : SK,
  'skUpperBound' : SK,
}
export interface ScanResult {
  'entities' : Array<Entity>,
  'nextKey' : [] | [SK],
}
export type Streams = Array<[] | [Order]>;
export interface SubDBSizeOuterOptions { 'outer' : OuterPair }
export type Tree = { 'leaf' : null } |
  { 'node' : [Color, Tree, [AttributeKey, [] | [AttributeValue]], Tree] };
export interface _SERVICE extends CanDBPartition {}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
