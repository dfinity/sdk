export const idlFactory = ({ IDL }) => {
  const Orders = IDL.Service({
    'addItemToFolder' : IDL.Func(
        [
          IDL.Tuple(IDL.Principal, IDL.Nat),
          IDL.Tuple(IDL.Principal, IDL.Nat),
          IDL.Bool,
          IDL.Variant({ 'end' : IDL.Null, 'beginning' : IDL.Null }),
        ],
        [],
        [],
      ),
    'getOwners' : IDL.Func([], [IDL.Vec(IDL.Principal)], ['query']),
    'init' : IDL.Func([IDL.Vec(IDL.Principal)], [], []),
    'insertIntoAllTimeStream' : IDL.Func(
        [IDL.Tuple(IDL.Principal, IDL.Nat)],
        [],
        [],
      ),
    'removeFromAllTimeStream' : IDL.Func(
        [IDL.Tuple(IDL.Principal, IDL.Nat)],
        [],
        [],
      ),
    'removeItemLinks' : IDL.Func([IDL.Tuple(IDL.Principal, IDL.Nat)], [], []),
    'setOwners' : IDL.Func([IDL.Vec(IDL.Principal)], [], []),
    'vote' : IDL.Func(
        [IDL.Principal, IDL.Nat, IDL.Principal, IDL.Nat, IDL.Int, IDL.Bool],
        [],
        [],
      ),
  });
  return Orders;
};
export const init = ({ IDL }) => { return []; };
