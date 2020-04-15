import { CanisterId } from './canisterId';
import { RejectCode } from './reject_code';
import { RequestId } from './request_id';
import { BinaryBlob } from './types';

export const enum Endpoint {
  Read = 'read',
  Submit = 'submit',
}

// An HttpAgent request, before it gets encoded and sent to the server.
// We create an empty request that we will fill later.
export type HttpAgentRequest = HttpAgentReadRequest | HttpAgentSubmitRequest;

export interface HttpAgentBaseRequest {
  readonly endpoint: Endpoint;
  request: RequestInit;
}

export interface HttpAgentSubmitRequest extends HttpAgentBaseRequest {
  readonly endpoint: Endpoint.Submit;
  body: SubmitRequest;
}

export interface HttpAgentReadRequest extends HttpAgentBaseRequest {
  readonly endpoint: Endpoint.Read;
  body: ReadRequest;
}

export type SignedHttpAgentRequest = SignedHttpAgentReadRequest | SignedHttpAgentSubmitRequest;

export interface SignedHttpAgentSubmitRequest extends HttpAgentBaseRequest {
  readonly endpoint: Endpoint.Submit;
  body: Signed<SubmitRequest>;
}

export interface SignedHttpAgentReadRequest extends HttpAgentBaseRequest {
  readonly endpoint: Endpoint.Read;
  body: Signed<ReadRequest>;
}

export interface Signed<T> {
  content: T;
  sender_pubkey: BinaryBlob;
  sender_sig: BinaryBlob;
}

export interface HttpAgentRequestTransformFn {
  (args: HttpAgentRequest): Promise<HttpAgentRequest | undefined | void>;
  priority?: number;
}

export type AuthHttpAgentRequestTransformFn =
  (args: HttpAgentRequest) => Promise<SignedHttpAgentRequest>;


export interface QueryFields {
  methodName: string;
  arg: BinaryBlob;
}
export interface ResponseStatusFields {
  requestId: RequestId;
}

// The fields in a "call" submit request.
// tslint:disable:camel-case
export interface CallRequest extends Record<string, any> {
  request_type: SubmitRequestType.Call;
  canister_id: CanisterId;
  method_name: string;
  arg: BinaryBlob;
}
export interface InstallCodeRequest extends Record<string, any> {
  request_type: SubmitRequestType.InstallCode;
  canister_id: CanisterId;
  module: BinaryBlob;
  arg?: BinaryBlob;
}
// tslint:enable:camel-case

// The types of values allowed in the `request_type` field for submit requests.
export enum SubmitRequestType {
  Call = 'call',
  InstallCode = 'install_code',
}

export type SubmitRequest = CallRequest | InstallCodeRequest;
export interface SubmitResponse {
  requestId: RequestId;
  response: Response;
}

// An ADT that represents responses to a "query" read request.
export type QueryResponse = QueryResponseReplied | QueryResponseRejected;

export interface QueryResponseBase {
  status: QueryResponseStatus;
}

export interface QueryResponseReplied extends QueryResponseBase {
  status: QueryResponseStatus.Replied;
  reply: { arg: BinaryBlob };
}

export interface QueryResponseRejected extends QueryResponseBase {
  status: QueryResponseStatus.Rejected;
  reject_code: RejectCode;
  reject_message: string;
}

export const enum QueryResponseStatus {
  Replied = 'replied',
  Rejected = 'rejected',
}

// The types of values allowed in the `request_type` field for read requests.
export const enum ReadRequestType {
  Query = 'query',
  RequestStatus = 'request_status',
}

// The fields in a "query" read request.
export interface QueryRequest extends Record<string, any> {
  request_type: ReadRequestType.Query;
  canister_id: CanisterId;
  method_name: string;
  arg: BinaryBlob;
}

// The fields in a "request_status" read request.
export interface RequestStatusRequest extends Record<string, any> {
  request_type: ReadRequestType.RequestStatus;
  request_id: RequestId;
}

// An ADT that represents responses to a "request_status" read request.
export type RequestStatusResponse =
  | RequestStatusResponsePending
  | RequestStatusResponseReplied
  | RequestStatusResponseRejected
  | RequestStatusResponseUnknown;

export interface RequestStatusResponsePending {
  status: RequestStatusResponseStatus.Pending;
}

export interface RequestStatusResponseReplied {
  status: RequestStatusResponseStatus.Replied;
  reply: { arg?: BinaryBlob };
}

export interface RequestStatusResponseRejected {
  status: RequestStatusResponseStatus.Rejected;
  reject_code: RejectCode;
  reject_message: string;
}

export interface RequestStatusResponseUnknown {
  status: RequestStatusResponseStatus.Unknown;
}

export enum RequestStatusResponseStatus {
  Pending = 'pending',
  Replied = 'replied',
  Rejected = 'rejected',
  Unknown = 'unknown',
}

export type ReadRequest = QueryRequest | RequestStatusRequest;
export type ReadResponse = QueryResponse | RequestStatusResponse;
