import {
  ApiClient,
  ReadRequestStatusResponseStatus as ResponseStatus,
} from "./apiClient";

import { ActorInterface } from "./IDL";

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
  return Object.fromEntries(entries.map(([methodName, desc]) => {
    return [methodName, async (...args/* FIXME */: any[]) => {
      // TODO: convert `args` to `arg` using `desc`
      const arg = new Blob([], { type: "application/cbor" });
      const {
        requestId,
        // response,
      } = await apiClient.call({ methodName, arg });

      const maxRetries = 3;

      // NOTE: we may need to use something like `setInterval` here
      for (let i = 0; i < maxRetries; i++) {
        const response = await apiClient.requestStatus({ requestId });
        // FIXME: the body should be a CBOR value
        // TODO: handle decoding failure
        const responseBody = await response.json();
        const replied = ResponseStatus[ResponseStatus.replied];
        if (responseBody.status === replied) {
          return responseBody.reply;
        }
        if (i + 1 === maxRetries) {
          return response; // TODO: throw?
        }
      }
    }];
  }));
};
