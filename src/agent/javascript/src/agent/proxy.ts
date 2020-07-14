import {
  ActorFactory,
  BinaryBlob,
  CallFields,
  CanisterId,
  JsonObject,
  Principal,
  QueryFields,
  QueryResponse,
  RequestStatusFields,
  RequestStatusResponse,
  SubmitResponse,
} from '@dfinity/agent';
import * as actor from '../actor';
import * as IDL from '../idl';
import { Agent } from './api';

export enum ProxyMessageKind {
  Error = 'err',
  Query = 'q',
  QueryResponse = 'qr',
  Call = 'c',
  CallResponse = 'cr',
  RequestStatus = 'r',
  RequestStatusResponse = 'rr',
  Status = 's',
  StatusResponse = 'sr',
}

export interface ProxyMessageBase {
  id: number;
  type: ProxyMessageKind;
}

export interface ProxyMessageQuery extends ProxyMessageBase {
  type: ProxyMessageKind.Query;
  args: [string, QueryFields, Principal | undefined];
}

export interface ProxyMessageCall extends ProxyMessageBase {
  type: ProxyMessageKind.Call;
  args: [string, CallFields, Principal | undefined];
}

export interface ProxyMessageRequestStatus extends ProxyMessageBase {
  type: ProxyMessageKind.RequestStatus;
  args: [RequestStatusFields, Principal | undefined];
}

export interface ProxyMessageError extends ProxyMessageBase {
  type: ProxyMessageKind.Error;
  error: any;
}

export interface ProxyMessageQueryResponse extends ProxyMessageBase {
  type: ProxyMessageKind.QueryResponse;
  response: QueryResponse;
}

export interface ProxyMessageCallResponse extends ProxyMessageBase {
  type: ProxyMessageKind.CallResponse;
  response: SubmitResponse;
}

export interface ProxyMessageRequestStatusResponse extends ProxyMessageBase {
  type: ProxyMessageKind.RequestStatusResponse;
  response: RequestStatusResponse;
}

export interface ProxyMessageStatus extends ProxyMessageBase {
  type: ProxyMessageKind.Status;
}

export interface ProxyMessageStatusResponse extends ProxyMessageBase {
  type: ProxyMessageKind.StatusResponse;
  response: JsonObject;
}

export type ProxyMessage =
  | ProxyMessageError
  | ProxyMessageQueryResponse
  | ProxyMessageCallResponse
  | ProxyMessageRequestStatusResponse
  | ProxyMessageQuery
  | ProxyMessageCall
  | ProxyMessageRequestStatus
  | ProxyMessageStatus
  | ProxyMessageStatusResponse;

// A Stub Agent that forwards calls to another Agent implementation.
export class ProxyStubAgent {
  constructor(private _frontend: (msg: ProxyMessage) => void, private _agent: Agent) {}

  public onmessage(msg: ProxyMessage): void {
    switch (msg.type) {
      case ProxyMessageKind.Query:
        this._agent.query(...msg.args).then(response => {
          this._frontend({
            id: msg.id,
            type: ProxyMessageKind.QueryResponse,
            response,
          });
        });
        break;
      case ProxyMessageKind.Call:
        this._agent.call(...msg.args).then(response => {
          this._frontend({
            id: msg.id,
            type: ProxyMessageKind.CallResponse,
            response,
          });
        });
        break;
      case ProxyMessageKind.RequestStatus:
        this._agent.requestStatus(...msg.args).then(response => {
          this._frontend({
            id: msg.id,
            type: ProxyMessageKind.RequestStatusResponse,
            response,
          });
        });
        break;
      case ProxyMessageKind.Status:
        this._agent.status().then(response => {
          this._frontend({
            id: msg.id,
            type: ProxyMessageKind.StatusResponse,
            response,
          });
        });
        break;

      default:
        throw new Error(`Invalid message received: ${JSON.stringify(msg)}`);
    }
  }
}

// An Agent that forwards calls to a backend. The calls are serialized
export class ProxyAgent implements Agent {
  private _nextId = 0;
  private _pendingCalls = new Map<number, [(resolve: any) => void, (reject: any) => void]>();

  constructor(private _backend: (msg: ProxyMessage) => void) {}

  public onmessage(msg: ProxyMessage): void {
    const id = msg.id;

    const maybePromise = this._pendingCalls.get(id);
    if (!maybePromise) {
      throw new Error('A proxy get the same message twice...');
    }

    this._pendingCalls.delete(id);
    const [resolve, reject] = maybePromise;

    switch (msg.type) {
      case ProxyMessageKind.Error:
        return reject(msg.error);
      case ProxyMessageKind.CallResponse:
      case ProxyMessageKind.QueryResponse:
      case ProxyMessageKind.RequestStatusResponse:
        return resolve(msg.response);
      default:
        throw new Error(`Invalid message being sent to ProxyAgent: ${JSON.stringify(msg)}`);
    }
  }

  public requestStatus(
    fields: RequestStatusFields,
    principal?: Principal,
  ): Promise<RequestStatusResponse> {
    return this._sendAndWait({
      id: this._nextId++,
      type: ProxyMessageKind.RequestStatus,
      args: [fields, principal],
    }) as Promise<RequestStatusResponse>;
  }

  public call(
    canisterId: CanisterId | string,
    fields: CallFields,
    principal?: Principal,
  ): Promise<SubmitResponse> {
    return this._sendAndWait({
      id: this._nextId++,
      type: ProxyMessageKind.Call,
      args: [canisterId.toString(), fields, principal],
    }) as Promise<SubmitResponse>;
  }

  public createCanister(principal?: Principal): Promise<SubmitResponse> {
    throw new Error('unimplemented. This will be removed when we upgrade the spec to 0.8');
  }

  public status(): Promise<JsonObject> {
    return this._sendAndWait({
      id: this._nextId++,
      type: ProxyMessageKind.Status,
    }) as Promise<JsonObject>;
  }

  public install(
    canisterId: CanisterId | string,
    fields: {
      module: BinaryBlob;
      arg?: BinaryBlob;
    },
    principal?: Principal,
  ): Promise<SubmitResponse> {
    throw new Error('unimplemented. This will be removed when we upgrade the spec to 0.8');
  }

  public query(
    canisterId: CanisterId | string,
    fields: QueryFields,
    principal?: Principal,
  ): Promise<QueryResponse> {
    return this._sendAndWait({
      id: this._nextId++,
      type: ProxyMessageKind.Query,
      args: [canisterId.toString(), fields, principal],
    }) as Promise<QueryResponse>;
  }

  public makeActorFactory(actorInterfaceFactory: IDL.InterfaceFactory): ActorFactory {
    return actor.makeActorFactory(actorInterfaceFactory);
  }

  private async _sendAndWait(msg: ProxyMessage): Promise<unknown> {
    return new Promise((resolve, reject) => {
      this._pendingCalls.set(msg.id, [resolve, reject]);

      this._backend(msg);
    });
  }
}
