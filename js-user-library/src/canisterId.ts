import borc from "borc";
import { Buffer } from "buffer/";
import { CborTag } from "./cbor";
import { Hex } from "./hex";

// Canister IDs are represented as u64 in the HTTP handler of the client.
export type CanisterId = borc.Tagged & { __canisterID__: void };

export const fromHex = (hex: Hex): CanisterId => {
  return new borc.Tagged(
    CborTag.Uint64LittleEndian,
    Buffer.from(`0x${hex}`),
  ) as CanisterId;
};

export const toHex = (id: CanisterId) => id.value.toString("hex");
