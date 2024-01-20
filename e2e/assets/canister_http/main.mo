import Types "types";
import Cycles "mo:base/ExperimentalCycles";
import Nat64 "mo:base/Nat64";
import Text "mo:base/Text";
import Blob "mo:base/Blob";
import Nat "mo:base/Nat";

shared actor class HttpQuery() = this {
    let MAX_RESPONSE_BYTES : Nat64 = 800000; // last seen ~90k
    let CYCLES_TO_PAY : Nat = 16_000_000_000;

    public func get_url(host : Text, url : Text) : async Text {
        let request_headers = [
            { name = "Host"; value = host },
            { name = "User-Agent"; value = "sdk-e2e-test" },
        ];

        let transform_context : Types.TransformContext = {
            function = transform;
            context = Blob.fromArray([]);
        };


        let request : Types.CanisterHttpRequestArgs = {
            url = url;
            max_response_bytes = ?MAX_RESPONSE_BYTES;
            headers = request_headers;
            body = null;
            method = #get;
            transform = ?transform_context;
        };

        Cycles.add(CYCLES_TO_PAY);
        let ic : Types.IC = actor ("aaaaa-aa");
        let response : Types.CanisterHttpResponsePayload = await ic.http_request(request);
        let result : Text = switch (Text.decodeUtf8(Blob.fromArray(response.body))) {
            case null "";
            case (?decoded) decoded;
        };
        result
    };

    public query func transform(raw : Types.TransformArgs) : async Types.CanisterHttpResponsePayload {
        let transformed : Types.CanisterHttpResponsePayload = {
            status = raw.response.status;
            body = raw.response.body;
            headers = [];
        };
        transformed;
    };
};