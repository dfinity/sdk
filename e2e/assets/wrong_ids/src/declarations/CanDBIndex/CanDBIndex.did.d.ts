import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export type AttributeKey = string;
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
export interface CanDBIndex {
  'autoScaleCanister' : ActorMethod<[string], string>,
  'checkSybil' : ActorMethod<[Principal], undefined>,
  'getCanistersByPK' : ActorMethod<[string], Array<string>>,
  'getFirstAttribute' : ActorMethod<
    [string, { 'sk' : SK, 'key' : AttributeKey }],
    [] | [[Principal, [] | [AttributeValue]]]
  >,
  'getOwners' : ActorMethod<[], Array<Principal>>,
  'init' : ActorMethod<[Array<Principal>], undefined>,
  'putAttributeNoDuplicates' : ActorMethod<
    [string, { 'sk' : SK, 'key' : AttributeKey, 'value' : AttributeValue }],
    Principal
  >,
  'putAttributeWithPossibleDuplicate' : ActorMethod<
    [string, { 'sk' : SK, 'key' : AttributeKey, 'value' : AttributeValue }],
    Principal
  >,
  'setOwners' : ActorMethod<[Array<Principal>], undefined>,
  'setVotingData' : ActorMethod<
    [Principal, [] | [Principal], VotingScore],
    undefined
  >,
  'sybilScore' : ActorMethod<[], [boolean, number]>,
  'upgradeAllPartitionCanisters' : ActorMethod<
    [Uint8Array | number[]],
    UpgradePKRangeResult
  >,
}
export type InterCanisterActionResult = { 'ok' : null } |
  { 'err' : string };
export type SK = string;
export type Time = bigint;
export interface UpgradePKRangeResult {
  'nextKey' : [] | [string],
  'upgradeCanisterResults' : Array<[string, InterCanisterActionResult]>,
}
export interface VotingScore {
  'ethereumAddress' : string,
  'lastChecked' : Time,
  'points' : number,
}
export interface _SERVICE extends CanDBIndex {}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
