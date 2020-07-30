import { Actor, IDL } from '@dfinity/agent';

export type Identity = Actor & {
  hashFromCall(): Promise<number>;
  hashFromQuery(): Promise<number>;
};

const factory: IDL.InterfaceFactory = ({ IDL }: any) => {
  return IDL.Service({
    hashFromCall: IDL.Func([], [IDL.Nat], []),
    hashFromQuery: IDL.Func([], [IDL.Nat], ['query']),
  });
};
export default factory;
