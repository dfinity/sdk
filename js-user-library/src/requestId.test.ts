// tslint:disable-next-line: max-line-length
// https://github.com/dfinity-lab/dfinity/blob/5fef1450c9ab16ccf18381379149e504b11c8218/docs/spec/public/index.adoc#request-ids

import { BinaryBlob } from "./blob";
import * as blob from "./blob";
import { Request } from "./request";
import { hash, requestIdOf } from "./requestId";
import { RequestType } from "./requestType";

const testHashOfBlob = async (input: BinaryBlob, expected: string) => {
  const hashed = await hash(input);
  const hex = blob.toHex(hashed);
  expect(hex).toBe(expected);
};

const testHashOfString = async (input: string, expected: string) => {
  const encoded = (new TextEncoder()).encode(input);
  return testHashOfBlob(encoded as BinaryBlob, expected);
};

test("hash", async () => {
  return Promise.all([
    testHashOfString(
      "request_type",
      "769e6f87bdda39c859642b74ce9763cdd37cb1cd672733e8c54efaa33ab78af9",
    ),
    testHashOfString(
      "call",
      "7edb360f06acaef2cc80dba16cf563f199d347db4443da04da0c8173e3f9e4ed",
    ),
    testHashOfString(
      "callee", // The "canister_id" field was previously named "callee"
      "92ca4c0ced628df1e7b9f336416ead190bd0348615b6f71a64b21d1b68d4e7e2",
    ),
    testHashOfString(
      "canister_id",
      "0a3eb2ba16702a387e6321066dd952db7a31f9b5cc92981e0a92dd56802d3df9",
    ),
    testHashOfBlob(
      Uint8Array.from([0, 0, 0, 0, 0, 0, 4, 210]) as BinaryBlob,
      "4d8c47c3c1c837964011441882d745f7e92d10a40cef0520447c63029eafe396",
    ),
    testHashOfString(
      "method_name",
      "293536232cf9231c86002f4ee293176a0179c002daa9fc24be9bb51acdd642b6",
    ),
    testHashOfString(
      "hello",
      "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
    ),
    testHashOfString(
      "arg",
      "b25f03dedd69be07f356a06fe35c1b0ddc0de77dcd9066c4be0c6bbde14b23ff",
    ),
    testHashOfBlob(
      Uint8Array.from([68, 73, 68, 76, 0, 253, 42]) as BinaryBlob,
      "6c0b2ae49718f6995c02ac5700c9c789d7b7862a0d53e6d40a73f1fcd2f70189",
    ),
  ]);
});

test("requestIdOf", async () => {
  const request: Request = {
    request_type: "call" as RequestType,
    method_name: "hello",

    // 0x00000000000004D2
    // \x00\x00\x00\x00\x00\x00\x04\xD2
    // 0   0   0   0   0   0   4   210
    canister_id: Uint8Array.from([0, 0, 0, 0, 0, 0, 4, 210]) as BinaryBlob,

    // DIDL\x00\xFD*
    // D   I   D   L   \x00  \253  *
    // 68  73  68  76  0     253   42
    arg: Uint8Array.from([68, 73, 68, 76, 0, 253, 42]) as BinaryBlob,

    // These fields are not included in the example provided in the spec but we
    // provide them here to verify that they do not affect the request ID:
    // "Remove the fields that are only used for authentication"
    sender_pubkey: new Uint8Array(32) as BinaryBlob,
    sender_sig: new Uint8Array(64) as BinaryBlob,
  };

  const requestId = await requestIdOf(request);

  expect(
    blob.toHex(requestId),
  ).toEqual(
    "8781291c347db32a9d8c10eb62b710fce5a93be676474c42babc74c51858f94b",
  );
});
