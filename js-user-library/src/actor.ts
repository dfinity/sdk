import { Buffer } from 'buffer/';
import { CanisterId } from './canisterId';
import { HttpAgent } from './http_agent';
import { QueryResponseStatus, RequestStatusResponseStatus } from './http_agent_types';
import * as IDL from './idl';
import { FuncClass } from './idl';
import { RequestId, toHex as requestIdToHex } from './request_id';
import { BinaryBlob } from './types';

declare const window: { icHttpAgent?: HttpAgent };
declare const global: { icHttpAgent?: HttpAgent };
declare const self: { icHttpAgent?: HttpAgent };

/**
 * An actor interface. An actor is an object containing only functions that will
 * return a promise. These functions are derived from the IDL definition.
 */
export interface Actor extends Record<string, (...args: unknown[]) => Promise<unknown>> {}

export interface ActorConfig {
  canisterId: CanisterId;
  httpAgent?: HttpAgent;
  maxAttempts?: number;
  throttleDurationInMSecs?: number;
}

const REQUEST_STATUS_RETRY_WAIT_DURATION_IN_MSECS = 500;
const DEFAULT_ACTOR_CONFIG: Partial<ActorConfig> = {
  maxAttempts: 10,
  throttleDurationInMSecs: REQUEST_STATUS_RETRY_WAIT_DURATION_IN_MSECS,
  httpAgent:
    typeof window === 'undefined'
      ? typeof global === 'undefined'
        ? typeof self === 'undefined'
          ? undefined
          : self.icHttpAgent
        : global.icHttpAgent
      : window.icHttpAgent,
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

  async function callDebriefing(
    httpAgent: HttpAgent,
    requestId: RequestId,
    func: FuncClass,
    attempts: number,
    maxAttempts: number,
    throttle: number,
  ): Promise<unknown> {
    const status = await httpAgent.requestStatus({ requestId });

    switch (status.status) {
      case RequestStatusResponseStatus.Replied: {
        const returnValue = IDL.decode(func.retTypes, Buffer.from(status.reply.arg));

        // IDL functions can have multiple return values, so decoding always
        // produces an array. Ensure that functions with single return
        // values behave as expected.
        if (returnValue.length === 1) {
          return returnValue[0];
        } else {
          return returnValue;
        }
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
          callDebriefing(httpAgent, requestId, func, attempts, maxAttempts, throttle),
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
    const actor: Actor = {};
    const { canisterId, maxAttempts, throttleDurationInMSecs, httpAgent } = {
      ...DEFAULT_ACTOR_CONFIG,
      ...config,
    } as Required<ActorConfig>;

    if (!httpAgent) {
      throw new Error('Cannot make call. httpAgent is undefined.');
    }

    for (const [methodName, func] of Object.entries(actorInterface._fields)) {
      actor[methodName] = async (...args: any[]) => {
        const arg = IDL.encode(func.argTypes, args) as BinaryBlob;

        if (func.annotations.includes('query')) {
          const result = await httpAgent.query(canisterId, { methodName, arg });

          switch (result.status) {
            case QueryResponseStatus.Rejected:
              throw new Error(
                `Query failed:\n` +
                  `  Status: ${result.status}\n` +
                  `  Message: ${result.reject_message}\n`,
              );

            case QueryResponseStatus.Replied:
              return IDL.decode(func.retTypes, result.reply.arg);
          }
        } else {
          const { requestId, response } = await httpAgent.call(canisterId, { methodName, arg });

          if (!response.ok) {
            throw new Error(
              [
                'Call failed:',
                `  Request ID: ${requestIdToHex(requestId)}`,
                `  HTTP status code: ${response.status}`,
                `  HTTP status text: ${response.statusText}`,
              ].join('\n'),
            );
          }

          return callDebriefing(
            httpAgent,
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
