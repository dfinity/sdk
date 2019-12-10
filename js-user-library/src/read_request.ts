import { QueryRequest } from './query_request';
import { RequestStatusRequest } from './request_status_request';

export type ReadRequest = QueryRequest | RequestStatusRequest;
