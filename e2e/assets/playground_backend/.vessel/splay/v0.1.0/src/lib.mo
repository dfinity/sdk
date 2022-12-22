/// Splay tree
///
/// Based on Sam Westrick's SML implementation: https://github.com/shwestrick/pure-splay/blob/master/BottomUpSplay.sml
/// Note for using this library in IC: Since lookup changes the shape of the splay tree, we cannot use query calls
/// for anything that touches the splay tree.
///
/// ```motoko
/// import Splay "mo:splay";
///
/// let t = Splay.Splay<Int>(Int.compare);
/// t.fromArray([3,5,4,2,6,4,1,9,7,8]);
/// for (x in arr.vals()) {
///    assert(t.find(x) == true);
///    t.remove(x);
///    assert(t.find(x) == false);
///    t.insert(x);
///    assert(t.find(x) == true);
///    assert(t.min() == ?1);
/// };
/// ```
/// 

import O "mo:base/Order";
import List "mo:base/List";
import I "mo:base/Iter";

module {
    public type Tree<X> = {
        #node: (Tree<X>, X, Tree<X>);
        #empty;
    };
    type Context<X> = { #right: (Tree<X>, X); #left: (X, Tree<X>) };
    type Zipper<X> = List.List<Context<X>>;
    type Path<X> = (Tree<X>, Zipper<X>);

    func path<X>(compareTo: (X,X) -> O.Order, k: X, (t, anc): Path<X>) : Path<X> {
        switch t {
        case (#empty) { (t, anc) };
        case (#node(L,x,R)) {
                 switch (compareTo(k,x)) {
                 case (#less) { path(compareTo, k, (L, List.push(#left(x, R), anc))) };
                 case (#equal) { (t, anc) };
                 case (#greater) { path(compareTo, k, (R, List.push(#right(L, x), anc))) };
                 };
             };
        };
    };
    func splay_<X>(compareTo: (X,X) -> O.Order, A: Tree<X>, B: Tree<X>, anc: Zipper<X>) : (Tree<X>, Tree<X>) {
        switch anc {
        case (null) { (A, B) };
        case (?(#left(p, C), null)) { (A, #node(B,p,C)) }; // zig
        case (?(#right(C, p), null)) { (#node(C,p,A), B) }; // zag
        case (?(#left(p, C), ?(#left(g, D), anc))) { // zig-zig
                 splay_(compareTo, A, #node(B,p,#node(C,g,D)), anc)
             };
        case (?(#right(D,p), ?(#right(C, g), anc))) { // zag-zag
                 splay_(compareTo, #node(#node(C,g,D),p,A), B, anc)
             };
        case (?(#right(C,p), ?(#left(g,D), anc))) { // zig-zag
                 splay_(compareTo, #node(C,p,A), #node(B,g,D), anc)
             };
        case (?(#left(p,D), ?(#right(C,g), anc))) { // zag-zig
                 splay_(compareTo, #node(C,g,A), #node(B,p,D), anc)
             };
        };
    };
    func splay<X>(compareTo: (X,X) -> O.Order, l: Tree<X>, x: X, r: Tree<X>, anc: Zipper<X>) : Tree<X> {
        let (l_,r_) = splay_(compareTo, l, r, anc);
        #node(l_,x,r_)
    };
    func subtree_max<X>(t: Tree<X>) : ?X {
        switch t {
        case (#empty) { null };
        case (#node(_, x, #empty)) { ?x };
        case (#node(_,_,r)) { subtree_max(r) };
        };
    };
    func subtree_min<X>(t: Tree<X>) : ?X {
        switch t {
        case (#empty) { null };
        case (#node(#empty, x, _)) { ?x };
        case (#node(l,_,_)) { subtree_min(l) };
        };
    };
    type IterRep<X> = List.List<{ #tr:Tree<X>; #x:X }>;
    func iter<X>(t: Tree<X>) : I.Iter<X> {
        object {
            var trees: IterRep<X> = ?(#tr(t), null);
            public func next() : ?X {
                switch trees {
                case null { null };
                case (?(#tr(#empty), ts)) {
                         trees := ts;
                         next()
                     };
                case (?(#x(x), ts)) {
                         trees := ts;
                         ?x
                     };
                case (?(#tr(#node(l, x, r)), ts)) {
                         trees := ?(#tr(l), ?(#x(x), ?(#tr(r), ts)));
                         next()
                     };
                };
            }
        }
    };
    
    public class Splay<X>(compareTo: (X,X) -> O.Order) {
        var tree : Tree<X> = #empty;
        public func insert(k: X) {
            switch (path(compareTo, k, (tree, null))) {
            case ((#node(l,_,r), anc)) {
                     tree := splay(compareTo, l, k, r, anc);
                 };
            case ((#empty, anc)) {
                     tree := splay(compareTo, #empty, k, #empty, anc);
                 };
            };
        };
        public func find(k: X) : Bool {
            switch (path(compareTo, k, (tree, null))) {
            case ((#node(l,_,r), anc)) {
                     tree := splay(compareTo, l, k, r, anc);
                     true
                 };
            case ((#empty, null)) { false };
            case ((#empty, ?(recent, anc))) {
                     let (l,x,r) = switch recent {
                     case (#left(x,r)) { (#empty, x, r) };
                     case (#right(l,x)) { (l, x, #empty) };
                     };
                     tree := splay(compareTo, l, x, r, anc);
                     false
                 };
            };
        };
        public func remove(k: X) {
            if (not find(k)) { return };
            switch tree {
            case (#empty) { assert false; };
            case (#node(l,_,r)) {
                     let l_max = subtree_max(l);
                     switch l_max {
                     case (null) { tree := r };
                     case (?l_max) {
                              switch (path(compareTo, l_max, (l, null))) {
                              case ((#node(l_,_,r_), anc)) {
                                       let l = splay(compareTo, l_, l_max, r_, anc);
                                       switch l {
                                       case (#node(l, l_max, #empty)) {
                                                tree := #node(l, l_max, r);
                                            };
                                       case _ { assert false };
                                       };
                                   };
                              case ((#empty, anc)) { assert false };
                              };
                          };
                     };
                 };
            };
        };
        public func min() : ?X {
            subtree_min(tree);
        };
        public func fromArray(arr: [X]) {
            for (x in arr.vals()) {
                insert(x);
            }
        };
        public func entries() : I.Iter<X> { iter(tree) };
    };
}

