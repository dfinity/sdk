export const idlFactory = ({ IDL }) => {
  return IDL.Service({
    'erc1155_balance_of' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Text, IDL.Nat64],
        [IDL.Nat],
        [],
      ),
    'erc721_owner_of' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Nat64],
        [IDL.Text],
        [],
      ),
    'verify_ecdsa' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Text],
        [IDL.Bool],
        ['query'],
      ),
  });
};
export const init = ({ IDL }) => { return []; };
