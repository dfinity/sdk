/**
 * Sets of API in a canister are called Actors. An actor can be a structure or a class.
 */
actor class Hello() {

    /**
     * A public function can be called by anyone, from inside and outside the network.
     * 
     */
    public func hello() {
        print("Hello called.");
        "Hello, World"
    }
}
