import { BinaryBlob } from './blob';
import { RejectCode } from './reject_code';
import { Response } from './response';

// An ADT that represents responses to a "request-status" read request.
export type RequestStatusResponse =
  | RequestStatusResponsePending
  | RequestStatusResponseReplied
  | RequestStatusResponseRejected
  | RequestStatusResponseUnknown;

interface RequestStatusResponsePending extends Response {
  status: RequestStatusResponseStatus.Pending;
}

interface RequestStatusResponseReplied extends Response {
  status: RequestStatusResponseStatus.Replied;
  reply: { arg: BinaryBlob };
}

interface RequestStatusResponseRejected extends Response {
  status: RequestStatusResponseStatus.Rejected;
  reject_code: RejectCode;
  reject_message: string;
}

interface RequestStatusResponseUnknown extends Response {
  status: RequestStatusResponseStatus.Unknown;
}

export enum RequestStatusResponseStatus {
  Pending = 'pending',
  Replied = 'replied',
  Rejected = 'rejected',
  Unknown = 'unknown',
}
