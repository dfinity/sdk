import Char "mo:base/Char";
import Option "mo:base/Option";

actor IsDigit {

    // Could not figure out how to have a Char parameter.
    public query func is_digit(text: Text) : async Bool {
        let ch = Option.unwrap<Char>(text.chars().next());
        Char.isDigit(ch);
    };

}
