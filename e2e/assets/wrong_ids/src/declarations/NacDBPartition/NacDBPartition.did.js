export const idlFactory = ({ IDL }) => {
  const GetByOuterPartitionKeyOptions = IDL.Rec();
  const OuterPair = IDL.Rec();
  const OuterSubDBKey = IDL.Nat;
  const InnerSubDBKey = IDL.Nat;
  const SK = IDL.Text;
  const AttributeValuePrimitive = IDL.Variant({
    'int' : IDL.Int,
    'float' : IDL.Float64,
    'bool' : IDL.Bool,
    'text' : IDL.Text,
  });
  const AttributeValue = IDL.Variant({
    'int' : IDL.Int,
    'float' : IDL.Float64,
    'tuple' : IDL.Vec(AttributeValuePrimitive),
    'bool' : IDL.Bool,
    'text' : IDL.Text,
    'arrayBool' : IDL.Vec(IDL.Bool),
    'arrayText' : IDL.Vec(IDL.Text),
    'arrayInt' : IDL.Vec(IDL.Int),
    'arrayFloat' : IDL.Vec(IDL.Float64),
  });
  const GetUserDataOuterOptions = IDL.Record({ 'outer' : OuterPair });
  const Direction = IDL.Variant({ 'bwd' : IDL.Null, 'fwd' : IDL.Null });
  const ScanLimitResult = IDL.Record({
    'results' : IDL.Vec(IDL.Tuple(IDL.Text, AttributeValue)),
    'nextKey' : IDL.Opt(IDL.Text),
  });
  const SubDBSizeOuterOptions = IDL.Record({ 'outer' : OuterPair });
  const OuterCanister = IDL.Service({
    'createOuter' : IDL.Func(
        [
          IDL.Record({
            'part' : IDL.Principal,
            'outerKey' : OuterSubDBKey,
            'innerKey' : InnerSubDBKey,
          }),
        ],
        [
          IDL.Record({
            'outer' : IDL.Record({
              'key' : OuterSubDBKey,
              'canister' : IDL.Principal,
            }),
            'inner' : IDL.Record({
              'key' : InnerSubDBKey,
              'canister' : IDL.Principal,
            }),
          }),
        ],
        [],
      ),
    'deleteInner' : IDL.Func(
        [IDL.Record({ 'sk' : SK, 'innerKey' : InnerSubDBKey })],
        [],
        [],
      ),
    'deleteSubDBInner' : IDL.Func(
        [IDL.Record({ 'innerKey' : InnerSubDBKey })],
        [],
        [],
      ),
    'deleteSubDBOuter' : IDL.Func(
        [IDL.Record({ 'outerKey' : OuterSubDBKey })],
        [],
        [],
      ),
    'getByInner' : IDL.Func(
        [IDL.Record({ 'sk' : SK, 'innerKey' : InnerSubDBKey })],
        [IDL.Opt(AttributeValue)],
        ['query'],
      ),
    'getByOuter' : IDL.Func(
        [IDL.Record({ 'sk' : SK, 'outerKey' : OuterSubDBKey })],
        [IDL.Opt(AttributeValue)],
        [],
      ),
    'getInner' : IDL.Func(
        [IDL.Record({ 'outerKey' : OuterSubDBKey })],
        [
          IDL.Opt(
            IDL.Record({ 'key' : InnerSubDBKey, 'canister' : IDL.Principal })
          ),
        ],
        ['query'],
      ),
    'getOuter' : IDL.Func(
        [GetByOuterPartitionKeyOptions],
        [IDL.Opt(AttributeValue)],
        [],
      ),
    'getSubDBUserDataInner' : IDL.Func(
        [IDL.Record({ 'innerKey' : InnerSubDBKey })],
        [IDL.Opt(IDL.Text)],
        [],
      ),
    'getSubDBUserDataOuter' : IDL.Func(
        [GetUserDataOuterOptions],
        [IDL.Opt(IDL.Text)],
        [],
      ),
    'hasByInner' : IDL.Func(
        [IDL.Record({ 'sk' : SK, 'innerKey' : InnerSubDBKey })],
        [IDL.Bool],
        ['query'],
      ),
    'hasByOuter' : IDL.Func(
        [IDL.Record({ 'sk' : SK, 'outerKey' : OuterSubDBKey })],
        [IDL.Bool],
        [],
      ),
    'hasSubDBByInner' : IDL.Func(
        [IDL.Record({ 'innerKey' : InnerSubDBKey })],
        [IDL.Bool],
        ['query'],
      ),
    'hasSubDBByOuter' : IDL.Func(
        [IDL.Record({ 'outerKey' : OuterSubDBKey })],
        [IDL.Bool],
        [],
      ),
    'isOverflowed' : IDL.Func([], [IDL.Bool], ['query']),
    'putLocation' : IDL.Func(
        [
          IDL.Record({
            'newInnerSubDBKey' : InnerSubDBKey,
            'innerCanister' : IDL.Principal,
            'outerKey' : OuterSubDBKey,
          }),
        ],
        [],
        [],
      ),
    'rawDeleteSubDB' : IDL.Func(
        [IDL.Record({ 'innerKey' : InnerSubDBKey })],
        [],
        [],
      ),
    'rawGetSubDB' : IDL.Func(
        [IDL.Record({ 'innerKey' : InnerSubDBKey })],
        [
          IDL.Opt(
            IDL.Record({
              'map' : IDL.Vec(IDL.Tuple(SK, AttributeValue)),
              'userData' : IDL.Text,
            })
          ),
        ],
        ['query'],
      ),
    'rawInsertSubDB' : IDL.Func(
        [
          IDL.Record({
            'map' : IDL.Vec(IDL.Tuple(SK, AttributeValue)),
            'userData' : IDL.Text,
            'hardCap' : IDL.Opt(IDL.Nat),
            'innerKey' : IDL.Opt(InnerSubDBKey),
          }),
        ],
        [IDL.Record({ 'innerKey' : InnerSubDBKey })],
        [],
      ),
    'rawInsertSubDBAndSetOuter' : IDL.Func(
        [
          IDL.Record({
            'map' : IDL.Vec(IDL.Tuple(SK, AttributeValue)),
            'userData' : IDL.Text,
            'keys' : IDL.Opt(
              IDL.Record({
                'outerKey' : OuterSubDBKey,
                'innerKey' : InnerSubDBKey,
              })
            ),
            'hardCap' : IDL.Opt(IDL.Nat),
          }),
        ],
        [
          IDL.Record({
            'outerKey' : OuterSubDBKey,
            'innerKey' : InnerSubDBKey,
          }),
        ],
        [],
      ),
    'scanLimitInner' : IDL.Func(
        [
          IDL.Record({
            'dir' : Direction,
            'lowerBound' : SK,
            'limit' : IDL.Nat,
            'upperBound' : SK,
            'innerKey' : InnerSubDBKey,
          }),
        ],
        [ScanLimitResult],
        ['query'],
      ),
    'scanLimitOuter' : IDL.Func(
        [
          IDL.Record({
            'dir' : Direction,
            'lowerBound' : SK,
            'limit' : IDL.Nat,
            'upperBound' : SK,
            'outerKey' : OuterSubDBKey,
          }),
        ],
        [ScanLimitResult],
        [],
      ),
    'scanSubDBs' : IDL.Func(
        [],
        [
          IDL.Vec(
            IDL.Tuple(
              OuterSubDBKey,
              IDL.Record({ 'key' : InnerSubDBKey, 'canister' : IDL.Principal }),
            )
          ),
        ],
        ['query'],
      ),
    'startInsertingImpl' : IDL.Func(
        [
          IDL.Record({
            'sk' : SK,
            'value' : AttributeValue,
            'innerKey' : InnerSubDBKey,
          }),
        ],
        [],
        [],
      ),
    'subDBSizeByInner' : IDL.Func(
        [IDL.Record({ 'innerKey' : InnerSubDBKey })],
        [IDL.Opt(IDL.Nat)],
        ['query'],
      ),
    'subDBSizeByOuter' : IDL.Func(
        [IDL.Record({ 'outerKey' : OuterSubDBKey })],
        [IDL.Opt(IDL.Nat)],
        [],
      ),
    'subDBSizeOuterImpl' : IDL.Func(
        [SubDBSizeOuterOptions],
        [IDL.Opt(IDL.Nat)],
        [],
      ),
    'superDBSize' : IDL.Func([], [IDL.Nat], ['query']),
  });
  OuterPair.fill(
    IDL.Record({ 'key' : OuterSubDBKey, 'canister' : OuterCanister })
  );
  GetByOuterPartitionKeyOptions.fill(
    IDL.Record({ 'sk' : SK, 'outer' : OuterPair })
  );
  const Direction__1 = IDL.Variant({ 'bwd' : IDL.Null, 'fwd' : IDL.Null });
  const ScanLimitResult__1 = IDL.Record({
    'results' : IDL.Vec(IDL.Tuple(IDL.Text, AttributeValue)),
    'nextKey' : IDL.Opt(IDL.Text),
  });
  const Partition = IDL.Service({
    'createOuter' : IDL.Func(
        [
          IDL.Record({
            'part' : IDL.Principal,
            'outerKey' : OuterSubDBKey,
            'innerKey' : InnerSubDBKey,
          }),
        ],
        [
          IDL.Record({
            'outer' : IDL.Record({
              'key' : OuterSubDBKey,
              'canister' : IDL.Principal,
            }),
            'inner' : IDL.Record({
              'key' : InnerSubDBKey,
              'canister' : IDL.Principal,
            }),
          }),
        ],
        [],
      ),
    'deleteInner' : IDL.Func(
        [IDL.Record({ 'sk' : SK, 'innerKey' : InnerSubDBKey })],
        [],
        [],
      ),
    'deleteSubDBInner' : IDL.Func(
        [IDL.Record({ 'innerKey' : InnerSubDBKey })],
        [],
        [],
      ),
    'deleteSubDBOuter' : IDL.Func(
        [IDL.Record({ 'outerKey' : OuterSubDBKey })],
        [],
        [],
      ),
    'getByInner' : IDL.Func(
        [IDL.Record({ 'sk' : SK, 'innerKey' : InnerSubDBKey })],
        [IDL.Opt(AttributeValue)],
        ['query'],
      ),
    'getByOuter' : IDL.Func(
        [IDL.Record({ 'sk' : SK, 'outerKey' : OuterSubDBKey })],
        [IDL.Opt(AttributeValue)],
        [],
      ),
    'getInner' : IDL.Func(
        [IDL.Record({ 'outerKey' : OuterSubDBKey })],
        [
          IDL.Opt(
            IDL.Record({ 'key' : InnerSubDBKey, 'canister' : IDL.Principal })
          ),
        ],
        ['query'],
      ),
    'getOuter' : IDL.Func(
        [GetByOuterPartitionKeyOptions],
        [IDL.Opt(AttributeValue)],
        [],
      ),
    'getOwners' : IDL.Func([], [IDL.Vec(IDL.Principal)], ['query']),
    'getSubDBUserDataInner' : IDL.Func(
        [IDL.Record({ 'innerKey' : InnerSubDBKey })],
        [IDL.Opt(IDL.Text)],
        [],
      ),
    'getSubDBUserDataOuter' : IDL.Func(
        [GetUserDataOuterOptions],
        [IDL.Opt(IDL.Text)],
        [],
      ),
    'hasByInner' : IDL.Func(
        [IDL.Record({ 'sk' : SK, 'innerKey' : InnerSubDBKey })],
        [IDL.Bool],
        ['query'],
      ),
    'hasByOuter' : IDL.Func(
        [IDL.Record({ 'sk' : SK, 'outerKey' : OuterSubDBKey })],
        [IDL.Bool],
        [],
      ),
    'hasSubDBByInner' : IDL.Func(
        [IDL.Record({ 'innerKey' : InnerSubDBKey })],
        [IDL.Bool],
        ['query'],
      ),
    'hasSubDBByOuter' : IDL.Func(
        [IDL.Record({ 'outerKey' : OuterSubDBKey })],
        [IDL.Bool],
        [],
      ),
    'isOverflowed' : IDL.Func([], [IDL.Bool], ['query']),
    'putLocation' : IDL.Func(
        [
          IDL.Record({
            'newInnerSubDBKey' : InnerSubDBKey,
            'innerCanister' : IDL.Principal,
            'outerKey' : OuterSubDBKey,
          }),
        ],
        [],
        [],
      ),
    'rawDeleteSubDB' : IDL.Func(
        [IDL.Record({ 'innerKey' : InnerSubDBKey })],
        [],
        [],
      ),
    'rawGetSubDB' : IDL.Func(
        [IDL.Record({ 'innerKey' : InnerSubDBKey })],
        [
          IDL.Opt(
            IDL.Record({
              'map' : IDL.Vec(IDL.Tuple(SK, AttributeValue)),
              'userData' : IDL.Text,
            })
          ),
        ],
        ['query'],
      ),
    'rawInsertSubDB' : IDL.Func(
        [
          IDL.Record({
            'map' : IDL.Vec(IDL.Tuple(SK, AttributeValue)),
            'userData' : IDL.Text,
            'hardCap' : IDL.Opt(IDL.Nat),
            'innerKey' : IDL.Opt(InnerSubDBKey),
          }),
        ],
        [IDL.Record({ 'innerKey' : InnerSubDBKey })],
        [],
      ),
    'rawInsertSubDBAndSetOuter' : IDL.Func(
        [
          IDL.Record({
            'map' : IDL.Vec(IDL.Tuple(SK, AttributeValue)),
            'userData' : IDL.Text,
            'keys' : IDL.Opt(
              IDL.Record({
                'outerKey' : OuterSubDBKey,
                'innerKey' : InnerSubDBKey,
              })
            ),
            'hardCap' : IDL.Opt(IDL.Nat),
          }),
        ],
        [
          IDL.Record({
            'outerKey' : OuterSubDBKey,
            'innerKey' : InnerSubDBKey,
          }),
        ],
        [],
      ),
    'scanLimitInner' : IDL.Func(
        [
          IDL.Record({
            'dir' : Direction__1,
            'lowerBound' : SK,
            'limit' : IDL.Nat,
            'upperBound' : SK,
            'innerKey' : InnerSubDBKey,
          }),
        ],
        [ScanLimitResult__1],
        ['query'],
      ),
    'scanLimitOuter' : IDL.Func(
        [
          IDL.Record({
            'dir' : Direction__1,
            'lowerBound' : SK,
            'limit' : IDL.Nat,
            'upperBound' : SK,
            'outerKey' : OuterSubDBKey,
          }),
        ],
        [ScanLimitResult__1],
        [],
      ),
    'scanSubDBs' : IDL.Func(
        [],
        [
          IDL.Vec(
            IDL.Tuple(
              OuterSubDBKey,
              IDL.Record({ 'key' : InnerSubDBKey, 'canister' : IDL.Principal }),
            )
          ),
        ],
        ['query'],
      ),
    'setOwners' : IDL.Func([IDL.Vec(IDL.Principal)], [], []),
    'startInsertingImpl' : IDL.Func(
        [
          IDL.Record({
            'sk' : SK,
            'value' : AttributeValue,
            'innerKey' : InnerSubDBKey,
          }),
        ],
        [],
        [],
      ),
    'subDBSizeByInner' : IDL.Func(
        [IDL.Record({ 'innerKey' : InnerSubDBKey })],
        [IDL.Opt(IDL.Nat)],
        ['query'],
      ),
    'subDBSizeByOuter' : IDL.Func(
        [IDL.Record({ 'outerKey' : OuterSubDBKey })],
        [IDL.Opt(IDL.Nat)],
        [],
      ),
    'subDBSizeOuterImpl' : IDL.Func(
        [SubDBSizeOuterOptions],
        [IDL.Opt(IDL.Nat)],
        [],
      ),
    'superDBSize' : IDL.Func([], [IDL.Nat], ['query']),
  });
  return Partition;
};
export const init = ({ IDL }) => { return [IDL.Vec(IDL.Principal)]; };
