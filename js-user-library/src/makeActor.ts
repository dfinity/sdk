import {
  HttpAgent,
  RequestStatusResponse,
  RequestStatusResponseStatus,
} from "./httpAgent";

import { zipWith } from "./array";
import { ActorInterface, Func } from "./IDL";

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
  actorInterface: ActorInterface,
) => (
  httpAgent: HttpAgent,
) => {
  const entries = Object.entries(actorInterface.__fields);
  return Object.fromEntries(entries.map((entry) => {
    const [methodName, func] = entry as [string, Func];
    return [methodName, async (...args: Array<any>) => {
      // TODO: throw if func.argTypes.length !== args.length
      // FIXME: Old code does something like:
      // const buffer = zipWith(func.argTypes, args, (x, y) => x.encode(y));

      const {
        requestId,
        // response, // FIXME: check response is OK before continuing
      } = await httpAgent.call({ methodName, arg: [] });

      const maxRetries = 3;

      // NOTE: we may need to use something like `setInterval` here
      for (let i = 0; i < maxRetries; i++) {
        const response: RequestStatusResponse = await httpAgent.requestStatus({
          requestId,
        });

        switch (response.status) {
          case RequestStatusResponseStatus.Replied: {
            return response.reply;

            // FIXME: Old code does something like the following:
            // tslint:disable-next-line: max-line-length
            // https://github.com/dfinity-lab/dev/blob/9030c90efe5b3de33670d4f4f0331482d51c5858/experimental/js-dfinity-client/src/IDL.js#L753
            // TODO: throw if func.retTypes.length !== response.reply.arg.length
            // TODO: handle IDL decoding failures
            // return zipWith(func.retTypes, response.reply.arg, (x, y) => {
            //   return x.decode(y);
            // });
          }
          default: {
            if (i + 1 === maxRetries) {
              return response; // TODO: throw
            }
          }
        }
        /*
        // TODO: handle decoding failure
        const responseBody = await response.arrayBuffer();
        // TODO: throw if fn.retTypes.length !== args.length
        const decoded = zipWith(fn.retTypes, responseBody, (x, y) => {
          return x.decode(y);
        });
        */
      }
    }];
  }));
};
