import { BinaryBlob } from './blob';
import { CanisterId } from './canisterId';
import { AsyncRequest } from './request';
import { SubmitRequestType } from './submit_request_type';

// The fields in a "call" submit request.
export interface CallRequest extends AsyncRequest {
  request_type: SubmitRequestType.Call;
  canister_id: CanisterId;
  method_name: string;
  arg: BinaryBlob;
}
