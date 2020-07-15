import A "mo:base/AssocList";
import D "mo:base/Debug";
import L "mo:base/List";
import P "mo:prim";

//////////////////
import Array "mo:base/Array";
import Iter "mo:base/Iter";
import Option "mo:base/Option";
import Prim "mo:prim";
import Result "mo:base/Result";
//////////////////

actor {

    public type Path = Text;

    public type Contents = Blob;

    private let initializer : Principal = P.caller();

    private stable var db: A.AssocList<Path, Contents> = L.nil();

    func eq(a: Path, b: Path): Bool {
        return a == b;
    };

    public shared { caller } func store(path : Path, contents : Contents) {
        //////////////////
        D.print("Caller is " # principalToText(caller));
        D.print("Initializer is " # principalToText(initializer));
        //////////////////
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

    //////////////////
    private func principalToText(p : Principal) : Text {
        return encode(Iter.toArray<Word8>(Prim.blobOfPrincipal(p).bytes()));
    };

    private type Result<Ok, Err> = Result.Result<Ok, Err>;

    private let base : Word8 = 0x10;

    private let symbols = [
        '0', '1', '2', '3', '4', '5', '6', '7',
        '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
    ];

    /**
     * Encode an array of unsigned 8-bit integers in hexadecimal format.
     */
    private func encode(array : [Word8]) : Text {
        Array.foldLeft<Word8, Text>(array, "", func (accum, w8) {
            accum # encodeW8(w8);
        });
    };

    /**
     * Encode an unsigned 8-bit integer in hexadecimal format.
     */
    private func encodeW8(w8 : Word8) : Text {
        let c1 = symbols[Prim.word8ToNat(w8 / base)];
        let c2 = symbols[Prim.word8ToNat(w8 % base)];
        Prim.charToText(c1) # Prim.charToText(c2);
    };
    //////////////////
};
