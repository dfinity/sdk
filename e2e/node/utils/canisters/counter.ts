import { Actor, IDL, blobFromUint8Array } from '@dfinity/agent';
import * as path from 'path';
import { readFileSync } from 'fs';
import { httpAgent } from '../agent';

const wasm = readFileSync(path.join(__dirname, 'counter.wasm'));

type CounterActor = Actor & {
  read(): Promise<number>;
  inc_read(): Promise<number>;
  inc(): Promise<void>;
  write(n: number): Promise<void>;
};

const factory: IDL.InterfaceFactory = ({ IDL }) =>
  IDL.Service({
    read: IDL.Func([], [IDL.Nat], ['query']),
    inc_read: IDL.Func([], [IDL.Nat], []),
    inc: IDL.Func([], [], []),
    write: IDL.Func([IDL.Nat], [], []),
  });

// TODO(hansl): Add a type to create an Actor interface from a IDL.Service definition.
export async function counterFactory(): Promise<CounterActor> {
  return ((await Actor.createAndInstallCanister(
    factory,
    {
      module: blobFromUint8Array(wasm),
    },
    {
      agent: httpAgent,
    },
  )) as unknown) as CounterActor;
}
