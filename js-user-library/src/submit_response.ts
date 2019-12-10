import { RequestId } from './request_id';
import { Response } from './response';

export interface SubmitResponse extends Response {
  requestId: RequestId;
  response: Response;
}
