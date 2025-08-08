import Int "mo:core/Int";

actor TestCore {

    public query func test_core() : async Bool {
        // Test a simple function from the core package
        Int.abs(-42) == 42;
    };

}
