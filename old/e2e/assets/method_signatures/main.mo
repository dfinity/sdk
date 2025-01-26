import Text "mo:base/Text";

actor {
  public query func returns_string(name: Text) : async Text {
    return "Hello, " # name # "!";
  };

  public query func returns_opt_string(name: ?Text) : async ?Text {
    return switch (name) {
      case null null;
      case (?x) ?("Hello, " # x # "!");
    };
  };

  public query func returns_int(v: Int) : async Int {
    return v;
  };

  public query func returns_int32(v: Int32) : async Int32 {
    return v;
  };

  public query func returns_principal(p: Principal) : async Principal {
    return p;
  };

  public query func returns_strings() : async [Text] {
    return ["Hello, world!", "Hello, Mars!"];
  };

  type ObjectReturnType = {
    foo: Text;
    bar: Int;
  };

  public query func returns_object() : async ObjectReturnType {
    return {foo = "baz"; bar = 42};
  };

  type VariantType = { #foo; #bar : Text; #baz : { a : Int32 }; };
  public query func returns_variant(i: Nat) : async VariantType {
    if (i == 0) {
      return #foo;
    } else if (i == 1) {
      return #bar("a bar");
    } else {
      return #baz({a = 51});
    }
  };

  public query func returns_blob(s: Text): async Blob {
    return Text.encodeUtf8(s);
  };

  public query func returns_tuple(): async (Text, Nat32, Text) {
    return ("the first element", 42, "the third element");
  };

  public query func returns_single_elem_tuple(): async (Text) {
    return ("the only element");
  };
}
