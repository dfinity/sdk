import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export type Fraction = bigint;
export interface Payments {
  'getBuyerAffiliateShare' : ActorMethod<[], Fraction>,
  'getOurDebt' : ActorMethod<[Principal], bigint>,
  'getOwners' : ActorMethod<[], Array<Principal>>,
  'getSalesOwnersShare' : ActorMethod<[], Fraction>,
  'getSellerAffiliateShare' : ActorMethod<[], Fraction>,
  'getUploadOwnersShare' : ActorMethod<[], Fraction>,
  'getUpvotesOwnersShare' : ActorMethod<[], Fraction>,
  'init' : ActorMethod<[Array<Principal>], undefined>,
  'payout' : ActorMethod<[[] | [Subaccount]], undefined>,
  'setBuyerAffiliateShare' : ActorMethod<[Fraction], undefined>,
  'setOwners' : ActorMethod<[Array<Principal>], undefined>,
  'setSalesOwnersShare' : ActorMethod<[Fraction], undefined>,
  'setSellerAffiliateShare' : ActorMethod<[Fraction], undefined>,
  'setUploadOwnersShare' : ActorMethod<[Fraction], undefined>,
  'setUpvotesOwnersShare' : ActorMethod<[Fraction], undefined>,
}
export type Subaccount = Uint8Array | number[];
export interface _SERVICE extends Payments {}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
