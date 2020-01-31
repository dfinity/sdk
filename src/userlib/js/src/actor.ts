import { Buffer } from 'buffer/';
import { CanisterId } from './canisterId';
import { HttpAgent } from './http_agent';
import { QueryResponseStatus, RequestStatusResponseStatus } from './http_agent_types';
import * as IDL from './idl';
import { FuncClass } from './idl';
import { RequestId, toHex as requestIdToHex } from './request_id';
import { BinaryBlob } from './types';

/**
 * An actor interface. An actor is an object containing only functions that will
 * return a promise. These functions are derived from the IDL definition.
 */
export type Actor = Record<string, (...args: unknown[]) => Promise<unknown>> & {
  __canisterId(): string;
  __getAsset(path: string): Promise<Uint8Array>;
  __install(
    fields: {
      module: BinaryBlob;
      arg?: BinaryBlob;
    },
    options?: {
      maxAttempts?: number;
      throttleDurationInMSecs?: number;
    },
  ): Promise<void>;
};

export interface ActorConfig {
  canisterId: string | CanisterId;
  httpAgent?: HttpAgent;
  maxAttempts?: number;
  throttleDurationInMSecs?: number;
}

declare const window: { icHttpAgent?: HttpAgent };
declare const global: { icHttpAgent?: HttpAgent };
declare const self: { icHttpAgent?: HttpAgent };

function getDefaultHttpAgent() {
  return typeof window === 'undefined'
    ? typeof global === 'undefined'
      ? typeof self === 'undefined'
        ? undefined
        : self.icHttpAgent
      : global.icHttpAgent
    : window.icHttpAgent;
}

// IDL functions can have multiple return values, so decoding always
// produces an array. Ensure that functions with single or zero return
// values behave as expected.
function decodeReturnValue(types: IDL.Type[], msg: BinaryBlob) {
  const returnValues = IDL.decode(types, Buffer.from(msg));
  if (returnValues.length === 0) {
    return undefined;
  } else if (returnValues.length === 1) {
    return returnValues[0];
  } else {
    return returnValues;
  }
}

const REQUEST_STATUS_RETRY_WAIT_DURATION_IN_MSECS = 500;
const DEFAULT_ACTOR_CONFIG: Partial<ActorConfig> = {
  maxAttempts: 30,
  throttleDurationInMSecs: REQUEST_STATUS_RETRY_WAIT_DURATION_IN_MSECS,
};

export type ActorConstructor = (config: ActorConfig) => Actor;

