import Error "mo:base/Error";

shared ({caller = initializer}) actor class() {
  public shared (message) func test(): async Bool {
    message.caller == initializer
  }
}