import { ReadRequestType } from "./readRequestType";
import { Request } from "./request";
import { RequestId } from "./requestId";

// The fields in a "request-status" read request.
export interface RequestStatusRequest extends Request {
  request_type: ReadRequestType.RequestStatus;
  request_id: RequestId;
}
