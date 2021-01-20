actor Certificate {

    public query func hello_query(name: Text) : async Text {
        "Hello, " # name # "!"
    };

    public func hello_update(name: Text) : async Text {
        "Hello, " # name # "!"
    }

}
