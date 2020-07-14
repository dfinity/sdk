import { Actor, IDL, blobFromUint8Array } from '@dfinity/agent';
import * as path from 'path';
import { readFileSync } from 'fs';
import { httpAgent } from '../agent';
import { default as factory, Identity } from './identity/main.did';

const wasm = readFileSync(path.join(__dirname, 'identity/main.wasm'));

// TODO(hansl): Add a type to create an Actor interface from a IDL.Service definition.
export async function identityFactory(): Promise<Identity> {
  return ((await Actor.createAndInstallCanister(
    factory as IDL.InterfaceFactory,
    {
      module: blobFromUint8Array(wasm),
    },
    {
      agent: httpAgent,
    },
  )) as unknown) as Identity;
}
