import A "mo:base/AssocList";
import L "mo:base/List";
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
        let result = A.find<Path, Contents>(db, path, eq);
        switch result {
            case null throw P.error("not found");
            case (?contents) contents;
        };
    };
};
