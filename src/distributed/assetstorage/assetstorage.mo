import A "mo:base/AssocList";
import L "mo:base/List";
import O "mo:base/Option";
import P "mo:prim";

actor {

    public type Path = Text;

    public type Contents = Blob;

    private let initializer : Principal = P.caller();

    private stable var db: A.AssocList<Path, Contents> = L.nil();

    func eq(a: Path, b: Path): Bool {
        return a == b;
    };

    public shared { caller } func store(path : Path, contents : Contents) {
        if (caller != initializer) {
            throw P.error("not authorized")
        } else {
            db := A.replace<Path, Contents>(db, path, eq, ?contents).0;
        };
    };

    public query func retrieve(path : Path) : async Contents {
        O.get(A.find<Path, Contents>(db, path, eq), {
            throw P.error("not found")
        });
    };
};
