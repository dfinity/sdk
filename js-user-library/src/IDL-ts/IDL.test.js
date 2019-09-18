import { IDL } from "./index";

test("hash", () => {
  const testHash = (string: string, hash: number) => {
    expect(IDL.hash(string)).toBe(hash);
  };

  testHash("", 0);
  testHash("id", 23515);
  testHash("description", 1595738364);
  testHash("short_name", 3261810734);
  testHash("Hi ☃", 1419229646);
});
