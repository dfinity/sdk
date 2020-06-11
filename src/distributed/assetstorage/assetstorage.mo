import A "mo:base/AssocList";
import B "mo:base/Blob";
import L "mo:base/List";
import P "mo:base/Prelude";
import Prim "mo:prim";

actor {
    public type Path = Text;
    public type Contents = Blob;

    var stored: A.AssocList<Path, Contents> = L.nil<(Path, Contents)>();

    func pathEq(a: Path, b: Path): Bool {
        return a == b;
    };

    public func store(path : Path, contents: Contents) : async () {
        let (newStored, _) = A.replace<Path, Contents>(stored, path, pathEq, ?contents);
        stored := newStored;
    };

    public query func retrieve(path: Path): async Contents {
        let result = A.find<Path, Contents>(stored, path, pathEq);
        switch result {
            case null { throw Prim.error("asset '" # asset # "' not found") };
            case (?contents) { contents };
        }
    };

    public query func die(): async ()) {
        throw Prim.error("just throwing");
    }
};