// Make an actor from an actor interface.
//
// Allows for one HTTP agent for the lifetime of the actor:
//
// ```
// const actor = makeActor(actorInterface)(httpAgent);
// const reply = await actor.greet();
// ```
//
// or using a different HTTP agent for the same actor if necessary:
//
// ```
// const actor = makeActor(actorInterface);
// const reply1 = await actor(httpAgent1).greet();
// const reply2 = await actor(httpAgent2).greet();
// ```
export function makeActorFactory(
  actorInterfaceFactory: (_: { IDL: typeof IDL }) => IDL.ActorInterface,
): ActorConstructor {
  const actorInterface = actorInterfaceFactory({ IDL });

  async function requestStatusAndLoop(
    httpAgent: HttpAgent,
    requestId: RequestId,
    func: FuncClass | null,
    attempts: number,
    maxAttempts: number,
    throttle: number,
  ): Promise<unknown> {
    const status = await httpAgent.requestStatus({ requestId });

    switch (status.status) {
      case RequestStatusResponseStatus.Replied: {
        return func ? decodeReturnValue(func.retTypes, status.reply.arg) : null;
      }

      case RequestStatusResponseStatus.Unknown:
      case RequestStatusResponseStatus.Pending:
        if (--attempts === 0) {
          throw new Error(
            `Failed to retrieve a reply for request after ${maxAttempts} attempts:\n` +
              `  Request ID: ${requestIdToHex(requestId)}\n` +
              `  Request status: ${status.status}\n`,
          );
        }

        // Wait a little, then retry.
        return new Promise(resolve => setTimeout(resolve, throttle)).then(() =>
          requestStatusAndLoop(httpAgent, requestId, func, attempts, maxAttempts, throttle),
        );

      case RequestStatusResponseStatus.Rejected:
        throw new Error(
          `Call was rejected:\n` +
            `  Request ID: ${requestIdToHex(requestId)}\n` +
            `  Reject code: ${status.reject_code}\n` +
            `  Reject text: ${status.reject_message}\n`,
        );
    }
  }

  return (config: ActorConfig) => {
    const { canisterId, maxAttempts, throttleDurationInMSecs, httpAgent } = {
      ...DEFAULT_ACTOR_CONFIG,
      ...config,
    } as Required<ActorConfig>;
    const cid = typeof canisterId === 'string' ? CanisterId.fromText(canisterId) : canisterId;
    const actor: Actor = {
      __canisterId() {
        return cid.toHex();
      },
      async __getAsset(path: string) {
        const agent = httpAgent || getDefaultHttpAgent();
        if (!agent) {
          throw new Error('Cannot make call. httpAgent is undefined.');
        }

        return agent.retrieveAsset(canisterId, path);
      },
      async __install(
        fields: {
          module: BinaryBlob;
          arg?: BinaryBlob;
        },
        options: {
          maxAttempts?: number;
          throttleDurationInMSecs?: number;
        } = {},
      ) {
        const agent = httpAgent || getDefaultHttpAgent();
        if (!agent) {
          throw new Error('Cannot make call. httpAgent is undefined.');
        }

        // Resolve the options that can be used globally or locally.
        const effectiveMaxAttempts = options.maxAttempts?.valueOf() || 0;
        const effectiveThrottle = options.throttleDurationInMSecs?.valueOf() || 0;

        const { requestId, response } = await agent.install(canisterId, fields);
        if (!response.ok) {
          throw new Error(
            [
              'Install failed:',
              `  Canister ID: ${cid.toHex()}`,
              `  Request ID: ${requestIdToHex(requestId)}`,
              `  HTTP status code: ${response.status}`,
              `  HTTP status text: ${response.statusText}`,
            ].join('\n'),
          );
        }

        return requestStatusAndLoop(
          agent,
          requestId,
          null,
          effectiveMaxAttempts,
          effectiveMaxAttempts,
          effectiveThrottle,
        );
      },
    } as Actor;

    for (const [methodName, func] of Object.entries(actorInterface._fields)) {
      actor[methodName] = async (...args: any[]) => {
        const agent = httpAgent || getDefaultHttpAgent();
        if (!agent) {
          throw new Error('Cannot make call. httpAgent is undefined.');
        }

        const arg = IDL.encode(func.argTypes, args) as BinaryBlob;
        if (func.annotations.includes('query')) {
          const result = await agent.query(cid, { methodName, arg });

          switch (result.status) {
            case QueryResponseStatus.Rejected:
              throw new Error(
                `Query failed:\n` +
                  `  Status: ${result.status}\n` +
                  `  Message: ${result.reject_message}\n`,
              );

            case QueryResponseStatus.Replied:
              return decodeReturnValue(func.retTypes, result.reply.arg);
          }
        } else {
          const { requestId, response } = await agent.call(cid, { methodName, arg });

          if (!response.ok) {
            throw new Error(
              [
                'Call failed:',
                `  Method: ${methodName}(${args})`,
                `  Canister ID: ${cid.toHex()}`,
                `  Request ID: ${requestIdToHex(requestId)}`,
                `  HTTP status code: ${response.status}`,
                `  HTTP status text: ${response.statusText}`,
              ].join('\n'),
            );
          }

          return requestStatusAndLoop(
            agent,
            requestId,
            func,
            maxAttempts,
            maxAttempts,
            throttleDurationInMSecs,
          );
        }
      };
    }

    return actor;
  };
}
