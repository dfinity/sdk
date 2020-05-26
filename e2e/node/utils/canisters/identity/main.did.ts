import { Actor, IDL } from '@dfinity/agent';

export type Identity = Actor & {
  hashFromCall(): IDL.NatClass;
  hashFromQuery(): IDL.NatClass;
};

export default ({ IDL }: any) => {
 return IDL.Service({'hashFromCall': IDL.Func([], [IDL.Nat], []),
  'hashFromQuery': IDL.Func([], [IDL.Nat], ['query'])});
};
