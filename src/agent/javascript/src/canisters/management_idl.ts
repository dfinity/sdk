/**
 * This file is generated from the candid for asset management.
 */
/* tslint:disable */
// @ts-ignore
export default ({ IDL }) => {
  const canister_id = IDL.Principal;
  const wasm_module = IDL.Vec(IDL.Nat8);
  return IDL.Service({
    create_canister: IDL.Func([], [IDL.Record({ canister_id: canister_id })], []),
    install_code: IDL.Func(
      [
        IDL.Record({
          mode: IDL.Variant({ install: IDL.Null, reinstall: IDL.Null, upgrade: IDL.Null }),
          canister_id: canister_id,
          wasm_module: wasm_module,
          arg: IDL.Vec(IDL.Nat8),
          compute_allocation: IDL.Opt(IDL.Nat),
          memory_allocation: IDL.Opt(IDL.Nat),
        }),
      ],
      [],
      [],
    ),
    set_controller: IDL.Func(
      [IDL.Record({ canister_id: canister_id, new_controller: IDL.Principal })],
      [],
      [],
    ),
  });
};
