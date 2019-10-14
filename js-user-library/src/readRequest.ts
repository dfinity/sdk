import { QueryRequest } from "./queryRequest";
import { RequestStatusRequest } from "./requestStatusRequest";

export type ReadRequest
  = QueryRequest
  | RequestStatusRequest;
