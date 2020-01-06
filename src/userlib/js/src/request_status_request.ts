import { ReadRequestType } from './read_request_type';
import { Request } from './request';
import { RequestId } from './request_id';

// The fields in a "request-status" read request.
export interface RequestStatusRequest extends Request {
  request_type: ReadRequestType.RequestStatus;
  request_id: RequestId;
}
