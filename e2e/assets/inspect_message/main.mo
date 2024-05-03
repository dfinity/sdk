import Principal = "mo:base/Principal";

actor {
   public func always_accepted() : async () { };
   public func always_rejected() : async () { };

   system func inspect(
     {
      caller : Principal;
      arg : Blob;
      msg : {
        #always_accepted : () -> ();
        #always_rejected : () -> ();
      }
    }) : Bool {
         switch (msg) {
           case (#always_accepted _) { true };
           case (#always_rejected _) { false };
         }
     };
};
