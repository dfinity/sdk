export const idlFactory = ({ IDL }) => {
  const Fraction = IDL.Int;
  const Subaccount = IDL.Vec(IDL.Nat8);
  const Payments = IDL.Service({
    'getBuyerAffiliateShare' : IDL.Func([], [Fraction], ['query']),
    'getOurDebt' : IDL.Func([IDL.Principal], [IDL.Nat], ['query']),
    'getOwners' : IDL.Func([], [IDL.Vec(IDL.Principal)], ['query']),
    'getSalesOwnersShare' : IDL.Func([], [Fraction], ['query']),
    'getSellerAffiliateShare' : IDL.Func([], [Fraction], ['query']),
    'getUploadOwnersShare' : IDL.Func([], [Fraction], ['query']),
    'getUpvotesOwnersShare' : IDL.Func([], [Fraction], ['query']),
    'init' : IDL.Func([IDL.Vec(IDL.Principal)], [], []),
    'payout' : IDL.Func([IDL.Opt(Subaccount)], [], ['oneway']),
    'setBuyerAffiliateShare' : IDL.Func([Fraction], [], ['oneway']),
    'setOwners' : IDL.Func([IDL.Vec(IDL.Principal)], [], []),
    'setSalesOwnersShare' : IDL.Func([Fraction], [], ['oneway']),
    'setSellerAffiliateShare' : IDL.Func([Fraction], [], ['oneway']),
    'setUploadOwnersShare' : IDL.Func([Fraction], [], ['oneway']),
    'setUpvotesOwnersShare' : IDL.Func([Fraction], [], ['oneway']),
  });
  return Payments;
};
export const init = ({ IDL }) => { return []; };
