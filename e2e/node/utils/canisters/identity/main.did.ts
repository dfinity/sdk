import { Actor, IDL } from '@dfinity/agent';

export type Identity = Actor & {
  hashFromCall(): Promise<number>;
  hashFromQuery(): Promise<number>;
};

export const factory: IDL.InterfaceFactory = ({ IDL }) => {
  return IDL.Service({
    hashFromCall: IDL.Func([], [IDL.Nat], []),
    hashFromQuery: IDL.Func([], [IDL.Nat], ['query']),
  });
};
