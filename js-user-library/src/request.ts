import { BinaryBlob } from "./blob";
import { RequestType } from "./requestType";

// Common request fields.
export interface Request extends Record<string, any> {
  request_type: RequestType;
  // expiry?:;
  // NOTE: `nonce`, but we provide it so that requests are unique and we avoid a
  // bug in the client when the same request is submitted more than once:
  // https://dfinity.atlassian.net/browse/DFN-895
  nonce?: BinaryBlob;
  // sender:;
  sender_pubkey: BinaryBlob;
  sender_sig: BinaryBlob;
}
