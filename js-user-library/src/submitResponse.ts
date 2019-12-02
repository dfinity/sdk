import { RequestId } from './requestId';
import { Response } from './response';

export interface SubmitResponse extends Response {
  requestId: RequestId;
  response: Response;
}
