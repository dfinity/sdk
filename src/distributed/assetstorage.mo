import Error "mo:base/Error";
import Tree "mo:base/RBTree";
import Text "mo:base/Text";
import Iter "mo:base/Iter";

shared {caller = creator} actor class () {

    public type Path = Text;
    public type Contents = Blob;

    let initializer : Principal = creator;

    let db: Tree.RBTree<Path, Contents> = Tree.RBTree(Text.compare);

    public shared { caller } func store(path : Path, contents : Contents) : async () {
        if (caller != initializer) {
            throw Error.reject("not authorized");
        } else {
            db.put(path, contents);
        };
    };

    public query func retrieve(path : Path) : async Contents {
        switch (db.get(path)) {
        case null throw Error.reject("not found");
        case (?contents) contents;
        };
    };

    public query func list() : async [Path] {
        let iter = Iter.map<(Path, Contents), Path>(db.entries(), func (path, _) = path);
        Iter.toArray(iter)
    };
};
