module {
    public func describeRating(rating: Nat): Text {
        switch rating {
            case 10 { "So hot right now." };
            case _ { "No comment." };
        }
    }
}
