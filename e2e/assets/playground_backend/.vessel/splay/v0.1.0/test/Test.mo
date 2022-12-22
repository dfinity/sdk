import M "mo:matchers/Matchers";
import Library "../src";
import S "mo:matchers/Suite";
import T "mo:matchers/Testable";
import Int "mo:base/Int";
import Debug "mo:base/Debug";
import Iter "mo:base/Iter";

let arr = [3,5,4,2,6,4,1,9,7,8];
let t = Library.Splay<Int>(Int.compare);
t.fromArray(arr);
for (x in arr.vals()) {
    assert(t.find(x) == true);
    t.remove(x);
    assert(t.find(x) == false);
    t.insert(x);
    assert(t.find(x) == true);
    assert(t.min() == ?1);
};
for (x in t.entries()) {
    assert(t.find(x) == true);
    t.remove(x);
    if (x < 9) {
        assert(t.min() == ?(x+1));
    } else {
        assert(t.min() == null);
    }
};

