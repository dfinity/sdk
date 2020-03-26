import { Actor, CanisterId, blobFromUint8Array } from '@internet-computer/userlib';
import httpAgent from '../agent';
import * as path from 'path';
import { readFileSync } from 'fs';

const wasm = readFileSync(path.join(__dirname, 'counter.wasm'));

// The canisterId will be reused.
const counterCanisterIdHex = (+new Date() % 0xFFFFFF).toString(16)
                           + (Math.floor(Math.random() * 256)).toString(16);
const counterCanisterId = CanisterId.fromHex(counterCanisterIdHex);

let actor: Promise<CounterActor>;

type CounterActor = Actor & {
  read(): Promise<number>,
  inc_read(): Promise<number>,
  write(n: number): Promise<void>,
};

const factory = httpAgent.makeActorFactory(({ IDL }) => new IDL.ActorInterface({
  'read': IDL.Func([], [IDL.Nat], ['query']),
  'inc_read': IDL.Func([], [IDL.Nat], []),
  'inc': IDL.Func([], [], []),
  'write': IDL.Func([IDL.Nat], [], []),
}));

// TODO(hansl): Add a type to create an Actor interface from a IDL.ActorInterface definition.
export default function(): Promise<CounterActor> {
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
