import { Buffer } from "buffer/";
import { BinaryBlob } from "./blob";
import * as blob from "./blob";
import { Hex } from "./hex";
import { HttpAgent } from "./httpAgent";
import _IDL from "./IDL";
import * as requestId from "./requestId";

import {
  RequestStatusResponse,
  RequestStatusResponseStatus,
} from "./requestStatusResponse";

import retry from "async-retry";

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
): Record<string, (...args: Array<any>) => any> => {
  const actorInterface = makeActorInterface({ IDL: _IDL });
  const entries = Object.entries(actorInterface.__fields);
  return Object.fromEntries(entries.map((entry) => {
    const [methodName, func] = entry as [string, _IDL.Func];
    return [methodName, async (...args: Array<any>) => {
      // TODO
      // * Throw if func.argTypes.length !== args.length
      // * Encode request arguments with the corresponding type

      // IDL.js encoding produces a feross/safe-buffer `Buffer`. We need to
      // convert to a ferross/buffer `Buffer` so that our `instanceof` checks
      // succeed. TODO: reconcile these `Buffer` types.
      const safeBuffer = func.argTypes[0].encode(args[0]);
      const hex = safeBuffer.toString("hex") as Hex;
      const arg = blob.fromHex(hex);

      const {
        requestId: requestIdent,
        response: callResponse,
      } = await httpAgent.call({
        methodName,
        arg,
      });

      if (!callResponse.ok) {
        throw new Error([
          `Request failed:`,
          `  Request ID: ${requestId.toHex(requestIdent)}`,
          `  HTTP status code: ${callResponse.status}`,
          `  HTTP status text: ${callResponse.statusText}`,
        ].join("\n"));
      }

      const maxAttempts = 3;

      const reply = await retry(async (bail, attempts) => {
        const response: RequestStatusResponse = await httpAgent.requestStatus({
          requestId: requestIdent,
        });

        switch (response.status) {
          case RequestStatusResponseStatus.Replied: {
            // TODO
            // * Throw if func.retTypes.length !== response.reply.arg.length
            // * Decode response arguments with the corresponding type
            return func.retTypes[0].decode(Buffer.from(response.reply.arg));
          }
          default: {
            throw new Error([
              `Failed to retrieve a reply for request after ${attempts} attempts:`,
              `  Request ID: ${requestId.toHex(requestIdent)}`,
              `  Request status: ${response.status}`,
            ].join("\n"));
          }
        }
      }, {
        retries: maxAttempts - 1,
      });

      return reply;
    }];
  }));
};
