/**
 * Sets of API in a canister are called Actors. An actor can be a structure or a class.
 */
actor class Hello() {

    /**
     * A public function can be called by anyone, from inside and outside the network.
     *
     * @return String an Hello World string.
     */
    public func hello() {
        /**
         * This will be sent to the cannisters' console in development, and dropped in production.
         */
        print("Hello called.");

        /**
         * Returns a string.
         */
        "Hello, World"
    }
}
