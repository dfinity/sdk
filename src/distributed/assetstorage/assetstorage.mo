import A "mo:base/AssocList";
import L "mo:base/List";
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
            case null {
                // more than 8 chars treated as invalid UTF-8
                // TODO: https://github.com/dfinity-lab/sdk/issues/701
                throw Prim.error("notfound")
            };
            case (?contents) { contents };
        }
    };
};
