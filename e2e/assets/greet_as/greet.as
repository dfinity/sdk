actor Greet {

    public query func greet(name: Text) : async Text {
        "Hello, " # name # "!"
    }

}
