export const idlFactory = ({ IDL }) => {
  const SK = IDL.Text;
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
  const Time = IDL.Int;
  const VotingScore = IDL.Record({
    'ethereumAddress' : IDL.Text,
    'lastChecked' : Time,
    'points' : IDL.Float64,
  });
  const InterCanisterActionResult = IDL.Variant({
    'ok' : IDL.Null,
    'err' : IDL.Text,
  });
  const UpgradePKRangeResult = IDL.Record({
    'nextKey' : IDL.Opt(IDL.Text),
    'upgradeCanisterResults' : IDL.Vec(
      IDL.Tuple(IDL.Text, InterCanisterActionResult)
    ),
  });
  const CanDBIndex = IDL.Service({
    'autoScaleCanister' : IDL.Func([IDL.Text], [IDL.Text], []),
    'checkSybil' : IDL.Func([IDL.Principal], [], []),
    'getCanistersByPK' : IDL.Func([IDL.Text], [IDL.Vec(IDL.Text)], ['query']),
    'getFirstAttribute' : IDL.Func(
        [IDL.Text, IDL.Record({ 'sk' : SK, 'key' : AttributeKey })],
        [IDL.Opt(IDL.Tuple(IDL.Principal, IDL.Opt(AttributeValue)))],
        [],
      ),
    'getOwners' : IDL.Func([], [IDL.Vec(IDL.Principal)], ['query']),
    'init' : IDL.Func([IDL.Vec(IDL.Principal)], [], []),
    'putAttributeNoDuplicates' : IDL.Func(
        [
          IDL.Text,
          IDL.Record({
            'sk' : SK,
            'key' : AttributeKey,
            'value' : AttributeValue,
          }),
        ],
        [IDL.Principal],
        [],
      ),
    'putAttributeWithPossibleDuplicate' : IDL.Func(
        [
          IDL.Text,
          IDL.Record({
            'sk' : SK,
            'key' : AttributeKey,
            'value' : AttributeValue,
          }),
        ],
        [IDL.Principal],
        [],
      ),
    'setOwners' : IDL.Func([IDL.Vec(IDL.Principal)], [], []),
    'setVotingData' : IDL.Func(
        [IDL.Principal, IDL.Opt(IDL.Principal), VotingScore],
        [],
        [],
      ),
    'sybilScore' : IDL.Func([], [IDL.Bool, IDL.Float64], []),
    'upgradeAllPartitionCanisters' : IDL.Func(
        [IDL.Vec(IDL.Nat8)],
        [UpgradePKRangeResult],
        [],
      ),
  });
  return CanDBIndex;
};
export const init = ({ IDL }) => { return []; };
