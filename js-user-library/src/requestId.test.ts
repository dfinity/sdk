// tslint:disable-next-line: max-line-length
// https://github.com/dfinity-lab/dfinity/blob/5fef1450c9ab16ccf18381379149e504b11c8218/docs/spec/public/index.adoc#request-ids

import { Request } from "./apiClient";
import { hash, requestIdOf } from "./index";
import { Int } from "./int";

const toHex = (bytes: Array<Int>): string => {
  return Array.from(bytes).map((x) => `00${x.toString(16)}`.slice(-2)).join("");
};

const testHashOfBlob = async (input: Array<Int>, expected: string) => {
  const buffer = new Uint8Array(input);
  const hashed = new Uint8Array(await hash(buffer));
  const hex = toHex(Array.from(hashed) as Array<Int>);
  expect(hex).toBe(expected);
};

const testHashOfString = async (input: string, expected: string) => {
  const encoded = (new TextEncoder()).encode(input);
  return testHashOfBlob(Array.from(encoded) as Array<Int>, expected);
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
      "92ca4c0ced628df1e7b9f336416ead190bd0348615b6f71a64b21d1b68d4e7e2",
    ),
    testHashOfBlob(
      [0, 0, 0, 0, 0, 0, 4, 210] as Array<Int>,
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
      [68, 73, 68, 76, 0, 253, 42] as Array<Int>,
      "6c0b2ae49718f6995c02ac5700c9c789d7b7862a0d53e6d40a73f1fcd2f70189",
    ),
  ]);
});

test("requestIdOf", async () => {
  const request: unknown = {
    request_type: "call",
    // 0x00000000000004D2
    // \x00\x00\x00\x00\x00\x00\x04\xD2
    // 0   0   0   0   0   0   4   210
    canister_id: [0, 0, 0, 0, 0, 0, 4, 210] as Array<Int>,
    method_name: "hello",
    // DIDL\x00\xFD*
    // D   I   D   L   \x00  \253  *
    // 68  73  68  76  0     253   42
    arg: [68, 73, 68, 76, 0, 253, 42] as Array<Int>,
  };

  const requestId = await requestIdOf(request as Request);

  expect(
    toHex(requestId as Array<Int>),
  ).toEqual(
    // "8781291c347db32a9d8c10eb62b710fce5a93be676474c42babc74c51858f94b",
    "8ef075dddcd53a18bc40d6a1b6ef7b559cc7945e8c7f898c689635a32148c71c",
  );
});
