import Verify "mo:passport-client/lib/Verifier";

module {
    // Don't verify users for sybil. It's useful for a test installation running locally.
    public let skipSybil = false; // true | false;
    public let minimumScore = 20.0;

    public let configScorer: Verify.Config = {
        scorerId =  12; //<NUMBER> // get it at https://scorer.gitcoin.co/
        scorerAPIKey = "<KEY>"; // get it at https://scorer.gitcoin.co/
        scorerUrl = "https://api.scorer.gitcoin.co";
    };
}