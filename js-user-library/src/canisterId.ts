import borc from "borc";
import * as blob from "./blob";
import { CborTag } from "./cbor";
import { Hex } from "./hex";

// Canister IDs are represented as u64 in the HTTP handler of the client.
export type CanisterId = borc.Tagged & { __canisterID__: void };

export const fromHex = (hex: Hex): CanisterId => {
  return new borc.Tagged(
    CborTag.Uint64LittleEndian,
    blob.fromHex(hex),
  ) as CanisterId;
};

export const toHex = (id: CanisterId) => blob.toHex(id.value);
