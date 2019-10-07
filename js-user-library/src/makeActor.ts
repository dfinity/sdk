import {
  HttpAgent,
  RequestStatusResponse,
  RequestStatusResponseStatus,
} from "./httpAgent";

import retry from "async-retry";
import { Buffer } from "buffer/";
import { zipWith } from "./array";
import { toHex } from "./buffer";
import _IDL from "./IDL";

// Allows for one client for the lifetime of the actor:
//
// ```
// const actor = makeActor(actorInterface)(client);
// const reply = actor.greet();
// ```
//
// or using a different client for the same actor if necessary:
//
// ```
// const actor = makeActor(actorInterface);
// const reply1 = actor(client1).greet();
// const reply2 = actor(client2).greet();
// ```
export const makeActor = (
  makeActorInterface: ({ IDL }: { IDL: typeof _IDL }) => _IDL.ActorInterface,
) => (
  httpAgent: HttpAgent,
): Record<string, (...args: Array<any>) => any> => {
  const actorInterface = makeActorInterface({ IDL: _IDL });
  const entries = Object.entries(actorInterface.__fields);
  return Object.fromEntries(entries.map((entry) => {
    const [methodName, func] = entry as [string, _IDL.Func];
    return [methodName, async (...args: Array<any>) => {
      // TODO: throw if func.argTypes.length !== args.length
      // FIXME: Old code does something like:
      // const buffer = zipWith(func.argTypes, args, (x, y) => x.encode(y));

      const {
        requestId,
        response: callResponse,
      } = await httpAgent.call({ methodName, arg: Buffer.from([]) });

      if (!callResponse.ok) {
        throw new Error([
          `Request failed:`,
          `  Request ID: ${toHex(requestId)}`,
          `  HTTP status code: ${callResponse.status}`,
          `  HTTP status text: ${callResponse.statusText}`,
        ].join("\n"));
      }

      const maxAttempts = 3;

      const reply = await retry(async (bail, attempts) => {
        const response: RequestStatusResponse = await httpAgent.requestStatus({
          requestId,
        });

        switch (response.status) {
          case RequestStatusResponseStatus.Replied: {
            return response.reply;

            // FIXME: Old code does something like the following:
            // tslint:disable-next-line: max-line-length
            // https://github.com/dfinity-lab/dev/blob/9030c90efe5b3de33670d4f4f0331482d51c5858/experimental/js-dfinity-client/src/IDL.js#L753
            // TODO: throw if
            //   func.retTypes.length !== response.reply.arg.length
            // TODO: handle IDL decoding failures
            //   return zipWith(
            //     func.retTypes,
            //     response.reply.arg,
            //     (x, y) => {
            //       return x.decode(y);
            //     },
            //   );
          }
          default: {
            throw new Error([
              `Failed to retrieve a reply for request after ${attempts} attempts:`,
              `  Request ID: ${toHex(requestId)}`,
              `  Request status: ${response.status}`,
            ].join("\n"));
          }
        }
      }, {
        retries: maxAttempts - 1,
      });

      return reply.arg;
    }];
  }));
};
