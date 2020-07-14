import { ActorFactory } from '../actor';
import { CanisterId } from '../canisterId';
import {
  QueryFields,
  QueryResponse,
  RequestStatusFields,
  RequestStatusResponse,
  SubmitResponse,
} from '../http_agent_types';
import * as IDL from '../idl';
import { Principal } from '../principal';
import { BinaryBlob, JsonObject } from '../types';

// An Agent able to make calls and queries to a Replica.
export interface Agent {
  requestStatus(fields: RequestStatusFields, principal?: Principal): Promise<RequestStatusResponse>;

  call(
    canisterId: CanisterId | string,
    fields: {
      methodName: string;
      arg: BinaryBlob;
    },
    principal?: Principal | Promise<Principal>,
  ): Promise<SubmitResponse>;

  createCanister(principal?: Principal): Promise<SubmitResponse>;

  status(): Promise<JsonObject>;

  install(
    canisterId: CanisterId | string,
    fields: {
      module: BinaryBlob;
      arg?: BinaryBlob;
    },
    principal?: Principal,
  ): Promise<SubmitResponse>;

  query(
    canisterId: CanisterId | string,
    fields: QueryFields,
    principal?: Principal,
  ): Promise<QueryResponse>;

  makeActorFactory(actorInterfaceFactory: IDL.InterfaceFactory): ActorFactory;
}
