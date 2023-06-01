export const idlFactory = ({ IDL }) => {
  const Config = IDL.Record({
    'backend_canister_id' : IDL.Opt(IDL.Principal),
    'remove_cycles_add' : IDL.Bool,
    'profiling' : IDL.Bool,
    'limit_stable_memory_page' : IDL.Opt(IDL.Nat32),
  });
  return IDL.Service({
    'transform' : IDL.Func(
        [IDL.Vec(IDL.Nat8), Config],
        [IDL.Vec(IDL.Nat8)],
        [],
      ),
  });
};
export const init = ({ IDL }) => { return []; };
