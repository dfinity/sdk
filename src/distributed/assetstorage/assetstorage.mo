import A "mo:base/AssocList";
import B "mo:base/Blob";
import L "mo:base/List";
import P "mo:base/Prelude";

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
              // what is the syntax to throw?
              P.nyi()
            };
            case (?contents) { contents };
        }
    };
};
