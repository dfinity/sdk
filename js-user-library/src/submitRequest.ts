import { BinaryBlob } from "./blob";
import { CanisterId } from "./canisterId";

// An ADT that represents requests to the "submit" endpoint.
export type SubmitRequest
  = CallRequest;

// The types of values allowed in the `request_type` field for submit requests.
export enum SubmitRequestType {
  Call = "call",
}

// The fields in a "call" submit request.
interface CallRequest extends Request {
  request_type: SubmitRequestType.Call;
  canister_id: CanisterId;
  method_name: string;
  arg: BinaryBlob;
}
