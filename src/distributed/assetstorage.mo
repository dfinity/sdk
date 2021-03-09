import Error "mo:base/Error";
import Iter "mo:base/Iter";
import Array "mo:base/Array";
import Text "mo:base/Text";
import Tree "mo:base/RBTree";

shared ({caller = creator}) actor class () {

    public type Path = Text;
    public type Contents = Blob;

    public type HeaderField = (Text, Text);

    public type HttpRequest = {
        method: Text;
        url: Text;
        headers: [HeaderField];
        body: Blob;
    };
    public type HttpResponse = {
        status_code: Nat16;
        headers: [HeaderField];
        body: Blob;
    };

    stable var authorized: [Principal] = [creator];

    let db: Tree.RBTree<Path, Contents> = Tree.RBTree(Text.compare);

    public shared ({ caller }) func authorize(other: Principal) : async () {
        if (isSafe(caller)) {
            authorized := Array.append<Principal>(authorized, [other]);
        } else {
            throw Error.reject("not authorized");
        }
    };

    public shared ({ caller }) func store(path : Path, contents : Contents) : async () {
        if (isSafe(caller)) {
            db.put(path, contents);
        } else {
            throw Error.reject("not authorized");
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

    func isSafe(caller: Principal) : Bool {
        func eq(value: Principal): Bool = value == caller;
        Array.find(authorized, eq) != null
    };

    public query func http_request(request: HttpRequest): async HttpResponse {
        let content = getContent(request.url);

        switch (content) {
            case null {{ status_code = 404; headers = []; body = "" }};
            case (?c) {{ status_code = 200; headers = []; body = c }};
        }
    };

    private func getContent(uri: Text): ?Blob {
        let splitted = Text.split(uri, #char '?');
        let array = Iter.toArray<Text>(splitted);
        let path = array[0];
        switch (db.get(path)) {
            case null db.get("/index.html");
            case (?contents) ?contents;
        }
    };
};
