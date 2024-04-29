export const idlFactory = ({ IDL }) => {
  const OuterCanister = IDL.Rec();
  const Tree = IDL.Rec();
  const AutoScalingCanisterSharedFunctionHook = IDL.Func(
      [IDL.Text],
      [IDL.Text],
      [],
    );
  const ScalingLimitType = IDL.Variant({
    'heapSize' : IDL.Nat,
    'count' : IDL.Nat,
  });
  const ScalingOptions = IDL.Record({
    'autoScalingHook' : AutoScalingCanisterSharedFunctionHook,
    'sizeLimit' : ScalingLimitType,
  });
  const SK = IDL.Text;
  const DeleteOptions = IDL.Record({ 'sk' : SK });
  const GetOptions = IDL.Record({ 'sk' : SK });
  const PK = IDL.Text;
  const Color = IDL.Variant({ 'B' : IDL.Null, 'R' : IDL.Null });
  const AttributeKey = IDL.Text;
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
  Tree.fill(
    IDL.Variant({
      'leaf' : IDL.Null,
      'node' : IDL.Tuple(
        Color,
        Tree,
        IDL.Tuple(AttributeKey, IDL.Opt(AttributeValue)),
        Tree,
      ),
    })
  );
  const AttributeMap = IDL.Variant({
    'leaf' : IDL.Null,
    'node' : IDL.Tuple(
      Color,
      Tree,
      IDL.Tuple(AttributeKey, IDL.Opt(AttributeValue)),
      Tree,
    ),
  });
  const Entity = IDL.Record({
    'pk' : PK,
    'sk' : SK,
    'attributes' : AttributeMap,
  });
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
  const ItemData = IDL.Record({
    'creator' : IDL.Principal,
    'edited' : IDL.Bool,
    'item' : ItemDataWithoutOwner,
  });
  const ItemTransfer = IDL.Record({ 'data' : ItemData, 'communal' : IDL.Bool });
  const OuterSubDBKey = IDL.Nat;
  const InnerSubDBKey = IDL.Nat;
  const SK__1 = IDL.Text;
  const AttributeValuePrimitive__1 = IDL.Variant({
    'int' : IDL.Int,
    'float' : IDL.Float64,
    'bool' : IDL.Bool,
    'text' : IDL.Text,
  });
  const AttributeValue__1 = IDL.Variant({
    'int' : IDL.Int,
    'float' : IDL.Float64,
    'tuple' : IDL.Vec(AttributeValuePrimitive__1),
    'bool' : IDL.Bool,
    'text' : IDL.Text,
    'arrayBool' : IDL.Vec(IDL.Bool),
    'arrayText' : IDL.Vec(IDL.Text),
    'arrayInt' : IDL.Vec(IDL.Int),
    'arrayFloat' : IDL.Vec(IDL.Float64),
  });
  const OuterPair = IDL.Record({
    'key' : OuterSubDBKey,
    'canister' : OuterCanister,
  });
  const GetByOuterPartitionKeyOptions = IDL.Record({
    'sk' : SK__1,
    'outer' : OuterPair,
  });
  const GetUserDataOuterOptions = IDL.Record({ 'outer' : OuterPair });
  const Direction = IDL.Variant({ 'bwd' : IDL.Null, 'fwd' : IDL.Null });
  const ScanLimitResult = IDL.Record({
    'results' : IDL.Vec(IDL.Tuple(IDL.Text, AttributeValue__1)),
    'nextKey' : IDL.Opt(IDL.Text),
  });
  const SubDBSizeOuterOptions = IDL.Record({ 'outer' : OuterPair });
  OuterCanister.fill(
    IDL.Service({
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
          [IDL.Record({ 'sk' : SK__1, 'innerKey' : InnerSubDBKey })],
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
          [IDL.Record({ 'sk' : SK__1, 'innerKey' : InnerSubDBKey })],
          [IDL.Opt(AttributeValue__1)],
          ['query'],
        ),
      'getByOuter' : IDL.Func(
          [IDL.Record({ 'sk' : SK__1, 'outerKey' : OuterSubDBKey })],
          [IDL.Opt(AttributeValue__1)],
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
          [IDL.Opt(AttributeValue__1)],
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
          [IDL.Record({ 'sk' : SK__1, 'innerKey' : InnerSubDBKey })],
          [IDL.Bool],
          ['query'],
        ),
      'hasByOuter' : IDL.Func(
          [IDL.Record({ 'sk' : SK__1, 'outerKey' : OuterSubDBKey })],
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
                'map' : IDL.Vec(IDL.Tuple(SK__1, AttributeValue__1)),
                'userData' : IDL.Text,
              })
            ),
          ],
          ['query'],
        ),
      'rawInsertSubDB' : IDL.Func(
          [
            IDL.Record({
              'map' : IDL.Vec(IDL.Tuple(SK__1, AttributeValue__1)),
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
              'map' : IDL.Vec(IDL.Tuple(SK__1, AttributeValue__1)),
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
              'lowerBound' : SK__1,
              'limit' : IDL.Nat,
              'upperBound' : SK__1,
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
              'lowerBound' : SK__1,
              'limit' : IDL.Nat,
              'upperBound' : SK__1,
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
                IDL.Record({
                  'key' : InnerSubDBKey,
                  'canister' : IDL.Principal,
                }),
              )
            ),
          ],
          ['query'],
        ),
      'startInsertingImpl' : IDL.Func(
          [
            IDL.Record({
              'sk' : SK__1,
              'value' : AttributeValue__1,
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
    })
  );
  const Order = IDL.Record({
    'reverse' : IDL.Tuple(OuterCanister, OuterSubDBKey),
    'order' : IDL.Tuple(OuterCanister, OuterSubDBKey),
  });
  const Streams = IDL.Vec(IDL.Opt(Order));
  const PutOptions = IDL.Record({
    'sk' : SK,
    'attributes' : IDL.Vec(IDL.Tuple(AttributeKey, AttributeValue)),
  });
  const ScanOptions = IDL.Record({
    'limit' : IDL.Nat,
    'ascending' : IDL.Opt(IDL.Bool),
    'skLowerBound' : SK,
    'skUpperBound' : SK,
  });
  const ScanResult = IDL.Record({
    'entities' : IDL.Vec(Entity),
    'nextKey' : IDL.Opt(SK),
  });
  const CanDBPartition = IDL.Service({
    'delete' : IDL.Func([DeleteOptions], [], []),
    'get' : IDL.Func([GetOptions], [IDL.Opt(Entity)], ['query']),
    'getAttribute' : IDL.Func(
        [GetOptions, IDL.Text],
        [IDL.Opt(AttributeValue)],
        ['query'],
      ),
    'getItem' : IDL.Func([IDL.Nat], [IDL.Opt(ItemTransfer)], []),
    'getOwners' : IDL.Func([], [IDL.Vec(IDL.Principal)], ['query']),
    'getPK' : IDL.Func([], [IDL.Text], ['query']),
    'getStreams' : IDL.Func([IDL.Nat, IDL.Text], [IDL.Opt(Streams)], ['query']),
    'put' : IDL.Func([PutOptions], [], []),
    'putAttribute' : IDL.Func(
        [
          IDL.Record({
            'sk' : SK,
            'key' : AttributeKey,
            'value' : AttributeValue,
          }),
        ],
        [],
        [],
      ),
    'putExisting' : IDL.Func([PutOptions], [IDL.Bool], []),
    'putExistingAttribute' : IDL.Func(
        [
          IDL.Record({
            'sk' : SK,
            'key' : AttributeKey,
            'value' : AttributeValue,
          }),
        ],
        [IDL.Bool],
        [],
      ),
    'scan' : IDL.Func([ScanOptions], [ScanResult], ['query']),
    'setOwners' : IDL.Func([IDL.Vec(IDL.Principal)], [], []),
    'skExists' : IDL.Func([IDL.Text], [IDL.Bool], ['query']),
    'transferCycles' : IDL.Func([], [], []),
  });
  return CanDBPartition;
};
export const init = ({ IDL }) => {
  const AutoScalingCanisterSharedFunctionHook = IDL.Func(
      [IDL.Text],
      [IDL.Text],
      [],
    );
  const ScalingLimitType = IDL.Variant({
    'heapSize' : IDL.Nat,
    'count' : IDL.Nat,
  });
  const ScalingOptions = IDL.Record({
    'autoScalingHook' : AutoScalingCanisterSharedFunctionHook,
    'sizeLimit' : ScalingLimitType,
  });
  return [
    IDL.Record({
      'owners' : IDL.Opt(IDL.Vec(IDL.Principal)),
      'partitionKey' : IDL.Text,
      'scalingOptions' : ScalingOptions,
    }),
  ];
};
