import Text "mo:base/Text";
import Time "mo:base/Time";
import Int "mo:base/Int";
import Logs "./Logs";

module {
    public type HttpRequest = {
        method: Text;
        url: Text;
        headers: [(Text, Text)];
        body: Blob;
    };
    public type HttpResponse = {
        status_code: Nat16;
        headers: [(Text, Text)];
        body: Blob;
    };
    func encode_single_value(kind: Text, name: Text, number: Int, desc: Text, time: Int) : Text {
        "# HELP " # name # " " # desc # "\n" #
        "# TYPE " # name # " " # kind # "\n" #
        name # " " # Int.toText(number) # " " # Int.toText(time) # "\n"
    };
    public func metrics(stats: Logs.Stats) : Blob {
        let now = Time.now() / 1_000_000;
        var result = "";
        result := result # encode_single_value("counter", "canister_count", stats.num_of_canisters, "Number of canisters deployed", now);
        result := result # encode_single_value("counter", "wasm_count", stats.num_of_installs, "Number of Wasm installed", now);
        result := result # encode_single_value("counter", "cycles_used", stats.cycles_used, "Cycles used", now);
        result := result # encode_single_value("counter", "out_of_capacity", stats.error_out_of_capacity, "Number of out of capacity requests", now);
        result := result # encode_single_value("counter", "total_wait_time", stats.error_total_wait_time, "Number of seconds waiting for out of capacity requests", now);
        result := result # encode_single_value("counter", "mismatch", stats.error_mismatch, "Number of mismatch requests including wrong nounce and timestamp", now);
        Text.encodeUtf8(result)
    };
}
