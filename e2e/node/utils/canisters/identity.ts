import { blobFromUint8Array } from '@dfinity/agent';
import {httpAgent, canisterIdFactory} from '../agent';
import * as path from 'path';
import { readFileSync } from 'fs';
import { default as idl, Identity } from "./identity/main.did";

const wasm = readFileSync(path.join(__dirname, 'identity/main.wasm'));
const factory = httpAgent.makeActorFactory(idl);

// TODO(hansl): Add a type to create an Actor interface from a IDL.Service definition.
export async function identityFactory(): Promise<Identity> {
  let actor = await factory({ httpAgent }) as Identity;
  let cid = await actor.__createCanister();
  actor.__setCanisterId(cid);

  await actor.__install({
    module: blobFromUint8Array(wasm),
  }, {
    maxAttempts: 600,
    throttleDurationInMSecs: 100,
  });

  return actor;
}
