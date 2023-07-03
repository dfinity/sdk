import Principal = "mo:base/Principal";

actor {
   public func always_succeed() : async () { };
   public func always_fail() : async () { };

   system func inspect(
     {
      caller : Principal;
      arg : Blob;
      msg : {
        #always_succeed : () -> ();
        #always_fail : () -> ();
      }
    }) : Bool {
         switch (msg) {
           case (#always_succeed _) { true };
           case (#always_fail _) { false };
         }
     };
};
