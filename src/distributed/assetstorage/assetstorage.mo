import AssocList "mo:base/AssocList";
import Debug "mo:base/Debug";
import List "mo:base/List";
import Prelude "mo:base/Prelude";
import Prim "mo:prim";

actor {

    public type Path = Text;

    public type Contents = Blob;

    private let initializer : Principal = Prim.caller();

    private stable var db: AssocList.AssocList<Path, Contents> = List.nil();

    func eq(a: Path, b: Path): Bool {
        return a == b;
    };

    public shared { caller } func store(path : Path, contents : Contents) {
        if (caller != initializer) {
            Debug.print("not authorized");
            Prelude.unreachable();
        } else {
            db := AssocList.replace<Path, Contents>(db, path, eq, ?contents).0;
        };
    };

    public query func retrieve(path : Path) : async Contents {
        let result = AssocList.find<Path, Contents>(db, path, eq);
        switch result {
            case null {
                Debug.print("not found");
                Prelude.unreachable();
            };
            case (?contents) contents;
        };
    };
};
