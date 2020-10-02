actor Greet(name: Nat) {

    public query func greet() : async Text {
        "Hello, " # name # "!"
    }

}
