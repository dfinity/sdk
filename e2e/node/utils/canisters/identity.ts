import { blobFromUint8Array } from '@dfinity/agent';
import {httpAgent, canisterIdFactory} from '../agent';
import * as path from 'path';
import { readFileSync } from 'fs';
import { default as idl, Identity } from "./identity/main.did";

const wasm = readFileSync(path.join(__dirname, 'identity/main.wasm'));

// The canisterId will be reused.
const identityCanisterId = canisterIdFactory();
let actor: Promise<Identity>;

const factory = httpAgent.makeActorFactory(idl);

// TODO(hansl): Add a type to create an Actor interface from a IDL.Service definition.
export function identityFactory(): Promise<Identity> {
  if (!actor) {
    actor = Promise.resolve(factory({
      canisterId: identityCanisterId,
      httpAgent,
    }) as Identity).then(actor => {
      return actor.__install({
        module: blobFromUint8Array(wasm),
      }, {
        maxAttempts: 600,
        throttleDurationInMSecs: 100,
      })
        .then(() => actor);
    });
  }

  return actor;
}
