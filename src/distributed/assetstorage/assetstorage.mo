import A "mo:base/AssocList";
import L "mo:base/List";

actor {
    public type Path = Text;
    public type Contents = Text;

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
            case null { assert false; "" };
            case (?contents) { contents };
        }
    };
};
