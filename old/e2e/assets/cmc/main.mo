import Array "mo:base/Array";
import P "mo:base/Principal";
import Text "mo:base/Text";
import Prim "mo:⛔";

actor Call {
  type SubnetTypesToSubnetsResponse = {
    data: [(Text, [Principal])];
  };

  public query func get_subnet_types_to_subnets() : async SubnetTypesToSubnetsResponse {
    let type1 = "type1";
    let type2 = "type2";
    {
      data = [
        (type1, [Prim.principalOfBlob("\00")]),
        (type2, [Prim.principalOfBlob("\01"), Prim.principalOfBlob("\02")]),
      ];
    }
  };

}
