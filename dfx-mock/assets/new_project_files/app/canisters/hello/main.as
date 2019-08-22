/**
 * Sets of API in a canister are called Actors. An actor can be a structure or a class.
 */
actor class HelloActor() {

    /**
     * A public function can be called by anyone, from inside and outside the network.
     *
     * @return String an Hello World string.
     */
    public func hello(callback: (shared (Text) -> ())): () {
        /**
         * Returns a string by calling the callback function directly.
         */
        callback("Hello, World");
    }
}
