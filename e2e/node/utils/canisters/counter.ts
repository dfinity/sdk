import { Actor, blobFromUint8Array } from '@dfinity/agent';
import { httpAgent, canisterIdFactory } from '../agent';
import * as path from 'path';
import { readFileSync } from 'fs';

const wasm = readFileSync(path.join(__dirname, 'counter.wasm'));

// The canisterId will be reused.
const counterCanisterId = canisterIdFactory();
let actor: Promise<CounterActor>;

type CounterActor = Actor & {
  read(): Promise<number>,
  inc_read(): Promise<number>,
  write(n: number): Promise<void>,
};

const factory = httpAgent.makeActorFactory(({ IDL }) => IDL.Service({
  'read': IDL.Func([], [IDL.Nat], ['query']),
  'inc_read': IDL.Func([], [IDL.Nat], []),
  'inc': IDL.Func([], [], []),
  'write': IDL.Func([IDL.Nat], [], []),
}));

// TODO(hansl): Add a type to create an Actor interface from a IDL.Service definition.
export function counterFactory(): Promise<CounterActor> {
  if (!actor) {
    actor = Promise.resolve(factory({
      canisterId: counterCanisterId,
      httpAgent,
    }) as CounterActor).then(actor => {
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
