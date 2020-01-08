import Friend "./friend.mo"

actor Greet {

    public query func greet(name: Text) : async Text {
        "1" # Friend.greet(name)
    }

}
