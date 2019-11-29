import { Nonce } from './nonce';
import { RequestType } from './requestType';
import { SenderPubKey } from './senderPubKey';
import { SenderSig } from './senderSig';

export interface AuthFields extends Record<string, any> {
  sender_pubkey: SenderPubKey;
  // TODO(eftychis) We should add the sender field, i.e. entity_id here.
  sender_sig: SenderSig;
}

// TODO: add missing common fields from the spec; `expiry` and `sender`
export interface AsyncFields extends Record<string, any> {
  // NOTE: `nonce` is optional, but we provide it so that requests are unique
  // and we avoid a bug in the client when the same request is submitted more
  // than once: https://dfinity.atlassian.net/browse/DFN-895
  nonce?: Nonce;
}

export interface CommonFields extends Record<string, any> {
  request_type: RequestType;
}

export type Request = AuthFields & CommonFields;

export type AsyncRequest = Request & AsyncFields;
