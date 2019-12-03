import { BinaryBlob } from './blob';
import { CanisterId } from './canisterId';
import { ReadRequestType } from './readRequestType';
import { Request } from './request';

// The fields in a "query" read request.
export interface QueryRequest extends Request {
  request_type: ReadRequestType.Query;
  canister_id: CanisterId;
  method_name: string;
  arg: BinaryBlob;
}
