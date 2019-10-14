import { BinaryBlob } from "./blob";
import { CanisterId } from "./canisterId";
import { AsyncRequest } from "./request";
import { SubmitRequestType } from "./submitRequestType";

// The fields in a "call" submit request.
export interface CallRequest extends AsyncRequest {
  request_type: SubmitRequestType.Call;
  canister_id: CanisterId;
  method_name: string;
  arg: BinaryBlob;
}
