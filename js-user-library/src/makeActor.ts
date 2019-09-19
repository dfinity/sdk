import {
  ApiClient,
  ReadRequestStatusResponseStatus as ResponseStatus,
} from "./apiClient";

import { zipWith } from "./array";
import { ActorInterface, Fn } from "./IDL";

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
  apiClient: ApiClient,
) => {
  const entries = Object.entries(actorInterface.__fields);
  return Object.fromEntries(entries.map((entry) => {
    const [methodName, fn] = entry as [string, Fn];
    return [methodName, async (...args: Array<any>) => {
      // TODO: throw if desc.argTypes.length !== args.length
      const encoded = zipWith(fn.argTypes, args, (x, y) => x.encode(y));
      const arg = new Blob(encoded, { type: "application/cbor" }); // TODO: is this the right thing to do?

      const {
        requestId,
        // response, // FIXME: check response is OK before continuing
      } = await apiClient.call({ methodName, arg });

      const maxRetries = 3;

      // NOTE: we may need to use something like `setInterval` here
      for (let i = 0; i < maxRetries; i++) {
        const response = await apiClient.requestStatus({ requestId });
        // TODO: handle decoding failure
        const responseBody = await response.arrayBuffer();
        const decoded = zipWith(fn.retTypes, responseBody, (x, y) => {
          return x.decode(y);
        });
        const replied = ResponseStatus[ResponseStatus.replied];

        if (decoded.status === replied) {
          return decoded.reply;
        }
        if (i + 1 === maxRetries) {
          return response; // TODO: throw
        }
      }
    }];
  }));
};
