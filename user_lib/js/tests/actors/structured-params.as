type O = { who : Text; what : Text; count : Int };

type RegionInfo = {
  id : Nat;
  short_name : Text;
  description : Text;
};


actor {
  public func hello (data : [O]) : async Text {
    var r = "Reporting this:\n";
    for (o in data.vals()) {
      r := r # o.who # " drank " # debug_show (o.count) # " bottles of " # o.what # ".\n"
    };
    return r;
  };

  public func returnRecord () : async [RegionInfo] {
    return [
      new {
        id = 1;
        short_name = "C";
        description= "Central";
      }
    ];
  };
}
