module {
    public func rateLanguage(name: Text): Nat {
        switch name {
            case "rust" { 10 };
            case _ { 1 };
        }
    };
}
