import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface HttpHeader { 'value' : string, 'name' : string }
export interface HttpResponsePayload {
  'status' : bigint,
  'body' : Uint8Array | number[],
  'headers' : Array<HttpHeader>,
}
export interface TransformArgs {
  'context' : Uint8Array | number[],
  'response' : HttpResponsePayload,
}
export interface _SERVICE {
  'getEthereumSigningMessage' : ActorMethod<
    [],
    { 'message' : string, 'nonce' : string }
  >,
  'removeHTTPHeaders' : ActorMethod<[TransformArgs], HttpResponsePayload>,
  'scoreBySignedEthereumAddress' : ActorMethod<
    [{ 'signature' : string, 'address' : string, 'nonce' : string }],
    string
  >,
  'submitSignedEthereumAddressForScore' : ActorMethod<
    [{ 'signature' : string, 'address' : string, 'nonce' : string }],
    string
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
