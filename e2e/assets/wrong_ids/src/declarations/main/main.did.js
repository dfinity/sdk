export const idlFactory = ({ IDL }) => {
  const ItemDetails = IDL.Variant({
    'link' : IDL.Text,
    'post' : IDL.Null,
    'message' : IDL.Null,
    'folder' : IDL.Null,
  });
  const ItemDataWithoutOwner = IDL.Record({
    'title' : IDL.Text,
    'locale' : IDL.Text,
    'description' : IDL.Text,
    'details' : ItemDetails,
    'price' : IDL.Float64,
  });
  const ItemTransferWithoutOwner = IDL.Record({
    'data' : ItemDataWithoutOwner,
    'communal' : IDL.Bool,
  });
  const User = IDL.Record({
    'title' : IDL.Text,
    'link' : IDL.Text,
    'nick' : IDL.Text,
    'locale' : IDL.Text,
    'description' : IDL.Text,
  });
  const ZonBackend = IDL.Service({
    'createItemData' : IDL.Func(
        [ItemTransferWithoutOwner],
        [IDL.Principal, IDL.Nat],
        [],
      ),
    'getRootItem' : IDL.Func(
        [],
        [IDL.Opt(IDL.Tuple(IDL.Principal, IDL.Nat))],
        ['query'],
      ),
    'get_trusted_origins' : IDL.Func([], [IDL.Vec(IDL.Text)], []),
    'init' : IDL.Func([], [], []),
    'removeItem' : IDL.Func([IDL.Principal, IDL.Nat], [], ['oneway']),
    'removeMainOwner' : IDL.Func([], [], ['oneway']),
    'removeUser' : IDL.Func([IDL.Principal], [], ['oneway']),
    'setItemData' : IDL.Func(
        [IDL.Principal, IDL.Nat, ItemDataWithoutOwner],
        [],
        ['oneway'],
      ),
    'setMainOwner' : IDL.Func([IDL.Principal], [], ['oneway']),
    'setPostText' : IDL.Func(
        [IDL.Principal, IDL.Nat, IDL.Text],
        [],
        ['oneway'],
      ),
    'setRootItem' : IDL.Func([IDL.Principal, IDL.Nat], [], []),
    'setUserData' : IDL.Func([IDL.Opt(IDL.Principal), User], [], ['oneway']),
  });
  return ZonBackend;
};
export const init = ({ IDL }) => { return []; };
