actor class Dependency(name: Text) {
    public query func greet() : async Text {
        return "Hello, " # name # "!";
    }
}