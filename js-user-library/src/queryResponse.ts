import { BinaryBlob } from './blob';
import { RejectCode } from './rejectCode';
import { Response } from './response';

// An ADT that represents responses to a "query" read request.
export type QueryResponse = QueryResponseReplied | QueryResponseRejected;

interface QueryResponseReplied extends Response {
  status: QueryResponseStatus.Replied;
  reply: { arg: BinaryBlob };
}

interface QueryResponseRejected extends Response {
  status: QueryResponseStatus.Rejected;
  reject_code: RejectCode;
  reject_message: string;
}

enum QueryResponseStatus {
  Replied = 'replied',
  Rejected = 'rejected',
}
