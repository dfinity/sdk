import Rate "mo:rate/language";
import Describe "mo:describe/rating";

actor Packtool {

    public query func rate(name: Text) : async Text {
        let rating = Rate.rateLanguage(name);
        let description = Describe.describeRating(rating);
        name # ": " # description;
    }

}
