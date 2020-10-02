actor Greet(name: Text) {

    public query func greet() : async Text {
        "Hello, " # name # "!"
    }

}
