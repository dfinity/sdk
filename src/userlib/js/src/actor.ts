import { Buffer } from 'buffer/';
import * as blob from './blob';
import { HttpAgent } from './http_agent';
import * as _IDL from './idl';
import * as requestId from './request_id';

import { RequestStatusResponse, RequestStatusResponseStatus } from './request_status_response';

function retry<T>(fn: () => Promise<T>, maxAttempts: number): Promise<T> {
  return fn().catch(err => {
    if (maxAttempts > 0) {
      return new Promise(resolve => setTimeout(resolve, 500)).then(() =>
        retry(fn, maxAttempts - 1),
      );
    } else {
      throw err;
    }
  });
}

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
export const makeActor = (
  makeActorInterface: ({ IDL }: { IDL: typeof _IDL }) => _IDL.ActorInterface,
) => (
  httpAgent: HttpAgent,
  // The return type here represents a record whose values may be any function.
  // By using "rest parameter syntax" we can type variadic functions in a
  // homogenous record, as well as process the arguments as an Array.
): Record<string, (...args: any[]) => any> => {
  const actorInterface = makeActorInterface({ IDL: _IDL });
  const entries: Array<[string, _IDL.FuncClass]> = Object.entries(actorInterface._fields).filter(
    ([_, x]) => x instanceof _IDL.FuncClass,
  ) as any;

  const result: Record<string, (...args: any[]) => any> = {};
  for (const [methodName, func] of entries) {
    result[methodName] = async (...args: any[]) => {
      // IDL.js encoding produces a feross/safe-buffer `Buffer`. We need to
      // convert to a ferross/buffer `Buffer` so that our `instanceof` checks
      // succeed. TODO: reconcile these `Buffer` types.
      const safeBuffer = _IDL.encode(func.argTypes, args);
      const hex = safeBuffer.toString('hex');
      const arg = blob.fromHex(hex);

      const { requestId: requestIdent, response: callResponse } = await httpAgent.call({
        methodName,
        arg,
      });

      if (!callResponse.ok) {
        throw new Error(
          [
            'Request failed:',
            `  Request ID: ${requestId.toHex(requestIdent)}`,
            `  HTTP status code: ${callResponse.status}`,
            `  HTTP status text: ${callResponse.statusText}`,
          ].join('\n'),
        );
      }

      const maxAttempts = 10;

      const reply = await retry(async () => {
        const response: RequestStatusResponse = await httpAgent.requestStatus({
          requestId: requestIdent,
        });

        switch (response.status) {
          case RequestStatusResponseStatus.Replied: {
            const returnValue = _IDL.decode(func.retTypes, Buffer.from(response.reply.arg));

            // IDL functions can have multiple return values, so decoding always
            // produces an array. Ensure that functions with single return
            // values behave as expected.
            if (returnValue instanceof Array && returnValue.length === 1) {
              return returnValue[0];
            } else {
              return returnValue;
            }
          }
          default: {
            throw new Error(
              [
                `Failed to retrieve a reply for request after ${maxAttempts} attempts:`,
                `  Request ID: ${requestId.toHex(requestIdent)}`,
                `  Request status: ${response.status}`,
                `  Method: ${methodName}([${args.join(', ')}]`,
              ].join('\n'),
            );
          }
        }
      }, maxAttempts);

      return reply;
    };
  }
  return result;
};
