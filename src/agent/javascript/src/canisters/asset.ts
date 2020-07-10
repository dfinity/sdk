/**
 * This file is generated from the candid for asset management.
 */
/* tslint:disable */
// @ts-ignore
export default ({ IDL }) => {
  return IDL.Service({
    retrieve: IDL.Func([IDL.Text], [IDL.Vec(IDL.Nat8)], ['query']),
    store: IDL.Func([IDL.Text, IDL.Vec(IDL.Nat8)], [], []),
  });
};
